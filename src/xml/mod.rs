// Copyright © 2025 Stephan Kunz

//! [`behaviortree`](crate) xml module.

pub mod creator;
pub mod error;
pub mod parser;

#[cfg(test)]
mod tests {
	use crate::{XmlCreator, xml::parser::XmlParser};

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<crate::xml::error::Error>();
		is_normal::<XmlParser>();
		is_normal::<XmlCreator>();
	}
}
