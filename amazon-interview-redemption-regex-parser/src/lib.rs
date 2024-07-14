//! Regex parser based on a DFA implementation
#![feature(lazy_cell)]

use std::iter::*;
use std::str::Chars;
use std::cell::LazyCell;
use std::collections::HashSet;

use gerber::*;


// Notes:
// All cell provides safe interior mutability with an unsafe implemenation.
// Cells can do this by making sure the value is always either moved or copied. This means they do not provide mutable aliasing for non-copy types, unless you yourself remember to give it back to the cell, when you are done with it.
// This is what RefCell automates, enforcing mutation-xor-aliasing at runtime. Because the lifetime of this mutable access is created at runtime, it only lets you borrow mutably, and this borrow can't outlive the context where it was made.
// This means that even with cells, the lifetime of a value is still tied to the variable containing the cell. This makes it hard to use our variables to represent movement of values. If there was a way to dynamically assign variables to be referencing a


// To avoid option hell or having to delete and merge nodes, introduce a shared mutable reference
type NodeAlias<'a> = &'a mut NfaVertex<char>;

// const DUMMY_REF: OnceCell::<NfaVertex<char>> = OnceCell::new();
enum Pattern<'a> {
    NonEmpty(NodeAlias<'a>, NodeAlias<'a>),
    Empty(NodeAlias<'a>)
}

impl Pattern<'_> {
    
}

// Designing systems is usually a spectrum between state affecting procedures, or procedures emulating state.
// Design decisions leads to certain kind of data flow. Its up to designers to choose what to encode in state and what to encode into procedure, based on the project's goals.
// Similair to recursion vs iteration
const RESERVED_CHARS: LazyCell<HashSet<char>> = LazyCell::new(|| {
    let mut hashset = HashSet::new();
    hashset.extend(&['+','?','*','|','(',')']);
    hashset
});

// Return index out to main constructor for error
struct RegexParser<'a> {
    iter: Peekable<Chars<'a>>,
    nfa: Nfa<char>,
    //dict_set: HashSet<char>,
}

// The references to nodes provided by the NFA are exclusive. They would have to be rapped in some shared mutation if we wanted to keep track of graph state in one context.
// We could also put everything in options, or more generally move the exlusive refernces based on the state. We are going to go for the latter, using recursion to move them around.

// Allow for custom allocator
impl<'a> RegexParser<'a> {
    fn new(str: &'a str) -> Self {
        Self {
            iter: str.chars().peekable(),
            nfa: Nfa::new(),
            //dict_set:
        }
    }

    // pub fn parse(&mut self, input: &str) {
    //     //self.dict_set.reset();
    //     // 
    // }

    /// During all points in this function it has either
    /// 1 node, the empty pattern OR
    /// 2 nodes, with a transition on the symbol. Operators like +, -, ? will add additional epsilon transitions.
    fn parse_symbol(&mut self, cur_node: NodeAlias<'a>) -> Pattern {        
        // There actually wasn't any symbol... 
        if self.iter.peek().is_some_and(|c| RESERVED_CHARS.contains(c)) {
            return Pattern::Empty(cur_node)
        }

        // Eat
        let c = self.iter.next().unwrap();

        // Create node and new transiton
        let symbol_start = cur_node;
        let symbol_end = self.nfa.insert_node(false, &[]);
        symbol_start.append_transitions(&[(Some(c), &[Some(symbol_end)])]);

        let mut add_cycle = false;
        let mut add_skip = false;

        while let Some(c) = self.iter.peek() {
            match c {
                '+' => {
                    add_cycle |= true;
                },
                '?' => {
                    add_skip |= true;
                },
                '*' => {
                    add_skip |= true;
                }
                _ => {
                    //Won't eat
                    break;
                }
            }
            
            // Eat
            self.iter.next();
        }

        if add_cycle {
            symbol_end.append_transitions(&[(None, &[Some(symbol_start)])])
        }

        if add_skip {
            symbol_start.append_transitions(&[(None, &[Some(symbol_end)])]);
        }

        Pattern::NonEmpty(symbol_start, symbol_end)
    }

