//! # Tool set for brainfuck language
//! Including:
//! 
//TODO ...

#![warn(missing_docs)]

#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_safety_doc)]
//#![warn(clippy::missing_inline_in_public_items)]

#![warn(rustdoc::private_doc_tests)]


/// bf instruction related definitions:
/// 
/// [`bf`] macro
/// 
/// [`ins::BfIns`] & [`ins::BfCode`] types
pub mod ins;
/// Simple parser for bf instructions:
/// ```
/// # use bf_tools::{ bf, ins::BfCode };
/// let expected = bf!(,[[-]]<);
/// // from string slice
/// let result = ",[[-]]<".parse();
/// assert_eq!(result, Ok(expected));
/// ```
pub mod ins_parser;
