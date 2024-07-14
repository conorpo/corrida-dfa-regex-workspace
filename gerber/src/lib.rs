#![warn(missing_docs)]

//! A simple DFA library to construct state machines, fast allocation using an a custom Arena implementation, and safe construction using Rust's borrow checker.

use std::collections::{HashMap, HashSet};
use std::ptr::NonNull;
use std::cell::Cell;
use std::fmt::Display;

use impls::impls;

use corrida::*;

type VertexLink<Σ> = NonNull<DfaVertex<Σ>>;

/// A node in the DFA, contains is_accept and a transition hashmap.
pub struct DfaVertex<Σ: Eq + core::hash::Hash + Copy + Display> {
    transitions: HashMap<Σ, VertexLink<Σ>>,
    is_accept: bool
}


/// Provides an API for construction and simulation of a DFA structure. 
/// Symbol type Σ must be hashable and implement display (not asking for alot here..)
pub struct Dfa<Σ: Eq + core::hash::Hash + Copy + Display> {
    arena: Arena<DfaVertex<Σ>>,
    start_node: Cell<Option<VertexLink<Σ>>>
}


impl<Σ:Eq + core::hash::Hash + Copy + Display> DfaVertex<Σ> {
    fn new(accept: bool) -> Self {
        Self {
            transitions: HashMap::new(),
            is_accept: accept
        }
    }

    /// Updates the accept state
    pub fn set_accept(&mut self, accept: bool) {
        self.is_accept = accept;
    }

    fn get_transition(&self, symbol: &Σ) -> Option<&DfaVertex<Σ>> {
        self.transitions.get(symbol).map(|non_null_ref| {
            unsafe { &*(*non_null_ref).as_ptr() }
        })
    }

    /// Inserts the provided transitions into this vertex'es hashmap.
    pub fn append_transitions(&mut self, transitions: &[(Σ, Option<&DfaVertex<Σ>>)]){  
        let hashmap_iter = transitions.into_iter().map(|(symbol, opt)| {
            let vert_ref = opt.or(Some(self)).unwrap() as *const DfaVertex<Σ> as *mut DfaVertex<Σ>;

            (*symbol, NonNull::new(vert_ref).unwrap())
        }).collect::<HashMap<Σ,NonNull<DfaVertex<Σ>>>>();

        self.transitions.extend(hashmap_iter);
    }
}

impl<Σ:Eq + core::hash::Hash + Copy + Display> Dfa<Σ> {
    /// Creates a new DFA with no vertices, but an Arena ready for pushing verts.
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            start_node: Cell::new(None)
        }
    }

    /// Insert a Vertex that is either accept or not accept, also provide transitions if they are known already. Both attributes can be updated later.
    /// Transitions are a slice of tuples, where the 0th element is the symbol, and the 1st element is an option of a reference to the target vert. Putting None here implies a self-reference, as DFA's must have a reference to every symbol.
    pub fn insert_vertex(&self, is_accept: bool, transitions: &[(Σ, Option<&DfaVertex<Σ>>)]) -> &mut DfaVertex<Σ> {
        let mut new_vertex = DfaVertex::<Σ>::new(is_accept);
        new_vertex.append_transitions(transitions);
        self.arena.alloc(new_vertex)
    }

    /// Updates the start node, takes a reference to a vertex.
    pub fn set_start_node(&self, start_ref: &DfaVertex<Σ>) {
        self.start_node.set(Some(NonNull::new(start_ref as *const DfaVertex<Σ> as *mut DfaVertex<Σ>).unwrap()));        
    }   

    /// Tests the provided input sequence, returning true if the DFA ends at an accept state.
    pub fn simulate_slice(&mut self, input: &[Σ]) -> bool {
        // Would like to get rid of unsafe code in simulation API, but that would require not storing references as NonNulls
        let mut cur: &DfaVertex<Σ> = unsafe { self.start_node.get().expect("Start Node must be set before simulation.").as_mut() };
        for symbol in input {
            if let Some(next) = cur.get_transition(symbol) {
                cur = next;
            } else {
                // Transition did not exist, DFA error
                panic!("Transition not provided from current node on the '{symbol}' symbol.")
            }
        }
        cur.is_accept
    }

    pub fn simulate_iter(&mut self, input: impl Iterator<Item = Σ>) -> bool {
        let mut cur: &DfaVertex<Σ> = unsafe { self.start_node.get().expect("Start Node must be set before simulation.").as_mut() };
        for symbol in input {
            if let Some(next) = cur.get_transition(&symbol) {
                cur = next;
            } else {
                // Transition did not exist, DFA error
                panic!("Transition not provided from current node on the '{symbol}' symbol.")
            }
        }
        cur.is_accept
    }
}

