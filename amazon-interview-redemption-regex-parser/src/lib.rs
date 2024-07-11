//! Regex parser based on a DFA implementation

use gerber::*;

enum RegexToken {
    Union,
    Symbol,
    Group,
}

// fn map_

pub fn create_regex_dfa(regex_string: &str) -> Dfa<char> {
    // Easier to create it as an NFA first, then convert using Subset Construction.
    let nfa = Nfa::<char>::new();
    //let mut group_stack = vec![];
    let mut last_token = RegexToken::Symbol;
    
    let mut cur_group_end = nfa.insert_vertex(false, &[]);
    let mut cur_group_start = nfa.insert_vertex(false, &[]);

    let mut last_group_complete = false;

    nfa.set_start_node(cur_group_start);
    
    for c in regex_string.chars() {
        match c {
            '(' => {
                
            },
            ')' => {
            
            },
            '+' | '*' => {
                let add_skip_transtion = c == '*';

                match last_token {
                    RegexToken::Symbol | RegexToken::Group if last_group_complete => {
                        if add_skip_transtion {
                            cur_group_start.append_transitions(&[(None, &[Some(cur_group_end)])]);
                        }
                        cur_group_end.append_transitions(&[(None, &[Some(cur_group_start)])]);
                    },
                    _ => {
                        panic!("Union expressions must be in their own () group if you want to use the * or + operators on them.");
                    }
                }
            },
            '|' => {
                last_token = RegexToken::Union;
                last_group_complete = false;
            },
            c => {
                if let RegexToken::Union = last_token {
                    cur_group_start.append_transitions(&[(Some(c), &[Some(cur_group_end)])]);
                } else {
                    let new_end = nfa.insert_vertex(false, &[]);
                    cur_group_end.append_transitions(&[(Some(c), &[Some(new_end)])]);

                    cur_group_start = cur_group_end;
                    cur_group_end = new_end;

                    last_group_complete = true;
                }

                last_token = RegexToken::SymbolOrGroup;
            }
        }
    }

    todo!();
}