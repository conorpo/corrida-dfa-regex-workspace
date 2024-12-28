use corrida::Corrida;
use smallmap::Map;
use std::{collections::HashSet, ptr::NonNull};
use std::hash::Hash;
use smallvec::{Array, SmallVec};

use crate::dfa::{Dfa, PartialState};

// MARK: State
/// A state in the NFA, optimized for NFA which mostly contain nodes with small number of transitions. If a node has more than 'TARGETS_HINT' transitions on a given symbol, the target list will be heap allocated.
pub struct State<const TARGETS_HINT: usize, Σ: Eq + Hash + Copy> 
where 
    [NonNull<State<TARGETS_HINT, Σ>>; TARGETS_HINT]: Array<Item = NonNull<State<TARGETS_HINT, Σ>>>,
{
    transitions: Map<Option<Σ>, SmallVec<[NonNull<State<TARGETS_HINT, Σ>>; TARGETS_HINT]>>,
    is_accept: bool,
}

/// An iterator over the targets of the transitions from a state for a given symbol.
pub struct HomoIter<'a, const TARGETS_HINT: usize, Σ: Eq + Hash + Copy> 
where 
    [NonNull<State<TARGETS_HINT, Σ>>; TARGETS_HINT]: Array<Item = NonNull<State<TARGETS_HINT, Σ>>>,
{
    _self: &'a State<TARGETS_HINT, Σ>,
    transitions_vec: &'a [NonNull<State<TARGETS_HINT, Σ>>],
    index: usize,
}

impl <'a, const TARGETS_HINT: usize, Σ: Eq + Hash + Copy> Iterator for HomoIter<'a, TARGETS_HINT, Σ> 
where 
    [NonNull<State<TARGETS_HINT, Σ>>; TARGETS_HINT]: Array<Item = NonNull<State<TARGETS_HINT, Σ>>>,
{
    type Item = &'a State<TARGETS_HINT, Σ>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.transitions_vec.len() {
            return None;
        } else {
            let next = unsafe { self.transitions_vec[self.index].as_ref() };
            self.index += 1;
            Some(next)
        }
    }
}

impl<const TARGETS_HINT: usize, Σ: Eq + Hash + Copy> State<TARGETS_HINT, Σ> 
where
    [NonNull<State<TARGETS_HINT, Σ>>; TARGETS_HINT]: Array<Item = NonNull<State<TARGETS_HINT, Σ>>>,
{
    /// Creates a new state with no transitions and not accepting.
    pub fn new() -> Self {
        Self {
            transitions: Map::new(),
            is_accept: false,
        }
    }

    /// Pushes a transition to the state.
    pub fn push_transition(&mut self, symbol: Option<Σ>, target: Option<&Self>) {
        let target = (match target {
            Some(target) => NonNull::new(target as *const Self as *mut Self),
            None => NonNull::new(self as *const Self as *mut Self),
        }).unwrap();

        let vec = match self.transitions.get_mut(&symbol) {
            Some(existing) => existing,
            None => {
                self.transitions.insert(symbol, SmallVec::new());
                self.transitions.get_mut(&symbol).unwrap()
            }
        };

        vec.push(target);
    }

    /// Sets the state to be accepting or not.
    pub fn set_accept(&mut self, is_accept: bool) {
        self.is_accept = is_accept;
    }

    /// Returns an iterator over the transitions for the given symbol.
    pub fn get_transitions<'a>(&'a self, symbol: Option<Σ>) -> HomoIter<'a, TARGETS_HINT, Σ> {
        HomoIter {
            _self: self,
            transitions_vec: self.transitions.get(&symbol).map(|smallvec| smallvec.as_slice()).unwrap_or(&[]),
            index: 0,
        }
    }

    /// Returns if the state is accepting.
    pub fn is_accept(&self) -> bool {
        self.is_accept
    }
}


#[macro_export]
macro_rules! dynamic_state_creator {
    (($d: tt), $func_name: ident, $arena: expr, $symbol: ty) => {
        macro_rules! $func_name {
            ($TARGETS_HINT: expr, $d($is_accept:expr $d(,$transitions: expr)?)? ) => {
                {
                    let new_state = $arena.alloc::<DynamicState<$TARGETS_HINT, $symbol>>(DynamicState::new());
                    $d(
                        new_state.set_accept($is_accept);
                        $d(
                            let transitions: &[(_, Option<NonNull<DynamicState<$TARGETS_HINT, $symbol>>)] = $transitions;
                            transitions.iter().for_each(|transition| new_state.push_transition(*transition));
                        )?
                    )?
                    new_state
                }
            };
        }
    };
}

/// Creates a macro for creating states in the NFA.
#[macro_export]
macro_rules! nfa_state_creator {
    (($d: tt), $func_name: ident, $arena: expr, $symbol: ty, $TARGETS_HINT: expr) => {
        macro_rules! $func_name {
            ($d($is_accept:expr $d(,$transitions: expr)?)? ) => {
                {
                    let new_state = $arena.alloc(State::<$TARGETS_HINT, $symbol>::new());
                    $d(
                        new_state.set_accept($is_accept);
                        $d(
                            let transitions: &[(_, Option<&State::<$TARGETS_HINT, $symbol>>)] = $transitions;
                            transitions.iter().for_each(|&(symbol, target)| new_state.push_transition(symbol, target));
                        )?
                    )?
                    new_state
                }
            };
        }        
    }
}

