use corrida::Corrida;
use hashbrown::HashSet;
use small_map::SmallMap;
use std::{iter::Map, ptr::NonNull};
use std::hash::Hash;
use smallvec::{Array, SmallVec};

// pub trait NfaState<Σ: Eq + Hash + Copy> {
//     fn set_accept(&mut self, is_accept: bool);
//     //fn get_transitions(&self, symbol: Option<Σ>) -> Option<Iter<Item = (&dyn NfaState)>>;
// }


// pub struct DynamicState<const symbols_hint: usize, const targets_hint: usize, Σ: Eq + Hash + Copy> 
// where 
//     [Option<Σ>; symbols_hint]: Array<Item = Option<Σ>>,
//     [Option<NonNull<dyn NfaState<Σ>>>; targets_hint]: Array<Item = Option<NonNull<dyn NfaState<Σ>>>>
// {
//     transitions: SmallMap<symbols_hint, Option<Σ>, SmallVec<[Option<NonNull<dyn NfaState<Σ>>>; targets_hint]>>,
//     is_accept: bool,
// }

// impl<const symbols_hint: usize, const targets_hint: usize, Σ: Eq + Hash + Copy> NfaState<Σ> for DynamicState<symbols_hint, targets_hint, Σ> 
// where 
//     [Option<Σ>; symbols_hint]: Array<Item = Option<Σ>>,
//     [Option<NonNull<dyn NfaState<Σ>>>; targets_hint]: Array<Item = Option<NonNull<dyn NfaState<Σ>>>>,
// {
//     fn set_accept(&mut self, is_accept: bool) {
//         self.is_accept = is_accept;
//     }
// }

// struct DynamicIter<'a, Σ: Eq + Hash + Copy> 
// {
//     _self: &'a dyn NfaState<Σ>,
//     transitions_vec: &'a [Option<NonNull<dyn NfaState<Σ>>>],
//     index: usize,
// }

// impl <'a, Σ: Eq + Hash + Copy> Iterator for DynamicIter<'a, Σ> {
//     type Item = &'a dyn NfaState<Σ>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.index >= self.transitions_vec.len() {
//             return None;
//         } else {
//             let next = match self.transitions_vec[self.index] {
//                 Some(target) => {
//                     Some(unsafe { target.as_ref() })
//                 },
//                 None => {
//                     Some(self._self as &dyn NfaState<Σ>)
//                 }
//             };
//             self.index += 1;
//             next
//         }
//     }
// }

// impl<const symbols_hint: usize, const targets_hint: usize, Σ: Eq + Hash + Copy> DynamicState<symbols_hint, targets_hint, Σ> 
// where 
//     [Option<Σ>; symbols_hint]: Array<Item = Option<Σ>>,
//     [Option<NonNull<dyn NfaState<Σ>>>; targets_hint]: Array<Item = Option<NonNull<dyn NfaState<Σ>>>>
// {
//     pub fn new() -> Self {
//         Self {
//             transitions: SmallMap::new(),
//             is_accept: false,
//         }
//     }

//     pub fn push_transition(&mut self, (symbol, target): (Option<Σ>, Option<NonNull<dyn NfaState<Σ>>>)) {
//         let vec = match self.transitions.get_mut(&symbol) {
//             Some(existing) => existing,
//             None => {
//                 self.transitions.insert(symbol, SmallVec::new());
//                 self.transitions.get_mut(&symbol).unwrap()
//             }
//         };

//         vec.push(target);
//     }

//     pub fn set_accept(&mut self, is_accept: bool) {
//         self.is_accept = is_accept;
//     }

//     pub fn get_transitions<'a>(&'a self, symbol: Option<Σ>) -> DynamicIter<'a, Σ> {
//         DynamicIter {
//             _self: self,
//             transitions_vec: self.transitions.get(&symbol).map(|smallvec| smallvec.as_slice()).unwrap_or(&[]),
//             index: 0,
//         }
//     }
// }


// pub struct HomoState<const symbols_hint: usize, const targets_hint: usize, Σ: Eq + Hash + Copy> 
// where 
//     [Option<Σ>; symbols_hint]: Array<Item = Option<Σ>>,
//     [NonNull<HomoState<symbols_hint, targets_hint, Σ>>; targets_hint]: Array<Item = NonNull<HomoState<symbols_hint, targets_hint, Σ>>>,
// {
//     transitions: SmallMap<symbols_hint, Option<Σ>, SmallVec<[NonNull<HomoState<symbols_hint, targets_hint, Σ>>; targets_hint]>>,
//     is_accept: bool,
// }

