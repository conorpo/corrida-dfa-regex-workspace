use std::hash::Hash;
use std::marker::PhantomData;
use std::ptr::NonNull;

use smallmap::Map;

/// A node in the DFA, contains is_accept and a transition hashmap. MARK: State
pub trait State<Σ:Eq + Hash + Copy> {
    /// The type of the index used to access transitions in the hashmap.
    type Index;

    /// Returns the next state given the provided symbol, if it exists.
    fn get_transition(&self, symbol: Self::Index) -> Option<&Self>;
    /// Adds a transition to the hashmap. None represents a self-transition.
    fn add_transition(&mut self, transition: (Σ, Option<&Self>));
    /// Sets the accept state flag.
    fn set_accept(&mut self, accept: bool);
    /// Returns the accept state flag.
    fn is_accept(&self) -> bool;
}

/// A node in the DFA, this version uses a hashmap is intended to be used when constructing a partial DFA.
pub struct PartialState<Σ: Eq + Hash + Copy> {
    transitions: Map<Σ, NonNull<PartialState<Σ>>>,
    is_accept: bool
}


impl<Σ:Eq + Hash + Copy> PartialState<Σ> {
    /// Creates a new PartialState with an empty hashmap and is_accept set to false.
    pub fn new() -> Self {
        Self {
            transitions: Map::new(),
            is_accept: false
        }
    }
}


impl<Σ:Eq + Hash + Copy> State<Σ> for PartialState<Σ> 
{
    type Index = Σ;

    fn get_transition(&self, symbol: Self::Index) -> Option<&PartialState<Σ>> {
        self.transitions.get(&symbol).map(|non_null_ref| {
            // Safety, ptr dereference is coming directly from a reference to a PartialState<Σ>
            unsafe { &*(*non_null_ref).as_ptr() }
        })
    }
    
    /// Inserts the provided transitions into this vertex'es hashmap. None represents a self-transition.
    fn add_transition(&mut self, transition: (Σ, Option<&PartialState<Σ>>)){  
        let vert_ref = transition.1.unwrap_or(self) as *const PartialState<Σ> as *mut PartialState<Σ>;
        self.transitions.insert(transition.0, NonNull::new(vert_ref).unwrap());
    }

    fn set_accept(&mut self, accept: bool) {
        self.is_accept = accept;
    }

    fn is_accept(&self) -> bool {
        self.is_accept
    }
}

/// A marker trait for symbol types which are easily indexable.
pub trait Indexable {
    /// Returns the index of the symbol.
    fn get_index(&self) -> usize;
    /// Returns the number of possible symbols.
    fn count() -> usize;
}

/// A node in the DFA, this version uses a vector and is intended to be used when constructing a complete DFA.
pub struct CompleteState<Σ: Eq + Hash + Copy + Indexable> {
    transitions: Vec<Option<NonNull<CompleteState<Σ>>>>,
    is_accept: bool,
    _boo: PhantomData<Σ>
}

impl<Σ:Eq + Hash + Copy + Indexable> CompleteState<Σ> {
    /// Creates a new CompleteState with an empty vector and is_accept set to false.
    pub fn new() -> Self {
        Self {
            transitions: vec![None; Σ::count()],
            is_accept: false,
            _boo: PhantomData
        }
    }
}

impl<Σ:Eq + Hash + Copy + Indexable> State<Σ> for CompleteState<Σ> {
    type Index = usize;

    fn get_transition(&self, index: Self::Index) -> Option<&CompleteState<Σ>> {
        self.transitions[index].map(|non_null_ref| {
            // Safety, ptr dereference is coming directly from a reference to a PartialState<Σ>
            unsafe { &*non_null_ref.as_ptr() }
        })
    }
    
    /// Inserts the provided transitions into this vertex'es hashmap. None represents a self-transition.
    fn add_transition(&mut self, transition: (Σ, Option<&CompleteState<Σ>>)){  
        let vert_ref = transition.1.unwrap_or(self) as *const CompleteState<Σ> as *mut CompleteState<Σ>;
        self.transitions[transition.0.get_index()] = Some(NonNull::new(vert_ref).unwrap());
    }

    fn set_accept(&mut self, accept: bool) {
        self.is_accept = accept;
    }

    fn is_accept(&self) -> bool {
        self.is_accept
    }
}


