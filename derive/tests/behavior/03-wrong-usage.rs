// Copyright © 2025 Stephan Kunz

//! Test wrong usage of behavior derive macro `Behavior` 

#[doc(hidden)]
extern crate alloc;

#[derive(behaviortree_derive::Behavior)]
struct TestBehavior;


// dummy main
fn main(){}
