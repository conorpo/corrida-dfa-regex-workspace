#![warn(missing_docs)]

//! A simple DFA library to construct state machines, fast allocation using an a custom Arena implementation, and safe construction using Rust's borrow checker.

pub mod dfa;
pub mod nfa;