#![warn(missing_docs)]

//! A simple DFA library to construct state machines, fast allocation using an a custom Arena implementation, and safe construction using Rust's borrow checker.


use std::{collections::HashMap, ptr::NonNull};
use std::cell::Cell;
use std::fmt::Display;

use corrida::arena::{self, *};

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
        for (symbol, target_vert) in transitions.iter() {
            let vert_ref = match target_vert {
                Some(vert_ref) => {
                    *vert_ref
                },
                None => { 
                    //DFA, so this is used to indicate a self reference. Actual pointers not exisiting in the hashmap leads to an option where None means transition not defined.
                    &*self
                }
            };


            let target_ptr = unsafe {
                NonNull::new(vert_ref as *const DfaVertex<Σ> as *mut DfaVertex<Σ>).unwrap()
            };
            self.transitions.insert(*symbol, target_ptr);
        }
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

    pub fn simulate_iter(&mut self, input: impl Iterator<Item = Σ>) {
        
    }
}

/// A node in the Nfa, transitions are stored as a hashmap to a vec of target ptrs. At what point do we have too much indirections?
pub struct NfaVertex<Σ: Eq + core::hash::Hash + Copy + Display > {
    transitions: HashMap<Option<Σ>, Vec<NonNull<NfaVertex<Σ>>>>,
    is_accept: bool,
}


pub struct Nfa<Σ: Eq + core::hash::Hash + Copy + Display> {
    arena: Arena<NfaVertex<Σ>>,
    start_vert: Cell<Option<NonNull<NfaVertex<Σ>>>>
}

impl<Σ: Eq + core::hash::Hash + Copy + Display> NfaVertex<Σ> {
    //Creates a new nfa vertex with no transitions, provide is_accept.
    pub fn new(is_accept: bool) -> Self {
        Self {
            transitions: HashMap::new(),
            is_accept
        }
    }
}

impl <Σ: Eq + core::hash::Hash + Copy + Display> Nfa<Σ> {
    pub fn new () -> Self {
        Self {
            arena: Arena::new(),
            start_vert: Cell::new(None)
        }
    }

    /// Inserts a a new vertex into the NFA, provide is_accept and transitions as a slice of tuples. 
    /// 0th element is Some symbol or none for epsilon.
    /// 1st element is a Vec of some target verts or none for a self reference.
    pub fn insert_vertex(&self, is_accept: bool, transitions: &[(Option<Σ>, Vec<Option<&NfaVertex<Σ>>>)]) -> &mut NfaVertex<Σ> {
        let mut new_vertex = NfaVertex::<Σ>::new(is_accept);
        todo!();
    }
}




#[cfg(test)]
mod test{
    use crate::DfaVertex;

    //use super::Vertex;
    use super::Dfa;

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
    fn check_missing_transition() {
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
    fn test_repeat_transitions() {
    }
}