// MARK: NFA
/// A non-deterministic fintie automaton.
pub struct Nfa<'a ,T> {
    start_node: &'a T,
}

impl<'a, const TARGETS_HINT:usize, Σ: Eq + Hash + Copy>  Nfa<'a, State<TARGETS_HINT, Σ>> 
where 
    [NonNull<State<TARGETS_HINT, Σ>>; TARGETS_HINT]: Array<Item = NonNull<State<TARGETS_HINT, Σ>>>,
{
    /// Creates a new NFA with the given start node, consumes the arena where the nodes were made.
    /// TODO: This can just take any arena, and then we would be able to change the NFA after creation. is that a good idea?
    pub fn new(start_node: &'a State<TARGETS_HINT, Σ>) -> Self {
        Self {
            start_node
        }
    }

    pub fn as_dfa(&self) -> Dfa<'a, Σ, PartialState<Σ>> {
        let mut hash_map = HashMap::new();

        fn set_hash(states: &[&State<TARGETS_HINT, Σ>]) -> 

        let mut current_states: SmallVec<[&State<TARGETS_HINT, Σ>; 32]> = SmallVec::from_elem(self.start_node, 1);
        let mut set: HashSet<*const State<TARGETS_HINT, Σ>> = HashSet::from([self.start_node as *const State<TARGETS_HINT, Σ>]);
        set.reserve(32);
        let mut i = 0;
        while i < current_states.len() {
            let state = current_states[i];
            for next in state.get_transitions(None) {
                if set.insert(next as *const State<TARGETS_HINT, Σ>) {
                    current_states.push(next);
                }
            }
            i += 1;
        }



        todo!()
    }

    /// Simulates the NFA on the given input, returning if the NFA accepts the input.
    pub fn simulate_iter(&self, input: impl Iterator<Item = Σ>) -> bool {
        let mut current_states: SmallVec<[&State<TARGETS_HINT, Σ>; 32]> = SmallVec::from_elem(self.start_node, 1);
        let mut set: HashSet<*const State<TARGETS_HINT, Σ>> = HashSet::from([self.start_node as *const State<TARGETS_HINT, Σ>]);
        set.reserve(32);
        let mut next_states: SmallVec<[&State<TARGETS_HINT, Σ>; 32]> = SmallVec::new();

        let mut i = 0;
        while i < current_states.len() {
            let state = current_states[i];
            for next in state.get_transitions(None) {
                if set.insert(next as *const State<TARGETS_HINT, Σ>) {
                    current_states.push(next);
                }
            }
            i += 1;
        }

        for symbol in input {
            set.clear();

            // Symbol Transition
            for cur in current_states.into_iter() {
                for next in cur.get_transitions(Some(symbol)) {
                    if set.insert(next as *const State<TARGETS_HINT, Σ>) {
                        next_states.push(next);
                    }
                }
            }

            // Epsilon Closure
            let mut i = 0;
            while i < next_states.len() {
                let state = next_states[i];
                for next in state.get_transitions(None) {
                    if set.insert(next as *const State<TARGETS_HINT, Σ>) {
                        next_states.push(next);
                    }
                }
                i += 1;
            }

            (current_states, next_states) = (next_states, SmallVec::new());
        }

        current_states.into_iter().any(|state| state.is_accept)
    }

    /// Simulates the NFA on the given input, returning if the NFA accepts the input.
    pub fn simulate_slice(&self, input: &[Σ]) -> bool {
        self.simulate_iter(input.iter().copied())
    }

    /// Simulates the NFA on the given input, returning if the NFA accepts the input. Will infinite loop on epsilon loops, so only use for 'friendly' NFAs where specific states are not reached many times at the same token.
    pub fn simulate_iter_friendly(&self, input: impl Iterator<Item = Σ>) -> bool {
        let mut current_states: SmallVec<[&State<TARGETS_HINT, Σ>; 32]> = SmallVec::from_elem(self.start_node, 1);
        let mut next_states: SmallVec<[&State<TARGETS_HINT, Σ>; 32]> = SmallVec::new();

        let mut i = 0;
        while i < current_states.len() {
            let state = current_states[i];
            for next in state.get_transitions(None) {
                current_states.push(next);
            }
            i += 1;
        }

        //? In a well formed NFA, i believe that reaching the same state from two different paths is very rare.
        for symbol in input {
            for cur in current_states.into_iter() {
                for next in cur.get_transitions(Some(symbol)) {
                    next_states.push(next);
                }
            }

            let mut i = 0;
            while i < next_states.len() {
                
                for next in next_states[i].get_transitions(None) {
                    next_states.push(next);
                }
                i += 1;
            }
            (current_states, next_states) = (next_states, SmallVec::new());
        }

        current_states.into_iter().any(|state| state.is_accept)
    }

    /// Simulates the NFA on the given input, returning if the NFA accepts the input. Will infinite loop on epsilon loops, so only use for 'friendly' NFAs where specific states are not reached many times at the same token.
    pub fn simulate_slice_friendly(&self, input: &[Σ]) -> bool {
        self.simulate_iter_friendly(input.iter().copied())
    }
}

