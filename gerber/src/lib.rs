use std::{collections::HashMap, ptr::NonNull};

use corrida::attempt2::*;

struct Vertex {
    transitions: HashMap<char, NonNull<Vertex>>,
    accept: bool
}

pub struct DFA<'a> {
    arena: Arena<Vertex>,
    start_node: Option<&'a Vertex>
}


impl ArenaTrait<Vertex> for Arena<Vertex> {
    fn alloc(&self, vert: Vertex) -> &mut Vertex {
        let slot = self.alloc_core();

        (*slot_)
    }
}

impl<'a> DFA<'a> {
    fn new() -> Self {
        Self {
            arena: Arena::new(),
            start_node: None
        }
    }

    fn insert_vertex(&self)

}
