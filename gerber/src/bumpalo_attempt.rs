//! Attempt to create it using bumpalo, to see how they handle interior mutability and self references.
//! In this one structs like Nfa and State are specialized for graphs produced by the Thompson construction, they are not intended to be used as general NFA api.

use bumpalo::*;

// Simple states always have either one transition, or an additional epsilon transition
struct SimpleState<'a> {
    main_transition: Option<StateEnum<'a>>,
    extra_epsilon: Option<StateEnum<'a>>
}

// Union states have a dynamic amount of epsilon transitions.
struct UnionState<'a> {
    transitions: Vec<StateEnum<'a>>
}

trait State {
    fn new() -> Self;
}

impl<'a> State for SimpleState<'a> {
    fn new() -> Self {
        Self {
            main_transition: None,
            extra_epsilon: None,
        }
    }
}

impl<'a> State for UnionState<'a>{
    fn new() -> Self {
        Self {
            transitions: Vec::new(),
        }
    }
}

enum StateEnum<'a> {
    Simple(&'a SimpleState<'a>),
    Union(&'a UnionState<'a>)
}

struct NFA {
    bump: Bump
}

impl NFA {
    pub fn new(size_hint_bytes: usize) -> Self {
        Self {
            bump: Bump::with_capacity(size_hint_bytes)
        }
    }

    pub fn insert_simple(&self) -> &mut SimpleState {
        self.bump.alloc(SimpleState::new())
    }

    pub fn insert_union(&self) -> &mut UnionState {
        self.bump.alloc(UnionState::new())
    }
}


#[cfg(test)]
mod test {
    use super::NFA;
    // use super::SimpleState;
    // use super::UnionState;
    use super::StateEnum;

    #[test]
    pub fn create_basic_nfa() {
        let mut nfa = NFA::new(10);


        let basic = nfa.insert_simple();
        let two = nfa.insert_simple();
        basic.main_transition = Some(StateEnum::Simple(two));
        two.extra_epsilon = Some(StateEnum::Simple(basic));
        //two.main_transition = Some(StateEnum::Simple(basic));

        // nfa.bump.reset();

        // drop(basic);


    }
}

// pub struct RegexParser {
//     arena: Bump,
// }

// impl RegexParser {
//     pub fn from() -> Self {
//         Self {
//             arena: Bump::with_capacity(),
//         }
//     }
// }