//MARK: Tests
#[cfg(test)]
mod test {
    use std::time::Instant;

    use super::*;

    use corrida::Corrida;

    #[test]
    fn test_homo() {
        let arena = Corrida::new();

        nfa_state_creator!(($), new_state, arena, char, 2);

        let start_node = {
            let s_0 = new_state!();

            let s_1 = new_state!(true, &[(Some('1'), None)]);
            let s_2 = new_state!(false, &[(Some('1'), None)]);
            s_2.push_transition(Some('0'), Some(s_1));
            s_1.push_transition(Some('0'), Some(s_2));
            s_0.push_transition(None, Some(s_1));

            let s_3 = new_state!(true, &[(Some('0'), None)]);
            let s_4 = new_state!(false, &[(Some('0'), None)]);
            s_4.push_transition(Some('1'), Some(s_3));
            s_3.push_transition(Some('1'), Some(s_4));
            s_0.push_transition(None, Some(s_3));

            s_0
        };

        let nfa = Nfa::new(start_node);
        assert_eq!(nfa.simulate_slice_friendly(&[]), true);
        assert_eq!(nfa.simulate_slice_friendly(&['0','0']),true);
        assert_eq!(nfa.simulate_slice_friendly(&['0','1']), false);
        assert_eq!(nfa.simulate_slice_friendly(&['1','1']),true);
        assert_eq!(nfa.simulate_slice_friendly(&['0','0','0','1','0','1','0']),true);
        assert_eq!(nfa.simulate_slice_friendly(&['0','0','0','1','1','1','0','0']),false);
    }

    #[test]
    pub fn test_big_friendly() {
        let arena = Corrida::new();
        nfa_state_creator!(($), new_state, arena, u8, 2);

        let start_node = {
            let s_0 = new_state!(false, &[(Some(1), None), (Some(0), None)]);
            let s_1 = new_state!();
            s_0.push_transition(Some(1), Some(s_1));
            let s_2 = new_state!();
            s_1.push_transition(Some(0), Some(s_2));
            s_1.push_transition(Some(1), Some(s_2));
            let s_3 = new_state!();
            s_2.push_transition(Some(0), Some(s_3));
            s_2.push_transition(Some(1), Some(s_3));
            let s_4 = new_state!(true);
            s_3.push_transition(Some(0), Some(s_4));
            s_3.push_transition(Some(1), Some(s_4));
            
            s_0
        };

        let nfa = Nfa::new(start_node);
        let mut test = vec![1; 1_000_000];
        test.extend([0,0,0]);

        let start = Instant::now();
        assert_eq!(nfa.simulate_slice_friendly(&test),true);
        test.push(1);
        assert_eq!(nfa.simulate_slice_friendly(&test),false);

        println!("Big Input Friednly {:?}", start.elapsed());
    }

    #[test]
    pub fn test_big() {
        let arena = Corrida::new();
        nfa_state_creator!(($), new_state, arena, u8, 2);

        let start_node = {
            let s_0 = new_state!(false, &[(Some(1), None), (Some(0), None)]);
            let s_1 = new_state!();
            s_0.push_transition(Some(1), Some(s_1));
            let s_2 = new_state!();
            s_1.push_transition(Some(0), Some(s_2));
            s_1.push_transition(Some(1), Some(s_2));
            let s_3 = new_state!();
            s_2.push_transition(Some(0), Some(s_3));
            s_2.push_transition(Some(1), Some(s_3));
            let s_4 = new_state!(true);
            s_3.push_transition(Some(0), Some(s_4));
            s_3.push_transition(Some(1), Some(s_4));
            
            s_0
        };

        let nfa = Nfa::new(start_node);
        let mut test = vec![1; 1_000_000];
        test.extend([0,0,0]);

        let start = Instant::now();
        assert_eq!(nfa.simulate_slice(&test),true);
        test.push(1);
        assert_eq!(nfa.simulate_slice(&test),false);

        println!("Big Input {:?}", start.elapsed());
    }

    #[test]
    pub fn test_loop() {
        let arena = Corrida::new();
        nfa_state_creator!(($), new_state, arena, u8, 2);

        let start_node = {
            let s_0 = new_state!(false, &[(Some(1), None), (Some(0), None)]);
            let mut cur = new_state!(false);
            s_0.push_transition(None, Some(cur));

            for _ in (0..100).rev() {
                let new = new_state!(false);
                cur.push_transition(None, Some(new));
                cur = new;
            }
            cur.set_accept(true);
            cur.push_transition(None, Some(s_0));
            
            s_0
        };

        let nfa = Nfa::new(start_node);
        let mut test = vec![1; 100_000];
        let start = Instant::now();
        assert_eq!(nfa.simulate_slice(&test),true);
        test.push(1);
        assert_eq!(nfa.simulate_slice(&test),true);

        println!("Epsilon Closures {:?}", start.elapsed());
    }
}