// MARK: DFA
/// Provides an API for construction and simulation of a DFA structure. 
/// Symbol type Σ must be hashable and implement display (not asking for alot here..)
/// 
pub struct Dfa<'a, Σ: Eq + Hash + Copy, S: State<Σ>> {
    start_node: &'a S,
    _boo: PhantomData<Σ>
}

impl<'a, Σ:Eq + Hash + Copy> Dfa<'a, Σ, PartialState<Σ>>{
    /// Creates a new DFA with no vertices, but an Arena ready for pushing verts.
    pub fn new(start_node: &'a PartialState<Σ>) -> Self {
        Self {
            start_node,
            _boo: PhantomData
        }
    }


    /// Tests the provided input sequence, returning true if the DFA ends at an accept state.
    pub fn simulate_slice(&self, input: &[Σ]) -> bool {
        self.simulate_iter(input.iter().copied())
    }
    
    /// Tests the provided input sequence on an iterator, returning true if the DFA ends at an accept state.
    pub fn simulate_iter(&self, input: impl Iterator<Item = Σ>) -> bool {
        let mut cur: &PartialState<Σ> = self.start_node;
        for symbol in input {
            if let Some(next) = cur.get_transition(symbol) {
                cur = next;
            } else {
                return false;
            }
        }
        cur.is_accept()
    }
}

impl<'a, Σ:Eq + Hash + Copy + Indexable> Dfa<'a, Σ, CompleteState<Σ>>{
    /// Creates a new DFA with no vertices, but an Arena ready for pushing verts.
    pub fn new(start_node: &'a CompleteState<Σ>) -> Self {
        Self {
            start_node,
            _boo: PhantomData
        }
    }

    /// Tests the provided input sequence, returning true if the DFA ends at an accept state.
    pub fn simulate_iter(&self, input: impl Iterator<Item = Σ>) -> bool {
        let mut cur: &CompleteState<Σ> = self.start_node;
        for symbol in input {
            if let Some(next) = cur.get_transition(symbol.get_index()) {
                cur = next;
            } else {
                // Transition did not exist, DFA error
                panic!("Transition not provided from current node on the symbol.")
            }
        }
        cur.is_accept()
    }

    /// Tests the provided input sequence, returning true if the DFA ends at an accept state.
    pub fn simulate_slice(&self, input: &[Σ]) -> bool {
        self.simulate_iter(input.iter().copied())
    }
}

/// A macro for which allows you to make a state creator function for a given state type.
#[macro_export]
macro_rules! dfa_state_creator {
    (($d:tt), $func_name: ident, $arena: expr, $state_type: ty) => {
        macro_rules! $func_name {
            ($d($is_accept:expr $d(,$transitions: expr)?)? ) => {
                {
                    let new_state = $arena.alloc(<$state_type>::new());
                    $d(
                        new_state.set_accept($is_accept);
                        $d(
                            let transitions: &[(_, Option<&$state_type>)] = $transitions;
                            transitions.iter().for_each(|transition| new_state.add_transition(*transition));
                        )?
                    )?
                    new_state
                }
            };
        }
    };
}

// MARK: Tests
#[cfg(test)]
mod test {
    #![macro_use]
    use super::*;
    use corrida::Corrida;

    #[test]
    pub fn test_basics() {
        let arena = Corrida::new(None);

        dfa_state_creator!(($), new_state, arena, PartialState<char>);
        let start_node = {
            let s_0 = new_state!(true);
            let s_1 = new_state!();
            s_0.add_transition(('0', None));
            s_0.add_transition(('1', Some(s_1)));
            let s_2 = new_state!(false, &[('0', Some(s_1)), ('1', None)]);
            s_1.add_transition(('1', Some(s_0)));
            s_1.add_transition(('0',Some(s_2)));

            unsafe{
                let cur = (*s_0.transitions.get_mut(&'1').unwrap()).as_mut();
                let cur = (*cur.transitions.get_mut(&'0').unwrap()).as_mut();
                let cur = (*cur.transitions.get_mut(&'0').unwrap()).as_mut();
                let cur = (*cur.transitions.get_mut(&'1').unwrap()).as_mut();
    
                assert_eq!(cur as *const PartialState<char>, s_0 as *const PartialState<char>);
            }
            s_0
        };

        let dfa = Dfa::<char, PartialState<char>>::new(start_node);
        assert!(dfa.simulate_slice(&"1001".chars().collect::<Vec<char>>()));
        assert!(!dfa.simulate_slice(&"1000".chars().collect::<Vec<char>>()));
    }

