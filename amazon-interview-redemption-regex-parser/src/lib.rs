//! Regex parser based on a DFA implementation

use gerber::*;
use std::{cell::{Cell, OnceCell, Ref, RefCell, UnsafeCell}, collections::HashMap, iter::Peekable, rc::Rc, str::Chars};

// To avoid option hell or having to delete and merge nodes, introduce a shared mutable reference
type NodeAlias<'a> = Rc<RefCell<&'a mut NfaVertex<char>>>;

// const DUMMY_REF: OnceCell::<NfaVertex<char>> = OnceCell::new();
enum Pattern<'a> {
    NonEmpty(&'a mut NfaVertex<char>, &'a mut NfaVertex<char>),
    Empty(&'a mut NfaVertex<char>)
}

// Designing systems is usually a spectrum between state affecting procedures, or procedures emulating state.
// Design decisions leads to certain kind of data flow. Its up to designers to choose what to encode in state and what to encode into procedure, based on the project's goals.
// Similair to recursion vs iteration
struct RegexParser<'a> {
    iter: Peekable<Chars<'a>>,
    nfa: &'a Nfa<char>,
}

impl RegexParser<'_> {
    fn parse_concat<'a>(&mut self, cur_node: NodeAlias<'a>) {
        while let Some(c) = self.iter.next() { match c {
            ')'

            c => {
            
            }
        } };
    }

    fn parse_group<'a,const is_outermost: bool>(&mut self, cur_node: NodeAlias<'a>) -> (Chars<'a>, NodeAlias<'a>, NodeAlias<'a>) {
        let mut union_nodes = None;
        // Keeps track of the beginning of a concat expression, ended (and needed) on | or )
        let mut concat_start = cur_node;
        // Keeps track of the current pattern (either 1 symbol or a whole group), +, 0, ? need acess to the node at the start of this pattern
        let mut last_pattern_start = concat_start.clone();
        let mut last_pattern_end = concat_start.clone();
        // We can add certain flags and integers to keep track of state, then update last_pattern_start, last_pattern_end as we go

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
                        if Rc::ptr_eq(&last_pattern_start, &last_pattern_end) {
                            panic!("The '{c}' operator cannot be used on an empty pattern.")
                        }

                        let add_skip = c != '+';
                        let add_repeat = c != '?';

                        if c != '+' {
                            last_pattern_start.borrow_mut().append_transitions(&[
                                (None, &[Some(*last_pattern_end.borrow())])
                            ]);
                        }

                        if c != '?' {
                            last_pattern_end.borrow_mut().append_transitions(&[
                                (None, &[Some(*last_pattern_start.borrow())])
                            ]);
                        }
                    },
                    '|' => {
                        if union_nodes.is_none() {
                            let start = nfa.insert_node(false, &[]);
                            let end = nfa.insert_node(false, &[]);
                            
                            union_nodes = Some((start, end));
                        };

                        let (union_start , union_end) = union_nodes.as_mut().unwrap();
                        
                        // Hookup the concatenation sequence to our outer union nodes.
                        union_start.append_transitions(&[
                            (None, &[Some(*concat_start.borrow())])
                        ]);
                        last_pattern_end.borrow_mut().append_transitions(&[
                            (None, &[Some(union_end)])
                        ]);

                        //Start a new concatenation
                        concat_start = Rc::new(RefCell::new(nfa.insert_node(false, &[])));
                        last_pattern_start = concat_start.clone();
                        last_pattern_end = concat_start.clone();
                    },
                    '(' => {
                        (iter, last_pattern_start, last_pattern_end) = parse_group::<false>(iter, last_pattern_end, nfa);
                    },
                    c => {
                        // Basic concatenation
                        let new_node = nfa.insert_node(false, &[]);
                        last_pattern_end.borrow_mut().append_transitions(&[
                            (Some(c), &[Some(new_node)]) // How can this leak, and why?
                        ]);

                        last_pattern_start = last_pattern_end;
                        last_pattern_end = Rc::new(RefCell::new(new_node));
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
            // Hookup the concatenation sequence to our outer union nodes.
            union_start.append_transitions(&[
                (None, &[Some(*concat_start.borrow())])
            ]);  

            last_pattern_end.borrow_mut().append_transitions(&[
                (None, &[Some(union_end)])
            ]);

            (iter, Rc::new(RefCell::new(union_start)), Rc::new(RefCell::new(union_end)))
        } else {
            (iter, concat_start, last_pattern_end)
        }
    }

}

pub fn create_regex_dfa(regex_string: &str) -> Nfa<char> {
    // Easier to create it as an NFA first, then convert using Subset Construction.
    let nfa = Nfa::<char>::new();
    let start_node = nfa.insert_node(false, &[]);
    nfa.set_start_node(start_node);

    // Setup our recusrive parser, including setup up a static dummy node, in order to get away with Cell replace shenanigans.
    let mut iter = regex_string.chars();
    let (_, start_node, end_node) = parse_group::<true>(
        iter, 
        Rc::new(RefCell::new(start_node)), 
        &nfa
    );

    nfa
}

#[cfg(test)]
mod test {
    use super::create_regex_dfa;

    #[test]
    pub fn test_basics() {
        let nfa = create_regex_dfa("(0|(1(01*(00)*0)*1)*)*");
    }
}