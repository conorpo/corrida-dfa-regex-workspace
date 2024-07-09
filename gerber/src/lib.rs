use std::{collections::HashMap, ptr::NonNull};

use corrida::arena::*;

struct Vertex<Σ> {
    transitions: HashMap<Σ, NonNull<Vertex<Σ>>>,
    accept: bool
}

pub struct DFA<'a, Σ> {
    arena: Arena<Vertex<Σ>>,
    start_node: Option<&'a Vertex<Σ>>
}

impl<'a, Σ> DFA<'a, Σ> {
    fn new() -> Self {
        Self {
            arena: Arena::new(),
            start_node: None
        }
    }

    fn insert_vertex(&self, transitions: &[(Σ, &mut Vertex<Σ> )]) {
        self.arena.alloc_core()
    }
}

#[cfg(test)]
mod test{
    //use super::Vertex;
    use super::DFA;

    pub fn test_mixed_symbols() {
        let dfa = DFA::new();

        let a = );
    }
}
