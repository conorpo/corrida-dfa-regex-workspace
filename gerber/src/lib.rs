#![warn(missing_docs)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

//! A simple DFA library to construct state machines, fast allocation using an a custom Arena implementation, and safe construction using Rust's borrow checker.

/// The DFA module contains the implementation of the Deterministic Finite Automaton.
pub mod dfa;
/// The NFA module contains the implementation of the Non-Deterministic Finite Automaton.
pub mod nfa;