/// A node in the Nfa, transitions are stored as a hashmap to a vec of target ptrs. At what point do we have too much indirections?
pub struct NfaVertex<Σ: Eq + core::hash::Hash + Copy + Display + Unpin > {
    transitions: HashMap<Option<Σ>, HashSet<NonNull<NfaVertex<Σ>>>>,
    is_accept: bool,
}

// Can't have duplicates, Maybe use Unionto merge to existing sets, is HashMap<HashSet<>> for each vertex worth it?
impl<Σ: Eq + core::hash::Hash + Copy + Display + Unpin> NfaVertex<Σ> {
    /// Appends provided transition slice to vertex transitions. Provide transitions as tuple
    /// 0th element is Some symbol, or None for an epsilon transition
    /// 1st element is Some target vert (reference), or None for a self transition.
    pub fn append_transitions(&mut self, transitions: &[(Option<Σ>, &[Option<&NfaVertex<Σ>>])]) {
        for (symbol, targets) in transitions {
            let mut transitions_hashset = HashSet::new();

            targets.into_iter().map(|target_ref| {
                let target_ref = target_ref.or(Some(self)).unwrap();
                NonNull::new(target_ref as *const NfaVertex<Σ> as *mut NfaVertex<Σ>).unwrap()
            }).for_each(|target_ptr| {
                transitions_hashset.insert(target_ptr);
            });

            if let Some(existing_set) = self.transitions.get_mut(symbol) {
                existing_set.extend(transitions_hashset);
            } else {
                self.transitions.insert(*symbol, transitions_hashset);
            }
        }
    }

    /// Creates a new NFA vertex, used internally, does not allocate to the arena itself.
    pub fn new_not_allocate(is_accept: bool) -> Self {
        Self {
            transitions: HashMap::new(),
            is_accept
        }
    }

    /// Updates the `is_accept` state of the vertex.
    pub fn set_accept(&mut self, is_accept: bool) {
        self.is_accept = is_accept;
    }
}

/// Provides an API for construction of an NFA. 
/// Symbol type Σ must be hashable and implement display. 
pub struct Nfa<Σ: Eq + core::hash::Hash + Copy + Display + Unpin> {
    arena: Arena<NfaVertex<Σ>>,
    start_vert: Cell<Option<NonNull<NfaVertex<Σ>>>>
}

impl <Σ: Eq + core::hash::Hash + Copy + Display + Unpin> Nfa<Σ> {
    /// Creates a new Nfa with no vertices, but an arena block ready for allocation
    pub fn new () -> Self {
        Self {
            arena: Arena::new(),
            start_vert: Cell::new(None)
        }
    }

    /// Inserts a a new vertex into the NFA, provide is_accept and transitions as a slice of tuples. 
    /// 0th element is Some symbol or none for epsilon.
    /// 1st element is a Vec of some target verts or none for a self reference.
    pub fn insert_node(&self, is_accept: bool, transitions: &[(Option<Σ>, &[Option<&NfaVertex<Σ>>])]) -> &mut NfaVertex<Σ> {
        let mut new_vertex = NfaVertex::<Σ>::new_not_allocate(is_accept);
        new_vertex.append_transitions(transitions);

        let slot = self.arena.alloc(new_vertex);
        slot
    }

    /// Updates the start node of the Nfa.
    pub fn set_start_node(&self, start_node: &NfaVertex<Σ>) {
        self.start_vert.set(Some({
            NonNull::new(start_node as *const NfaVertex<Σ> as *mut NfaVertex<Σ>).unwrap()
        }));
    }

    /// Turns our NFA into a DFA using subset construction.
    pub fn to_dfa(&self) -> Dfa<Σ> {

        todo!();
    }
}




#[cfg(test)]
mod test{
    //use crate::DfaVertex;

    //use super::Vertex;
    use super::{Dfa, Nfa, NfaVertex, DfaVertex};

