//#![feature(trait_alias)]

use std::{collections::HashMap, ptr::NonNull};
use std::cell::Cell;

use corrida::arena::*;

struct Vertex<Σ: Eq + core::hash::Hash + Copy> {
    transitions: HashMap<Σ, NonNull<Vertex<Σ>>>,
    accept: bool
}

pub struct DFA<'a, Σ: Eq + core::hash::Hash + Copy> {
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

    fn append_transitions(&mut self, transitions: &[(Σ, &mut Vertex<Σ> )]){        
        for (symbol, target_vert) in transitions.iter() {
            let target_ptr = unsafe {
                NonNull::new(*target_vert as *const Vertex<Σ> as *mut Vertex<Σ>).unwrap()
            };
            self.transitions.insert(*symbol, target_ptr);
        }
    }
}

impl<'a, Σ:Eq + core::hash::Hash + Copy> DFA<'a, Σ> {
    fn new() -> Self {
        Self {
            arena: Arena::new(),
            start_node: Cell::new(None)
        }
    }

    fn insert_vertex(&self, is_accept: bool, transitions: &[(Σ, &mut Vertex<Σ> )]) -> &mut Vertex<Σ> {
        let mut new_vertex = Vertex::<Σ>::new(is_accept);
        new_vertex.append_transitions(transitions);

        self.arena.alloc(new_vertex)
    }

    fn set_start_node(&self) {
        
    }
}

#[cfg(test)]
mod test{
    //use super::Vertex;
    use super::DFA;

    pub fn test_mixed_symbols() {
        let dfa = DFA::<char>::new();

        let s_0 = dfa.insert_vertex(true, &[]);
        dfa.start_node = 



        //let a = );
    }
}
