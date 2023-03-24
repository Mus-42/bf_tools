//! # Tool set for brainfuck language
//! Including:
//!
//! Parser
//! Optimizer
//!
//TODO ...

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_safety_doc)]
//#![warn(clippy::missing_inline_in_public_items)]
#![warn(rustdoc::private_doc_tests)]

/// BF instruction related definitions
pub mod ins;
/// Parser for bf instructions
/// ```
/// # use bf_tools::{ bf, ins::BfCode };
/// // parse bf from string slice
/// let result = ",[[-]]<".parse();
/// let expected = bf!(,[[-]]<);
/// assert_eq!(result, Ok(expected));
/// ```
pub mod ins_parser;

/// Optimization passes collection
/// ```
/// # use bf_tools::{ bf, ins::BfCode, optimizer::OptState };
/// let code = bf!(+-[-]<>[-]);
/// let code = BfCode::from(
///     OptState::default()
///     .run_passes(code.into())
/// );
/// assert_eq!(code, bf!([-]));
/// ```
pub mod optimizer;

/// Interpreter for BF code
pub mod interpreter;