    #[test]
    pub fn test_basics() {
        let mut dfa = Dfa::<char>::new();

        //println!("test");
        {
            let s_0 = dfa.insert_vertex(true, &[]);
            let s_1 = dfa.insert_vertex(false, &[]);
            s_0.append_transitions(&[('0', None),('1',Some(s_1))]);
            let s_2 = dfa.insert_vertex(false, &[('0', Some(s_1)),('1', None)]);
            s_1.append_transitions(&[('1', Some(s_0)),('0',Some(s_2))]);

            unsafe{
                let cur = (*s_0.transitions.get_mut(&'1').unwrap()).as_mut();
                let cur = (*cur.transitions.get_mut(&'0').unwrap()).as_mut();
                let cur = (*cur.transitions.get_mut(&'0').unwrap()).as_mut();
                let cur = (*cur.transitions.get_mut(&'1').unwrap()).as_mut();
    
                assert_eq!(cur as *const DfaVertex<char>, s_0 as *const DfaVertex<char>);
            }
            dfa.set_start_node(s_0);
        }
        assert_eq!(dfa.arena.len(), 3);

        assert_eq!(dfa.simulate_slice(&"1001".chars().collect::<Vec<char>>()), true);
        assert_eq!(dfa.simulate_slice(&"1000".chars().collect::<Vec<char>>()), false);
    }

    #[test]
    fn test_big_string() {
        let mut dfa = Dfa::<char>::new();
        {
            let s_0 = dfa.insert_vertex(true, &[]);
            let s_1 = dfa.insert_vertex(false, &[]);
            s_0.append_transitions(&[('0', None),('1',Some(s_1))]);
            let s_2 = dfa.insert_vertex(false, &[('0', Some(s_1)),('1', None)]);
            s_1.append_transitions(&[('1', Some(s_0)),('0',Some(s_2))]);
            dfa.set_start_node(s_0);
        }

        let mut test_vec = Vec::new();

        for _ in 0..1000000 {
            test_vec.push('1');
            test_vec.push('0');
            test_vec.push('0');
            test_vec.push('1');
        }

        assert_eq!(dfa.simulate_slice(&test_vec),true);
    }

    #[test]
    #[should_panic]
    fn test_dfa_missing_transition() {
        let mut dfa = Dfa::<char>::new();
        {
            let s_0 = dfa.insert_vertex(true, &[]);
            let s_1 = dfa.insert_vertex(false, &[]);
            s_0.append_transitions(&[('0', None),('1',Some(s_1))]);
            let s_2 = dfa.insert_vertex(false, &[('0', Some(s_1)),('1', None)]);
            s_1.append_transitions(&[('1', Some(s_0))]); //s_1 has no transition on the '0' symbol.
            dfa.set_start_node(s_0);
        }

        dfa.simulate_slice(&['1','0','0','1']);

    }

    #[test]
    fn test_dfa_repeated_transitions() {
        let mut dfa = Dfa::<char>::new();
        {
            let s_0 = dfa.insert_vertex(true, &[]);
            let s_1 = dfa.insert_vertex(false, &[]);
            s_0.append_transitions(&[('0', None),('1',Some(s_1))]);
            let s_2 = dfa.insert_vertex(false, &[('0', Some(s_1)),('1', None)]);
            s_1.append_transitions(&[('1', Some(s_0)),('0',Some(s_2))]);
            dfa.set_start_node(s_0);

            s_0.append_transitions(&[('0', Some(s_2)), ('1',Some(s_1))]);
            s_0.append_transitions(&[('0', None)]);

            // s_0 updated a few extra times, should not add new entries in the hashmap, but just union and update repeats
        }

        assert!(dfa.simulate_iter(vec!['1','0','0','1'].into_iter()));
    }

    #[test]
    fn test_nfa() {
        let nfa = Nfa::<char>::new();

        {

            let s_0 = nfa.insert_node(true, &[]);

            let s_1 = nfa.insert_node(false, &[]);
            let s_3 = nfa.insert_node(false, &[]);

            s_0.append_transitions(&[(Some('a'), &[Some(s_1),Some(s_3)])]);

            let s_2 = nfa.insert_node(true, &[]);
            s_1.append_transitions(&[(Some('b'),&[Some(s_2)])]);

            let s_4 = nfa.insert_node(false, &[]);
            let s_5 = nfa.insert_node(false, &[]);
            let s_6 = nfa.insert_node(true, &[]);
            let s_7 = nfa.insert_node(true, &[]);
        
            s_3.append_transitions(&[(None, &[Some(s_4), Some(s_5)])]);

            s_4.append_transitions(&[(Some('d'), &[Some(s_6)])]);
            s_5.append_transitions(&[(Some('c'), &[Some(s_7)])]);

            nfa.set_start_node(s_0);
        }
    }


    #[test]
    fn test_unpin() {
        assert!(impls::impls!(NfaVertex<char>: Unpin));
    }
}
