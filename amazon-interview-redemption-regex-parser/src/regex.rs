use std::{iter::Peekable, ptr::NonNull, str::Chars};

use corrida::Corrida;
use gerber::{nfa::*, nfa_state_creator};

// TODO: Figure out epsilon loops
// TODO: Add escape characters
// TODO: Support back references
// TODO: Add . as wildcard character (ps how do we do this without creating a massive UnionState)
/*
    TODO: Maybe
    - Character Classes (encode . as character class)
    - Counted Repetition (how?)
    - Unanchored Matches, can be added if we add .
        - Maybe we can reuse the same final DFA to add functionality for both
    
*/
type RState = State<2, char>;
fn parse_regex(regex_string: &str) -> Nfa<RState> {
    let arena = Corrida::new();
    nfa_state_creator!(($), new_state, arena, char, 2);
    let create_state = |is_final| new_state!(is_final);
    
    fn parse_base<'a>(cur: &'a mut RState, chars: &mut Peekable<Chars>, create_state: &impl Fn(bool) -> &'a mut State<2, char>) -> (&'a mut RState, &'a mut RState) {
        println!("base_start: {}", chars.peek().unwrap_or(&'\0'));
        let (mut base_start, mut base_end) = match chars.next() {
            Some('(') => {
                let (start_node, end_state) = parse_group::<false>(chars, create_state);
                cur.push_transition(None, Some(start_node));
                (cur, end_state)
            },
            Some(c) => {
                if c == '+' || c == '*' || c == '?' {
                    panic!("Got an operator (+, *, ?) when there was no base to skip/repeat");
                }

                let new_state = create_state(false);
                cur.push_transition(Some(c), Some(new_state));

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

        if add_cycle {
            base_end.push_transition(None, Some(&base_start));

            if add_skip {
                let old_start = base_start;
                base_start = create_state(false);
                base_start.push_transition(None, Some(old_start));
                let old_end = base_end;
                base_end = create_state(false);
                old_end.push_transition(None, Some(base_end));
            }
        }
        if add_skip {
            base_start.push_transition(None, Some(&base_end));
        }

        println!("base done");
        (base_start, base_end)        
    }

    fn parse_concat<'a>(chars: &mut Peekable<Chars>,create_state: &impl Fn(bool) -> &'a mut State<2, char>) -> (&'a mut RState, Option<&'a mut RState>) {
        println!("concat_start: {}", chars.peek().unwrap_or(&'\0'));
        let mut cur = create_state(false);
        let mut pattern_start = None;

        while chars.peek() != None && chars.peek() != Some(&')') && chars.peek() != Some(&'|') {
            let (base_start, base_end) = parse_base(cur, chars, create_state);

            if pattern_start.is_none() {
                pattern_start = Some(base_start);
            }

            cur = base_end;
        }

        println!("concat_end: {}", chars.peek().unwrap_or(&'\0'));
        match pattern_start {
            Some(start) => (start, Some(cur)),
            None => (cur, None)
        }
    }

    fn parse_group<'a, const outermost: bool>(chars: &mut Peekable<Chars>, create_state: &impl Fn(bool) -> &'a mut State<2, char>) -> (&'a mut RState, &'a mut RState) {
        println!("group_start: {}", chars.peek().unwrap_or(&'\0'));
        fn add_to_union(union_start: &mut RState, union_end: &mut RState, concat_start: &mut RState, concat_end: Option<&mut RState>) {
            union_start.push_transition(None, Some(concat_start));
            let concat_end = concat_end.unwrap_or(concat_start);
            concat_end.push_transition(None, Some(&union_end));
        }

        let (concat_start, concat_end_opt) = parse_concat(chars, create_state);

        let (group_start, group_end) = if let Some(&'|') = chars.peek() {
            let (union_start, union_end) = (create_state(false), create_state(false));
            add_to_union(union_start, union_end, concat_start, concat_end_opt);
            
            loop {
                chars.next(); // eat '|'
                let (concat_start, concat_end_opt) = parse_concat(chars, create_state);
                add_to_union(union_start, union_end, concat_start, concat_end_opt);
                if chars.peek() != Some(&'|') { break; }
            }

            (union_start, union_end)
        } else {
            let concat_end = concat_end_opt.unwrap_or_else(|| {
                let end = create_state(true);
                concat_start.push_transition(None, Some(end));
                end
            });
            (concat_start, concat_end)
        };

        match chars.next() {
            Some(')') if outermost => {
                panic!("Attempted to close a group in the outermost context ( no matching '(' )");
            },
            None if !outermost => {
                panic!("EOF when not all groups were closed, '(' without matching ')'");
            },
            _ => {}
        }

        println!("group_end: {}", chars.peek().unwrap_or(&'\0'));
        // SAFETY, in both arms the non null comes from an exclusive reference, so all good.
        (group_start, group_end)
    }

    let mut chars = regex_string.chars().peekable();
    let (start_node, end_node) = parse_group::<true>(&mut chars, &create_state);
    end_node.set_accept(true);
    let start_node = NonNull::new(start_node).unwrap();

    Nfa::new(arena, start_node)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_basics() {
        let nfa = parse_regex("ab*(c|)");
        println!("Done Parsing");
        
        //assert_eq!(nfa.simulate_iter("".chars()), false);
        assert_eq!(nfa.simulate_iter("a".chars()), true);
        // assert_eq!(nfa.simulate_iter("ab".chars()), true);
        // assert_eq!(nfa.simulate_iter("ac".chars()), true);
        // assert_eq!(nfa.simulate_iter("abb".chars()), true);
        // assert_eq!(nfa.simulate_iter("abbcc".chars()), false);
        // assert_eq!(nfa.simulate_iter("abbbac".chars()), false);
        // assert_eq!(nfa.simulate_iter("abaa".chars()), false);
        // assert_eq!(nfa.simulate_iter("abbbbbbbc".chars()), true);
    }    
}