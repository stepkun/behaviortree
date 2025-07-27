// Copyright © 2025 Stephan Kunz

//! Control behavior library
//!

mod fallback;
mod if_then_else;
mod parallel;
mod parallel_all;
mod reactive_fallback;
mod reactive_sequence;
mod sequence;
mod sequence_with_memory;
mod switch;
mod while_do_else;

// flatten
pub use fallback::Fallback;
pub use if_then_else::IfThenElse;
pub use parallel::Parallel;
pub use parallel_all::ParallelAll;
pub use reactive_fallback::ReactiveFallback;
pub use reactive_sequence::ReactiveSequence;
pub use sequence::Sequence;
pub use sequence_with_memory::SequenceWithMemory;
pub use switch::Switch;
pub use while_do_else::WhileDoElse;
