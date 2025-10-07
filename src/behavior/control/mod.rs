// Copyright © 2025 Stephan Kunz
//! Built-in `Control` behaviors of [`behaviortree`](crate).

#[cfg(feature = "fallbacks")]
mod fallbacks;
#[cfg(feature = "if_then_else")]
mod if_then_else;
#[cfg(feature = "parallel")]
mod parallel;
#[cfg(feature = "parallel_all")]
mod parallel_all;
#[cfg(feature = "reactive_fallback")]
mod reactive_fallback;
#[cfg(feature = "reactive_sequence")]
mod reactive_sequence;
#[cfg(feature = "sequence_with_memory")]
mod sequence_with_memory;
#[cfg(feature = "sequences")]
mod sequences;
#[cfg(feature = "switch")]
mod switch;
#[cfg(feature = "while_do_else")]
mod while_do_else;

// flatten
#[cfg(feature = "fallbacks")]
pub use fallbacks::Fallback;
#[cfg(feature = "if_then_else")]
pub use if_then_else::IfThenElse;
#[cfg(feature = "parallel")]
pub use parallel::Parallel;
#[cfg(feature = "parallel_all")]
pub use parallel_all::ParallelAll;
#[cfg(feature = "reactive_fallback")]
pub use reactive_fallback::ReactiveFallback;
#[cfg(feature = "reactive_sequence")]
pub use reactive_sequence::ReactiveSequence;
#[cfg(feature = "sequence_with_memory")]
pub use sequence_with_memory::SequenceWithMemory;
#[cfg(feature = "sequences")]
pub use sequences::Sequence;
#[cfg(feature = "switch")]
pub use switch::Switch;
#[cfg(feature = "while_do_else")]
pub use while_do_else::WhileDoElse;
