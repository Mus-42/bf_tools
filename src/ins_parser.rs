use crate::ins::{BfIns, BfCode};

/// Error type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BfParseError {
    /// Can occur in cases like: ``[+]]``
    UnmatchedClosingBracket { 
        /// unmatched bracket position
        char_pos: usize
    },
    /// Can occur in cases like:  ``[[+]``
    UnclosedBracket
}

/// # Parse sequence of chars into [`BfCode`] object
/// example:
/// ```
/// # use bf_tools::{ bf, ins_parser::parse_chars };
/// let result = parse_chars("+>[-]<.".chars());
/// let expected = bf!(+>[-]<.);
/// assert_eq!(result, Ok(expected));
/// ```
/// # Errors
/// return `Err` if string contains invalid bracket sequense like `]]` (all others strings is valid bf code)
pub fn parse_chars(chars: impl Iterator<Item = char>) -> Result<BfCode, BfParseError> {
    let mut loops_stack = Vec::new();
    loops_stack.push(Vec::new());
    for (pos, ch) in chars.enumerate() {
        match ch {
            '+' => {
                if let Some(last) = loops_stack.last_mut() {
                    last.push(BfIns::Add(1));
                } else {
                    unreachable!();
                }
            }
            '-' => {
                if let Some(last) = loops_stack.last_mut() {
                    last.push(BfIns::Sub(1));
                } else {
                    unreachable!();
                }
            }
            '>' => {
                if let Some(last) = loops_stack.last_mut() {
                    last.push(BfIns::PtrAdd(1));
                } else {
                    unreachable!();
                }
            }
            '<' => {
                if let Some(last) = loops_stack.last_mut() {
                    last.push(BfIns::PtrSub(1));
                } else {
                    unreachable!();
                }
            }
            '.' => {
                if let Some(last) = loops_stack.last_mut() {
                    last.push(BfIns::Putchar);
                } else {
                    unreachable!();
                }
            }
            ',' => {
                if let Some(last) = loops_stack.last_mut() {
                    last.push(BfIns::Getchar);
                } else {
                    unreachable!();
                }
            }
            '[' => loops_stack.push(Vec::new()),
            ']' => {
                if let Some(inner) = loops_stack.pop() {
                    if let Some(last) = loops_stack.last_mut() {
                        last.push(BfIns::Loop(inner));
                    } else {
                        return Err(BfParseError::UnmatchedClosingBracket {
                            char_pos: pos
                        });
                    }
                } else {
                    unreachable!();
                }
            }
            _ => {}
        }
    }

    let mut loops = loops_stack.into_iter();

    if let Some(ins) = loops.next() {
        if loops.next().is_some() {
            Err(BfParseError::UnclosedBracket)
        } else {
            Ok(BfCode(ins))
        }
    } else {
        unreachable!()
    }
}
/// # parse bf instructions from iterator without grooping
/// example:
/// ```
/// # use bf_tools::{ bf, ins_parser::parse_str };
/// let result = parse_str(",[[-]]<");
/// let expected = bf!(,[[-]]<);
/// assert_eq!(result, Ok(expected));
/// ```
/// # Errors
/// return `Err` if string contains invalid bracket sequense like `]]` (all others strings is valid bf code)
#[inline]
pub fn parse_str<'a, T>(s: T) -> Result<BfCode, BfParseError>
    where T: Into<&'a str> {
    parse_chars(s.into().chars())
}

impl std::str::FromStr for BfCode {
    type Err = BfParseError;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_str(s)
    }
}