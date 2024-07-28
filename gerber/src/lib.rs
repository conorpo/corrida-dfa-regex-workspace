#![warn(missing_docs)]

//! A simple DFA library to construct state machines, fast allocation using an a custom Arena implementation, and safe construction using Rust's borrow checker.

use std::hash::Hash;
use std::ptr::NonNull;
use std::cell::Cell;
use std::fmt::Display;



use hashbrown::hash_map::{Entry, OccupiedEntry};
use hashbrown::HashMap;
use smallvec::*;


use corrida::*;

type VertexLink<Σ> = NonNull<DfaVertex<Σ>>;

/// A node in the DFA, contains is_accept and a transition hashmap.
pub struct DfaVertex<Σ: Eq + Hash + Copy> {
    transitions: HashMap<Σ, VertexLink<Σ>>,
    is_accept: bool
}


const DFA_BLOCK_SIZE_BYTES: usize = 2048;

/// Provides an API for construction and simulation of a DFA structure. 
/// Symbol type Σ must be hashable and implement display (not asking for alot here..)
pub struct Dfa<Σ: Eq + Hash + Copy> {
    arena: Corrida::<DFA_BLOCK_SIZE_BYTES>,
    start_node: Cell<Option<VertexLink<Σ>>>
}


impl<Σ:Eq + Hash + Copy> DfaVertex<Σ> {
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

impl<Σ:Eq + Hash + Copy> Dfa<Σ> {
    /// Creates a new DFA with no vertices, but an Arena ready for pushing verts.
    pub fn new() -> Self {
        Self {
            arena: Corrida::<{DFA_BLOCK_SIZE_BYTES}>::new(),
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
                panic!("Transition not provided from current node on the symbol.")
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
                panic!("Transition not provided from current node on the symbol.")
            }
        }
        cur.is_accept
    }
}

/// A node in the Nfa, transitions are stored as a hashmap to a vec of target ptrs. At what point do we have too much indirections?
// struct State<Σ: Eq + Hash> {
//     is_accept: bool,
//     transitions: HashMap<
//         Option<Σ>, Vec<NonNull<State<Σ>>>, // TODO: CHANGE HASH,  CHANGE ALOCATOR
//     >,
// }

// impl<Σ: Eq + Hash> State<Σ> {
//     pub fn new() -> Self {
//         Self {
//             is_accept: false,
//             transitions: HashMap::new()
//         }
//     }

//     /// Add just one transition
//     pub fn push_transition(&mut self, on: Option<Σ>, to: &State<Σ>) {
//         // SAFETY
//         // State Reference is garunteed to be a valid reference to a State
//         // We are only using NonNulls for niche optimizations, Nfa won't ever need to mutate after transitioning.
//         let ptr = unsafe {
//             NonNull::new_unchecked(
//                 to as *const State<Σ> as *mut State<Σ>
//             )
//         };

//         // How to prevent duplicate transitions on this one?
//         match self.transitions.entry(on) {
//             Entry::Occupied(existing) => {
//                 existing.into_mut().push(ptr);
//             },
//             Entry::Vacant(slot) => {
//                 slot.insert(vec![ptr]);
//             }
//         }    

//     }

//     /// Think about good way to do tihs.
//     pub fn push_transitions<'a>(&mut self, input: &[(Option<Σ>, &State<Σ>)])
//     where
//         Σ: 'a + Copy 
//     {
//         let input_slice: &[(Option<Σ>, &State<Σ>)] = input.into();

//         for transition in input_slice {
//             self.push_transition(transition.0, transition.1)
//         }
//     }
// }


// /// Nfa that provides shared references to nodes, transitions are owned by the Nfa itself.
// // TODO: This could use typed allocator, or could use the generic allocator version
// pub struct NfaSupreme<Σ: Eq + Hash> {
//     arena: Arena<State<Σ>>,
//     starting_node: Cell<Option<NonNull<State<Σ>>>>,
// }

// impl<Σ: Eq + Hash> NfaSupreme<Σ> {
//     /// Creates a new shared ref nfa.
//     pub fn new() -> Self {
//         Self {
//             arena: Arena::new(),
//             starting_node: Cell::new(None),
//         }
//     }
    
//     /// Updates the entry point of the NFA
//     pub fn set_start_node(&self, target: &State<Σ>) {
//         self.starting_node.set(Some(NonNull::from(target)));
//     }

//     /// 
//     pub fn insert_state(&self) -> &mut State<Σ> {
//         self.arena.alloc(State::new())
//     }
//     /// Converts this NFA into a DFA using subset construction.
//     pub fn into_dfa(self) -> Dfa<Σ> 
//     where
//         Σ: Copy
//     {
//         todo!();
//     }
// }




#[cfg(test)]
mod test {
    //use crate::DfaVertex;

    use corrida::Corrida;

    //use super::Vertex;
    use super::{Dfa, DfaVertex};

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

    // #[test]
    // fn test_nfa() {
    //     let mut arena = Arena::<State<char>>::new();
    //     let mut nfa = NfaSupreme::<char>::new();

    //     {
    //         let s_0 = nfa.insert_state();

    //         let s_1 = nfa.insert_state();
    //         let s_3 = nfa.insert_state();

    //         s_0.push_transition(Some('a'), s_1);
    //         s_0.push_transition(Some('a'), s_3);

    //         let s_2 = nfa.insert_state();
    //         s_1.push_transition(Some('b'), s_2);

    //         let s_4 = nfa.insert_state();
    //         let s_5 = nfa.insert_state();
    //         let s_6 = nfa.insert_state();
    //         let s_7 = nfa.insert_state();

    //         s_3.push_transitions(&[(None, s_4),(None, s_5)]);

    //         s_4.push_transition(Some('d'), s_6);
    //         s_5.push_transition(Some('c'), s_7);

    //         nfa.set_start_node(s_0);
    //     }

    //     let dfa = nfa.into_dfa();
    // }


}