    fn parse_concat(&mut self, mut cur_node: NodeAlias<'a>)  -> Pattern<'a>{
        //Basically just parse symbols and concatenate them
        //After (, parse group takes over, then parse concat takes over
        //After |, parse concat takes over
        // Immediately after either of these parse symbol takes over,
        // returning the first pattern a or a+ or a*? or a group ()
        // If parse_symbol returns the empty pattern, it means there weren't any characters when we expected any
        // This means that next symbol should end the concatenation | or ) or EOF, it can't be an operator if we just got it bacdk from parse_symbol, and it can't be a char or ( if we got the empty pattern back.
        // This means it MUST end the concatenationrt

        //let mut concat_start = cur_node;
        // First make sure this whole concatenation is not just empty

        let concat_start = match self.parse_symbol(cur_node) {
            Pattern::Empty(same_node) => {
                return Pattern::Empty(same_node);
            },
            Pattern::NonEmpty(symbol_start, symbol_end) => {
                cur_node = symbol_end;
                symbol_start
            }
        };

        // Now that we have a unique reference to the front and back node, simply loop
        while let Pattern::NonEmpty(_, symbol_end) = self.parse_symbol(cur_node) {
            cur_node = symbol_end;
        }

        Pattern::NonEmpty(concat_start, cur_node)
    }

    fn parse_group<const outermost: bool>(&mut self, mut cur_node: NodeAlias<'a>) -> Pattern<'a>{
        //let cur_pattern = Pattern::Empty(cur_node);

        //let union_buffer_nodes: Option::<(NodeAlias, NodeAlias)> = None;

        let mut last_concat = self.parse_concat(cur_node);

        // Should either be | or ) or EOF
        // Can't be an unary operator which wouldve been eaten by parse_symbol
        // Can't be a character which wouldbe been eaten by parse_symbol
        // Can't be a ( which wouldve been eaten by parse symbol
        if self.iter.peek() == Some(&'|') {
            // Its a union expression so we need to setup our buffer nodes.
            let union_buffer_nodes = (
                self.nfa.insert_node(false, &[]),
                self.nfa.insert_node(false, &[])
            );

        
            loop {
                // Hook up the concat pattern to our union nodes
                match last_concat {
                    Pattern::Empty(_) => {
                        // No point inhaving this pattern, just connect up the two union nodes
                        union_buffer_nodes.0.append_transitions(&[
                            (None, &[Some(union_buffer_nodes.1)])
                        ]);
                    },
                    Pattern::NonEmpty(concat_start, concat_end) => {
                        union_buffer_nodes.0.append_transitions(&[
                            (None, &[Some(concat_start)])  
                        ]);
                        concat_end.append_transitions(&[
                            (None, &[Some(union_buffer_nodes.1)])  
                        ]);
                    }
                }

                if self.iter.peek() != Some(&'|') {
                    //Union expression over
                    break;
                }
                self.iter.next(); // Must be a '|'
                
                // Starting a new concat expression between our union nodes, so give it a node to start off with
                last_concat = self.parse_concat(
                    self.nfa.insert_node(false, &[])
                );
            }
        }

        match (outermost, self.iter.next()) {
            (true, None) | (false, Some(')')) => {
                return last_concat;
            },
            (true, Some(')')) => {
                panic!("')' has no corresponding '(', there was no group to end.")
            },
            (false, None) => {
                panic!("Not every '(' has a corresponding ')', a group was never closed.")
            },
            _ => {
                //Note, need to refactor this a bit to be invariant of if we have a union buffer ot not
                panic!("Shouldn't have gotten her?")
            }
        }
    }

}

pub fn create_regex_dfa(regex_string: &str) -> Nfa<char> {
    // Easier to create it as an NFA first, then convert using Subset Construction.
    let nfa = Nfa::<char>::new();
    let start_node = nfa.alloc_node(false, &[]);
    nfa.set_start_node(start_node);

    // Setup our recusrive parser, including setup up a static dummy node, in order to get away with Cell replace shenanigans.
    let _iter = regex_string.chars();
    // let (_, start_node, end_node) = parse_group::<true>(
    //     iter, 
    //     Pin::new(start_node), 
    //     &nfa
    // );

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