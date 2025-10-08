// Copyright © 2025 Stephan Kunz
//! JSON configuration for mocking & behavior replacementin [`BehaviorTree`]s.

#[doc(hidden)]
#[cfg(feature = "std")]
extern crate std;

// region:      --- modules
use crate::{BehaviorState, ConstString, behavior::MockBehaviorConfig, factory::registry::SubstitutionRule};
use alloc::{collections::btree_map::BTreeMap, string::String};
use core::{str::FromStr, time::Duration};
use nanoserde::{DeJson, DeJsonTok};
// endregion:   --- modules

// region:      --- JsonConfig
#[derive(Debug, Default)]
pub struct JsonConfig {
	pub substitution_rules: BTreeMap<ConstString, SubstitutionRule>,
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::single_match_else)]
impl DeJson for JsonConfig {
	fn de_json(state: &mut nanoserde::DeJsonState, input: &mut core::str::Chars) -> Result<Self, nanoserde::DeJsonErr> {
		let mut result = Self::default();
		let mut test_behavior_configs: BTreeMap<String, MockBehaviorConfig> = BTreeMap::new();

		match state.tok {
			DeJsonTok::CurlyOpen => {
				while state.tok != DeJsonTok::CurlyClose {
					state.next_tok(input)?;
					match state.tok {
						DeJsonTok::Str => match state.strbuf.as_ref() {
							"MockBehaviorConfigs" => {
								// next must be an CurlyOpen
								while state.tok != DeJsonTok::CurlyOpen && state.tok != DeJsonTok::Eof {
									state.next_tok(input)?;
								}
								// consume that CurlyOpen
								state.next_tok(input)?;
								// read the configs
								while state.tok != DeJsonTok::CurlyClose && state.tok != DeJsonTok::Eof {
									// remember the configs name
									let config_name = state.strbuf.clone();
									let mut config = MockBehaviorConfig::default();
									// next must be an OpenCurly again
									while state.tok != DeJsonTok::CurlyOpen {
										state.next_tok(input)?;
									}
									// consume that CurlyOpen
									state.next_tok(input)?;
									// now we have the data entries
									// read the data entries for the config
									while state.tok != DeJsonTok::CurlyClose && state.tok != DeJsonTok::Eof {
										// match the data entries for the config
										let field = state.strbuf.clone();
										// consume colon
										state.next_tok(input)?;
										// field content
										state.next_tok(input)?;
										// std::dbg!(&field);
										match field.as_ref() {
											"return_status" => {
												let behavior_state =
													BehaviorState::from_str(&state.strbuf).map_err(|_x| {
														nanoserde::DeJsonErr {
															line: state.line,
															col: state.col,
															msg: nanoserde::DeJsonErrReason::CannotParse(
																state.strbuf.clone(),
															),
														}
													})?;
												config.return_state = behavior_state;
												// skip behind content
												state.next_tok(input)?;
											}
											"async_delay" => {
												let value: u64 =
													u64::from_str(&state.numbuf).map_err(|_x| nanoserde::DeJsonErr {
														line: state.line,
														col: state.col,
														msg: nanoserde::DeJsonErrReason::CannotParse(state.strbuf.clone()),
													})?;
												config.async_delay = Some(Duration::from_millis(value));
											}
											"failure_script" => {
												config.failure_script = Some((*state.strbuf).into());
												// skip behind content
												state.next_tok(input)?;
											}
											"success_script" => {
												config.success_script = Some((*state.strbuf).into());
												// skip behind content
												state.next_tok(input)?;
											}
											"post_script" => {
												config.post_script = Some((*state.strbuf).into());
												// skip behind content
												state.next_tok(input)?;
											}
											_ => {
												return Err(nanoserde::DeJsonErr {
													line: state.line,
													col: state.col,
													msg: nanoserde::DeJsonErrReason::UnexpectedToken(
														state.tok.clone(),
														state.strbuf.clone(),
													),
												});
											}
										}
										// forward to next relevant entry
										while state.tok != DeJsonTok::Str
											&& state.tok != DeJsonTok::CurlyClose
											&& state.tok != DeJsonTok::Eof
										{
											state.next_tok(input)?;
										}
										if state.tok == DeJsonTok::CurlyClose {
											test_behavior_configs.insert(config_name, config);
											// consume CurlyClose
											state.next_tok(input)?;
											break;
										}
									}
									// forward to next relevant entry
									while state.tok != DeJsonTok::Str
										&& state.tok != DeJsonTok::CurlyClose
										&& state.tok != DeJsonTok::Eof
									{
										state.next_tok(input)?;
									}
									if state.tok == DeJsonTok::CurlyClose {
										// consume CurlyClose
										state.next_tok(input)?;
										break;
									}
								}
							}
							"SubstitutionRules" => {
								// next must be an CurlyOpen
								while state.tok != DeJsonTok::CurlyOpen && state.tok != DeJsonTok::Eof {
									state.next_tok(input)?;
								}
								// consume that CurlyOpen
								state.next_tok(input)?;
								while state.tok != DeJsonTok::CurlyClose && state.tok != DeJsonTok::Eof {
									let key = state.strbuf.clone();
									// consume token
									state.next_tok(input)?;
									// forward to next relevant entry
									while state.tok != DeJsonTok::Str
										&& state.tok != DeJsonTok::CurlyClose
										&& state.tok != DeJsonTok::Eof
									{
										state.next_tok(input)?;
									}
									let value = state.strbuf.clone();
									let rule = test_behavior_configs.get(&value).map_or_else(
										|| SubstitutionRule::StringRule(value.into()),
										|config| SubstitutionRule::ConfigRule(config.clone()),
									);
									// std::dbg!(&rule);
									result.substitution_rules.insert(key.into(), rule);
									// consume token
									state.next_tok(input)?;
									while state.tok != DeJsonTok::Str
										&& state.tok != DeJsonTok::CurlyClose
										&& state.tok != DeJsonTok::Eof
									{
										state.next_tok(input)?;
									}
									if state.tok == DeJsonTok::CurlyClose {
										// consume CurlyClose
										state.next_tok(input)?;
										break;
									}
								}
							}
							_ => {
								return Err(nanoserde::DeJsonErr {
									line: state.line,
									col: state.col,
									msg: nanoserde::DeJsonErrReason::UnexpectedToken(
										state.tok.clone(),
										state.strbuf.clone(),
									),
								});
							}
						},
						_ => {
							return Err(nanoserde::DeJsonErr {
								line: state.line,
								col: state.col,
								msg: nanoserde::DeJsonErrReason::UnexpectedToken(state.tok.clone(), state.strbuf.clone()),
							});
						}
					}
				}
			}
			_ => {
				return Err(nanoserde::DeJsonErr {
					line: state.line,
					col: state.col,
					msg: nanoserde::DeJsonErrReason::UnexpectedToken(state.tok.clone(), state.strbuf.clone()),
				});
			}
		}
		// std::dbg!(&test_behavior_configs);
		// std::dbg!(&result);
		Ok(result)
	}
}
// endregion:   --- JsonConfig
