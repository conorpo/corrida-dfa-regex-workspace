//! Regex parser based on a DFA implementation

use gerber::*;
use std::str::Chars;

// So this is option hell.., maybe I needed more recursion? Kind of hard to design this when I can't delete or merge nodes.


fn parse_group<'a,const is_outermost: bool>(mut iter: Chars, starting_node: &'a mut NfaVertex<char>, nfa: &'a Nfa<char>) -> (Chars<'a>, &'a mut NfaVertex<char>, &'a mut NfaVertex<char>) {
    // Keeps track of the beginning of a concat expression, ended (and needed) on | or )
    let concat_start = starting_node;
    // Keeps track of the current pattern (either 1 symbol or a whole group), +, 0, ? need acess to the node at the start of this pattern
    let union_nodes = None;

    let last_pattern_start = &concat_start;
    let last_pattern_end = &concat_start;
    loop {
        if let Some(c) = iter.next() {
            match c {
                ')'  => { // Ends the group expression
                    if is_outermost {
                        panic!("Attempted to end a group with ')' while not in a group, no matching '('");
                    }
                    break;
                }, 
                '+' | '*' | '?' => {
                    // Pattern start is only needed on these operators, decided to avoid options on this one, as an empty pattern can be considered a single node.
                    // This measn checking for an empty pattern by comparing pointers.

                    if (*last_pattern_start as *const NfaVertex<char> == *last_pattern_end  as *const NfaVertex<char> ) {
                        panic!("The '{c}' operator cannot be used on an empty pattern.")
                    }

                    let add_skip = c != '+';
                    let add_repeat = c != '?';

                    if c != '+' {
                        last_pattern_start.append_transitions(&[(None, &[Some(last_pattern_end)])]);
                    }

                    if c != '?' {
                        last_pattern_end.append_transitions(&[(None, &[Some(last_pattern_start)])]);
                    }
                },
                '|' => {
                    let (union_start , union_end) = union_nodes.get_or_insert((
                        nfa.insert_node(false, &[]),
                        nfa.insert_node(false, &[])
                    ));

                    union_start.append_transitions(&[(None, &[Some(concat_start)])]);
                    last_pattern_end.append_transitions(&[(None, &[Some(union_end)])]);

                    //Start a new concatenation
                    concat_start = nfa.insert_node(false, &[]);
                    last_pattern_end = &concat_start;
                    last_pattern_start = &concat_start;
                },
                '(' => {
                    (iter, *last_pattern_start, *last_pattern_end) = parse_group::<false>(iter, *last_pattern_end, nfa);
                },
                c => {
                    // Basic concatenation
                    let new_node = nfa.insert_node(false, &[]);
                    last_pattern_end.append_transitions(&[(Some(c), &[Some(new_node)])]);

                    last_pattern_start = last_pattern_end;
                    last_pattern_end = &new_node;
                }
            }
        } else { // End of regex expression
            if !is_outermost {
                panic!("Not every '(' has a matching ')'");
            }
            break;
        }
    }

    // We treated | like a unary operator, so we need to do union upon ending a group aswell if its a union expression.
    if let Some((union_start, union_end)) = union_nodes {
        union_start.append_transitions(&[(None, &[Some(concat_start)])]);
        last_pattern_end.append_transitions(&[(None, &[Some(union_end)])]);

        (iter, union_start, union_end)
    } else {
        (iter, concat_start, *last_pattern_end)
    }
}

pub fn create_regex_dfa(regex_string: &str) -> Dfa<char> {
    // Easier to create it as an NFA first, then convert using Subset Construction.
    let nfa = Nfa::<char>::new();
    let start_node = nfa.insert_node(false, &[]);
    nfa.set_start_node(start_node);

    let mut iter = regex_string.chars();
    let (_, start_node, end_node) = parse_group::<true>(iter, start_node, &nfa);

    nfa.to_dfa()
}