// struct HomoIter<'a, const symbols_hint: usize, const targets_hint: usize, Σ: Eq + Hash + Copy> 
// where 
//     [Option<Σ>; symbols_hint]: Array<Item = Option<Σ>>,
//     [NonNull<HomoState<symbols_hint, targets_hint, Σ>>; targets_hint]: Array<Item = NonNull<HomoState<symbols_hint, targets_hint, Σ>>>,
// {
//     _self: &'a HomoState<symbols_hint, targets_hint, Σ>,
//     transitions_vec: &'a [NonNull<HomoState<symbols_hint, targets_hint, Σ>>],
//     index: usize,
// }

// impl <'a, const symbols_hint: usize, const targets_hint: usize, Σ: Eq + Hash + Copy> Iterator for HomoIter<'a, symbols_hint, targets_hint, Σ> 
// where 
//     [Option<Σ>; symbols_hint]: Array<Item = Option<Σ>>,
//     [NonNull<HomoState<symbols_hint, targets_hint, Σ>>; targets_hint]: Array<Item = NonNull<HomoState<symbols_hint, targets_hint, Σ>>>,
// {
//     type Item = &'a HomoState<symbols_hint, targets_hint, Σ>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.index >= self.transitions_vec.len() {
//             return None;
//         } else {
//             let next = unsafe { self.transitions_vec[self.index].as_ref() };
//             self.index += 1;
//             Some(next)
//         }
//     }
// }

// impl<const symbols_hint: usize, const targets_hint: usize, Σ: Eq + Hash + Copy> HomoState<symbols_hint, targets_hint, Σ> 
// where
//     [Option<Σ>; symbols_hint]: Array<Item = Option<Σ>>,
//     [NonNull<HomoState<symbols_hint, targets_hint, Σ>>; targets_hint]: Array<Item = NonNull<HomoState<symbols_hint, targets_hint, Σ>>>,
// {
//     pub fn new() -> Self {
//         Self {
//             transitions: SmallMap::new(),
//             is_accept: false,
//         }
//     }

//     pub fn push_transition(&mut self, symbol: Option<Σ>, target: Option<&Self>) {
//         let target = (match target {
//             Some(target) => NonNull::new(target as *const Self as *mut Self),
//             None => NonNull::new(self as *const Self as *mut Self),
//         }).unwrap();

//         let vec = match self.transitions.get_mut(&symbol) {
//             Some(existing) => existing,
//             None => {
//                 self.transitions.insert(symbol, SmallVec::new());
//                 self.transitions.get_mut(&symbol).unwrap()
//             }
//         };

//         vec.push(target);
//     }

//     pub fn set_accept(&mut self, is_accept: bool) {
//         self.is_accept = is_accept;
//     }

//     pub fn get_transitions<'a>(&'a self, symbol: Option<Σ>) -> HomoIter<'a, symbols_hint, targets_hint, Σ> {
//         HomoIter {
//             _self: self,
//             transitions_vec: self.transitions.get(&symbol).map(|smallvec| smallvec.as_slice()).unwrap_or(&[]),
//             index: 0,
//         }
//     }
// }

// macro_rules! dynamic_state_creator {
//     (($d: tt), $func_name: ident, $arena: expr, $symbol: ty) => {
//         macro_rules! $func_name {
//             ($symbols_hint: expr, $targets_hint: expr, $d($is_accept:expr $d(,$transitions: expr)?)? ) => {
//                 {
//                     let new_state = $arena.alloc::<DynamicState<$symbols_hint, $targets_hint, $symbol>>(DynamicState::new());
//                     $d(
//                         new_state.set_accept($is_accept);
//                         $d(
//                             let transitions: &[(_, Option<NonNull<DynamicState<$symbols_hint, $targets_hint, $symbol>>)] = $transitions;
//                             transitions.iter().for_each(|transition| new_state.push_transition(*transition));
//                         )?
//                     )?
//                     new_state
//                 }
//             };
//         }
//     };
// }

