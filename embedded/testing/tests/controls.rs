// Copyright © 2025 Stephan Kunz
//! Embedded control tests.

#![no_main]
#![no_std]
#![allow(clippy::unwrap_used)]

extern crate alloc;

#[cfg(test)]
#[embedded_test::tests]
mod tests {
	use behaviortree::prelude::*;

	#[test]
	async fn fallback() -> Result<(), Error> {
		// case 1

		Ok(())
	}

	#[test]
	async fn if_then_else() -> Result<(), Error> {
		Ok(())
	}

	#[test]
	async fn parallel_all() -> Result<(), Error> {
		Ok(())
	}

	#[test]
	async fn parallel() -> Result<(), Error> {
		Ok(())
	}

	#[test]
	async fn reactive_fallback() -> Result<(), Error> {
		Ok(())
	}

	#[test]
	async fn reactive_sequence() -> Result<(), Error> {
		Ok(())
	}

	#[test]
	async fn sequence_with_memory() -> Result<(), Error> {
		Ok(())
	}

	#[test]
	async fn sequence() -> Result<(), Error> {
		Ok(())
	}

	#[test]
	async fn switch() -> Result<(), Error> {
		Ok(())
	}

	#[test]
	async fn while_do_else() -> Result<(), Error> {
		Ok(())
	}
}
