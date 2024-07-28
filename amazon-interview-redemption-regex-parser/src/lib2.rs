use std::{cell::LazyCell, collections::HashSet, iter::Peekable, ptr::NonNull, str::Chars};

use smallvec::SmallVec;
use corrida::Corrida;
use gerber::Dfa;

// TODO: Add escape characters
// TODO: Support back references
// TODO: Add . as wildcard character (ps how do we do this without creating a massive UnionState)
// maybe we have store patterns in the union state, rather than explicit chars
/*
    TODO: Maybe
    - Character Classes (encode . as character class)
    - Counted Repetition (how?)
    - Unanchored Matches, can be added if we add .
        - Maybe we can reuse the same final DFA to add functionality for both
    
*/


const STATE_BLOCK_SIZE_BYTES: usize = 2048;
const UNION_STATE_SMALLVEC_SIZE: usize = 4;

const RESERVED_CHARS: LazyCell<HashSet<char>> = LazyCell::new(|| {
    let mut hashset = HashSet::new();
    hashset.extend(&['+','?','*','|','(',')']);
    hashset
});

enum StateLink {
    Simple(NonNull<SimpleState>),
    Union(NonNull<UnionState>)
}

type Transition = (Option<char>, StateLink);

struct SimpleState {
    main_transition: Option<Transition>,
    extra_epsilon: Option<NonNull<SimpleState>> //This one will always go to a simple state
}

// TODO: Try without SmallVec
struct UnionState {
    transitions: SmallVec<[Transition; UNION_STATE_SMALLVEC_SIZE]>
}

// TODO: Maybe add transiton function to state so that we can avoid the extra simple state on each group
trait State {
    fn make_transition(&mut self, on: Option<char>) -> Transition;
    fn new() -> Self;
}

impl State for SimpleState {
    fn new() -> Self {
        Self {
            main_transition: None,
            extra_epsilon: None
        }
    }

    fn make_transition(&mut self, on: Option<char>) -> Transition {
        //SAFETY, we are getting the pointer from an exclusive reference
        unsafe {
            (
                on,
                StateLink::Simple(NonNull::new_unchecked(self as *mut Self))
            )
        }
    }
}

impl State for UnionState {
    fn new() -> Self {
        Self {
            transitions: SmallVec::new()
        }
    }


    fn make_transition(&mut self, on: Option<char>) -> Transition {
        //SAFETY, we are getting the pointer from an exclusive reference
        unsafe {
            (
                on,
                StateLink::Union(NonNull::new_unchecked(self as *mut Self))
            )
        }
    }
}

struct RegexParser {
    arena: Corrida::<STATE_BLOCK_SIZE_BYTES>,
}

impl<'a> RegexParser {
    fn parse_base(&'a self, cur: &'a mut SimpleState, chars: &mut Peekable<Chars>) -> (&'a mut SimpleState, &'a mut SimpleState) {
        let (base_start, base_end) = match chars.next() {
            Some('(') => {
                let (start_link, end_state) = self.parse_group::<false>(chars);

                cur.main_transition = Some((None, start_link));

                (cur, end_state)
            },
            Some(c) => {
                if c == '+' || c == '*' || c == '?' {
                    panic!("Got an operator (+, *, ?) when there was no base to skip/repeat");
                }

                debug_assert!(!RESERVED_CHARS.contains(&c));

                let new_state = self.arena.alloc(SimpleState::new());
                
                cur.main_transition = Some(new_state.make_transition(Some(c)));

                (cur, new_state)
            },
            None => {
                panic!("How did we get here.")
            }
        };

        // Should be on operators, if not end base
        let mut add_skip = false;
        let mut add_cycle = false;

        while let Some(c) = chars.peek() {
            match c {
                '+' => {
                    add_cycle = true;
                },
                '*' => {
                    add_cycle = true;
                    add_skip = true;
                },
                '?' => {
                    add_skip = true;
                },
                _ => {
                    break;
                }
            }

            chars.next(); //eat
        }

        // SAFETY: Both base_start and base_end are exclusive references, valid non nulls
        unsafe {
            if add_skip {
                base_start.extra_epsilon = Some(NonNull::new_unchecked(base_end as *mut SimpleState));
            }
            if add_cycle {
                base_end.extra_epsilon = Some(NonNull::new_unchecked(base_start as *mut SimpleState));
            }
        }

        (base_start, base_end)        
    }

    fn parse_concat(&'a self, chars: &mut Peekable<Chars>) -> (&'a mut SimpleState, Option<&'a mut SimpleState>) {
        let mut cur = self.arena.alloc(SimpleState::new());
        let mut pattern_start = None;

        while let Some(&c) = chars.peek() {
            match c {
                ')' | '|' => {
                    break;
                },
                _ => {
                    let (base_start, base_end) = self.parse_base(cur, chars);

                    if pattern_start.is_none() {
                        pattern_start = Some(base_start);
                    }

                    cur = base_end;
                }
            }
        }

        match pattern_start {
            Some(start) => (start, Some(cur)),
            None => (cur, None)
        }
    }

    fn parse_group<const outermost: bool>(&'a self, chars: &mut Peekable<Chars>) -> (StateLink, &'a mut SimpleState) {
        let mut union_buffer: Option<(&mut UnionState, &mut SimpleState)> = None;

        fn add_to_union(union_start: &mut UnionState, union_end: &mut SimpleState, concat_start: &mut SimpleState, concat_end: Option<&mut SimpleState>) {
            union_start.transitions.push(concat_start.make_transition(None));

            let concat_end = concat_end.unwrap_or(concat_start);

            concat_end.main_transition = Some(union_end.make_transition(None));
        }

        let (mut concat_start, mut concat_end);

        loop {
            (concat_start, concat_end) = self.parse_concat(chars);
            
            if chars.peek() == Some(&'|') {
                let (union_start, union_end) = union_buffer.get_or_insert(
                    (
                        self.arena.alloc(UnionState::new()),
                        self.arena.alloc(SimpleState::new()),
                    )
                );

                add_to_union(union_start, union_end, concat_start, concat_end);

                chars.next();
            } else {
                break;
            }
        }

        match chars.next() {
            Some(')') if outermost => {
                panic!("Attempted to close a group in the outermost context ( no matching '(' )");
            },
            None if !outermost => {
                panic!("EOF when not all groups were closed, '(' without matching ')'");
            },
            _ => {}
        }
        
        // SAFETY, in both arms the non null comes from an exclusive reference, so all good.
        unsafe {
            if let Some((union_start, union_end)) = union_buffer {
                add_to_union(union_start, union_end, concat_start, concat_end);
    
                (
                    StateLink::Union(NonNull::new_unchecked(union_start as *mut UnionState)),
                    union_end
                )
            } else {
                (
                    StateLink::Simple(NonNull::new_unchecked(concat_start as *mut SimpleState)),
                    concat_end.unwrap_or(concat_start)
                )
            }
        }
    }

    fn make_into_dfa(&self, start_link: StateLink, end: &mut SimpleState) -> Dfa<char> {
        let dfa = Dfa::<char>::new();

        todo!();
    }
}

pub fn create_regex_dfa(regex_string: String) -> Dfa<char> {
    // Easier to create it as an NFA first, then convert using Subset Construction.
    let regex_parser = RegexParser {
        arena: Corrida::<STATE_BLOCK_SIZE_BYTES>::new(),
    };

    let mut chars = regex_string.chars().peekable();

    let (start_link, end) = regex_parser.parse_group::<true>(&mut chars);

    regex_parser.make_into_dfa(start_link, end)
}