    #[test]
    fn test_big_string() {


        let arena = Corrida::new(None);

        dfa_state_creator!(($), new_state, arena, PartialState<char>);

        let start_node = {
            let s_0 = new_state!(true);
            let s_1 = new_state!();
            s_0.add_transition(('0', None));
            s_0.add_transition(('1',Some(s_1)));
            let s_2 = new_state!(false, &[('0', Some(s_1)), ('1', None)]);
            s_1.add_transition(('1', Some(s_0)));
            s_1.add_transition(('0', Some(s_2)));
            
            s_0
        };

        let dfa = Dfa::<char, PartialState<char>>::new(start_node);

        let mut test_vec = Vec::new();

        for _ in 0..10_000_000 {
            test_vec.push('1');
            test_vec.push('0');
            test_vec.push('0');
            test_vec.push('1');
        }
        let start = std::time::Instant::now();
        assert!(dfa.simulate_slice(&test_vec));

        println!("Partial: {:?}", start.elapsed());
    }

    #[derive(Copy, Clone, Eq, PartialEq, Hash)]
    struct Binary {
        index: usize
    }

    use super::Indexable;

    impl Indexable for Binary {
        fn get_index(&self) -> usize {
            self.index
        }

        fn count() -> usize {
            2
        }
    }

    #[test]
    fn test_big_string_complete() {
        let arena = Corrida::new(None);
        const ONE:Binary = Binary { index: 1 };
        const ZERO:Binary = Binary { index: 0 };

        
        dfa_state_creator!(($), new_state, arena, CompleteState<Binary>);
        
        let start_node = {
            let s_0 = new_state!(true);
            let s_1 = new_state!();
            s_0.add_transition((ZERO, None));
            s_0.add_transition((ONE,Some(s_1)));
            let s_2 = new_state!(false, &[(ZERO, Some(s_1)), (ONE, None)]);
            s_1.add_transition((ONE, Some(s_0)));
            s_1.add_transition((ZERO,Some(s_2)));
            
            s_0
        };

        let dfa = Dfa::<Binary, CompleteState<Binary>>::new(start_node);

        let mut test_vec = Vec::new();
        
        for _ in 0..10_000_000 {
            test_vec.push(ONE);
            test_vec.push(ZERO);
            test_vec.push(ZERO);
            test_vec.push(ONE);
        }
        let start = std::time::Instant::now();
        
        assert!(dfa.simulate_slice(&test_vec));
        
        println!("Compelte: {:?}", start.elapsed());
    }

    #[test]
    #[should_panic]
    fn test_dfa_missing_transition() {
        let arena = Corrida::new(None);

        dfa_state_creator!(($), new_state, arena, CompleteState<Binary>);
        let one = Binary { index: 1 };
        let zero = Binary { index: 0 };

        let start_node = {
            let s_0 = new_state!(true);
            let s_1 = new_state!();
            s_0.add_transition((zero, None));
            s_0.add_transition((one,Some(s_1)));
            let s_2 = new_state!(false, &[(zero, Some(s_1)), (one, None)]);
            s_2.add_transition((zero, Some(s_1)));
            s_2.add_transition((one, None));
            s_1.add_transition((one, Some(s_0))); //s_1 has no transition on the zero symbol.

            s_0
        };

        let dfa = Dfa::<Binary, CompleteState<Binary>>::new(start_node);

        dfa.simulate_slice(&[one,zero,zero,one]);
    }

    #[test]
    fn test_dfa_repeated_transitions() {
        let arena = Corrida::new(None);

        let start_node = {
            let s_0 = arena.alloc(PartialState::new());
            s_0.set_accept(true);
            let s_1 = arena.alloc(PartialState::new());
            s_0.add_transition(('0', None));
            s_0.add_transition(('1',Some(s_1)));
            let s_2 = arena.alloc(PartialState::new());
            s_2.add_transition(('0', Some(s_1)));
            s_2.add_transition(('1', None));
            s_1.add_transition(('1', Some(s_0)));
            s_1.add_transition(('0',Some(s_2)));
            
            s_0
        };

        let dfa = Dfa::<char, PartialState<char>>::new(start_node);

        assert!(dfa.simulate_iter(vec!['1','0','0','1'].into_iter()));

    }
}