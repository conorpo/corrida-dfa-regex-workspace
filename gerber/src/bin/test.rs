use std::time::Instant;

use gerber::nfa::*;
use gerber::nfa_state_creator;


use corrida::*;


pub fn main() {
    let arena = Corrida::new(None);
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
    let mut test = vec![1; 30_000_000];
    test.extend([0,0,0]);

    let start = Instant::now();
    assert!(nfa.simulate_slice_friendly(&test));
    test.push(1);
    assert!(!nfa.simulate_slice_friendly(&test));

    println!("Big Input {:?}", start.elapsed());
}