// macro_rules! homo_state_creator {
//     (($d: tt), $func_name: ident, $arena: expr, $symbol: ty, $symbols_hint: expr, $targets_hint: expr) => {
//         macro_rules! $func_name {
//             ($d($is_accept:expr $d(,$transitions: expr)?)? ) => {
//                 {
//                     let new_state = $arena.alloc(HomoState::<$symbols_hint, $targets_hint, $symbol>::new());
//                     $d(
//                         new_state.set_accept($is_accept);
//                         $d(
//                             let transitions: &[(_, Option<&HomoState::<$symbols_hint, $targets_hint, $symbol>>)] = $transitions;
//                             transitions.iter().for_each(|&(symbol, target)| new_state.push_transition(symbol, target));
//                         )?
//                     )?
//                     new_state
//                 }
//             };
//         }        
//     }
// }

// const NFA_BLOCK_SIZE: usize = 1 << 7;
// pub struct Nfa<T> {
//     arena: Corrida::<NFA_BLOCK_SIZE>,
//     start_node: NonNull<T>
// }

// impl<const symbols_hint: usize, const targets_hint:usize, Σ: Eq + Hash + Copy>  Nfa<HomoState<symbols_hint, targets_hint, Σ>> 
// where 
//     [Option<Σ>; symbols_hint]: Array<Item = Option<Σ>>,
//     [NonNull<HomoState<symbols_hint, targets_hint, Σ>>; targets_hint]: Array<Item = NonNull<HomoState<symbols_hint, targets_hint, Σ>>>,
// {
//     pub fn new(arena: Corrida<NFA_BLOCK_SIZE>, start_node: NonNull<HomoState<symbols_hint, targets_hint, Σ>>) -> Self {
//         Self {
//             arena,
//             start_node
//         }
//     }

//     pub fn simulate_iter(&self, input: impl Iterator<Item = Σ>) -> bool {
//         let mut current_states = vec![unsafe { self.start_node.as_ref() }];
//         let mut next_states = Vec::new();
//         let mut i = 0;
//         while i < current_states.len() {
//             let state = current_states[i];
//             for next in state.get_transitions(None) {
//                 current_states.push(next);
//             }
//             i += 1;
//         }

//         //? In a well formed NFA, i believe that reaching the same state from two different paths is very rare.
//         for symbol in input {
//             for cur in current_states.into_iter() {
//                 for next in cur.get_transitions(Some(symbol)) {
//                     next_states.push(next);
//                 }
//             }

//             let mut i = 0;
//             while i < next_states.len() {
//                 for next in next_states[i].get_transitions(None) {
//                     next_states.push(next);
//                 }
//                 i += 1;
//             }

//             (current_states, next_states) = (next_states, Vec::new());
//         }

//         current_states.into_iter().any(|state| state.is_accept)
//     }

//     pub fn simulate_slice(&self, input: &[Σ]) -> bool {
//         self.simulate_iter(input.iter().copied())
//     }
// }

//MARK: Tests
// #[cfg(test)]
// mod test {
//     #![macro_use]
//     use super::*;

//     use corrida::Corrida;

//     #[test]
//     fn test_homo() {
//         let mut arena = Corrida::<NFA_BLOCK_SIZE>::new();

//         homo_state_creator!(($), new_state, arena, char, 5, 5);

//         let start_node = {
//             let s_0 = new_state!();

//             let s_1 = new_state!(true, &[(Some('1'), None)]);
//             let s_2 = new_state!(false, &[(Some('1'), None)]);
//             s_2.push_transition(Some('0'), Some(s_1));
//             s_1.push_transition(Some('0'), Some(s_2));
//             s_0.push_transition(None, Some(s_1));

//             let s_3 = new_state!(true, &[(Some('0'), None)]);
//             let s_4 = new_state!(false, &[(Some('0'), None)]);
//             s_4.push_transition(Some('1'), Some(s_3));
//             s_3.push_transition(Some('1'), Some(s_4));
//             s_0.push_transition(None, Some(s_3));

//             NonNull::new(s_0).unwrap()
//         };

//         let nfa = Nfa::new(arena, start_node);
//         assert_eq!(nfa.simulate_slice(&['0']),false);
//         assert_eq!(nfa.simulate_slice(&['0','0']),true);
//         assert_eq!(nfa.simulate_slice(&['0','1']),false);
//         assert_eq!(nfa.simulate_slice(&['1','1']),true);
//         assert_eq!(nfa.simulate_slice(&['0','0','0','1','0','1','0']),true);
//     }

// }




