//#![feature(trait_alias)]

use std::{collections::HashMap, ptr::NonNull};
use std::cell::Cell;

use corrida::arena::*;

struct Vertex<Σ: Eq + core::hash::Hash + Copy> {
    transitions: HashMap<Σ, NonNull<Vertex<Σ>>>,
    accept: bool
}

pub struct Dfa<'a, Σ: Eq + core::hash::Hash + Copy> {
    arena: Arena<Vertex<Σ>>,
    start_node: Cell<Option<&'a Vertex<Σ>>>
}


impl<Σ:Eq + core::hash::Hash + Copy> Vertex<Σ> {
    fn new(accept: bool) -> Self {
        Self {
            transitions: HashMap::new(),
            accept
        }
    }

    fn set_accept(&mut self, accept: bool) {
        self.accept = accept;
    }

    fn append_transitions(&mut self, transitions: &[(Σ, Option<&Vertex<Σ>>)]){  
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
                NonNull::new(vert_ref as *const Vertex<Σ> as *mut Vertex<Σ>).unwrap()
            };
            self.transitions.insert(*symbol, target_ptr);
        }
    }
}

impl<'a, Σ:Eq + core::hash::Hash + Copy> Dfa<'a, Σ> {
    fn new() -> Self {
        Self {
            arena: Arena::new(),
            start_node: Cell::new(None)
        }
    }

    fn insert_vertex(&self, is_accept: bool, transitions: &[(Σ, Option<&Vertex<Σ>>)]) -> &mut Vertex<Σ> {
        println!("1");
        let mut new_vertex = Vertex::<Σ>::new(is_accept);
        println!("2");
        new_vertex.append_transitions(transitions);
        println!("3");
        self.arena.alloc(new_vertex)
    }

    fn set_start_node(&self, start_ref: &'a Vertex<Σ>) {
        self.start_node.set(Some(start_ref));        
    }
}

#[cfg(test)]
mod test{
    use crate::Vertex;

    //use super::Vertex;
    use super::Dfa;

    #[test]
    pub fn test_mixed_symbols() {
        let dfa = Dfa::<char>::new();

        println!("test");
        let s_0 = dfa.insert_vertex(true, &[]);
        //let s_1 = dfa.insert_vertex(false, &[]);
        // s_0.append_transitions(&[('0', None),('1',Some(s_1))]);
        // let s_2 = dfa.insert_vertex(false, &[('0', Some(s_1)),('1', None)]);
        // s_1.append_transitions(&[('1', Some(s_0)),('0',Some(s_2))]);

        // assert_eq!(dfa.arena.len(), 3);
        
        // //dfa.set_start_node(s_0);
        // println!("Done");

        // unsafe{
        //     let cur = (*s_0.transitions.get_mut(&'1').unwrap()).as_mut();
        //     let cur = (*cur.transitions.get_mut(&'0').unwrap()).as_mut();
        //     let cur = (*cur.transitions.get_mut(&'0').unwrap()).as_mut();
        //     let cur = (*cur.transitions.get_mut(&'1').unwrap()).as_mut();

        //     assert_eq!(cur as *const Vertex<char>, s_0 as *const Vertex<char>);
        // }

        //let a = );
    }
}
