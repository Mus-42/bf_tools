/// Single BF instruction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BfIns {
    /// wrapping add value to current cell
    /// `+` (val=1) or `++++` (val=4) in bf
    Add(u8),
    /// wrapping sub value from current cell
    /// `-` in bf
    Sub(u8),
    /// move pointer "right" (in positive direction) by N cells
    /// `>` (N=1) or `>>>>>>` (N=6) in bf
    PtrAdd(usize),
    /// move pointer "left" (in positive direction) by N cells
    /// `<` in bf
    PtrSub(usize),
    /// print current cell value (as u8) to output stream
    /// `.` in bf
    Putchar,
    /// replace current cell value with value from input stream
    /// `,` in bf
    Getchar,
    /// repeat inner instruction while value of current cell != to 0
    /// `[` inner `]` in bf
    Loop(BfCode),
}

/// Collection of [`BfIns`] instructions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BfCode(pub Vec<BfIns>);

impl BfCode {
    /// bf code length in [`BfIns`] instructions
    /// ```
    /// # use bf_tools::{ bf, optimizer::{ OptPass } };
    /// // code without grouping (produced by bf macro)
    /// let code = bf!(+++[-]>,<+);
    /// assert_eq!(code.ins_len(), 9);
    /// ```
    #[inline]
    pub fn ins_len(&self) -> usize {
        self.0.iter().fold(0, |v, i| {
            v + match i {
                BfIns::Loop(inner) => 1 + inner.ins_len(),
                _ => 1,
            }
        })
    }
    /// bf code length in characters
    /// ```
    /// # use bf_tools::bf;
    /// assert_eq!(bf!(+++[-]>,<+).chars_len(), 10);
    /// ```
    #[inline]
    pub fn chars_len(&self) -> usize {
        self.0.iter().fold(0, |v, i| {
            v + match i {
                BfIns::Loop(inner) => 2 + inner.chars_len(),
                BfIns::Add(v) | BfIns::Sub(v) => *v as usize,
                BfIns::PtrAdd(v) | BfIns::PtrSub(v) => *v,
                BfIns::Getchar | BfIns::Putchar => 1,
            }
        })
    }
}

impl std::fmt::Display for BfCode {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        for i in &self.0 {
            let (ch, cnt) = match i {
                BfIns::Add(cnt) => ('+', *cnt as usize),
                BfIns::Sub(cnt) => ('-', *cnt as usize),
                BfIns::PtrAdd(cnt) => ('>', *cnt),
                BfIns::PtrSub(cnt) => ('<', *cnt),
                BfIns::Putchar => ('.', 1),
                BfIns::Getchar => (',', 1),
                BfIns::Loop(iner) => {
                    f.write_char('[')?;
                    iner.fmt(f)?;
                    f.write_char(']')?;
                    continue;
                }
            };
            for _ in 0..cnt {
                f.write_char(ch)?;
            }
        }
        Ok(())
    }
}
/// Creates [`BfCode`] object from
///
/// Example:
/// ```
/// # use bf_tools::{ ins::{ BfIns, BfCode }, bf };
/// assert_eq!(
///     bf!(+[-]),
///     BfCode(vec![BfIns::Add(1), BfIns::Loop(BfCode(vec![BfIns::Sub(1)]))])
/// );
/// ```
#[macro_export]
macro_rules! bf {
    ($($t:tt)*) => {
        {
            #[allow(unused_mut)]
            let mut res: Vec<$crate::ins::BfIns> = Vec::new();
            $(
                $crate::bf_impl!(res $t);
            )*
            $crate::ins::BfCode(res)
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! bf_impl {
    ($name:ident +) => {
        $name.push($crate::ins::BfIns::Add(1));
    };
    ($name:ident -) => {
        $name.push($crate::ins::BfIns::Sub(1));
    };
    ($name:ident ->) => {
        $name.push($crate::ins::BfIns::Sub(1));
        $name.push($crate::ins::BfIns::PtrAdd(1));
    };
    ($name:ident <-) => {
        $name.push($crate::ins::BfIns::PtrSub(1));
        $name.push($crate::ins::BfIns::Sub(1));
    };
    ($name:ident >) => {
        $name.push($crate::ins::BfIns::PtrAdd(1));
    };
    ($name:ident >>) => {
        $name.push($crate::ins::BfIns::PtrAdd(2));
    };
    ($name:ident <) => {
        $name.push($crate::ins::BfIns::PtrSub(1));
    };
    ($name:ident <<) => {
        $name.push($crate::ins::BfIns::PtrSub(2));
    };
    ($name:ident .) => {
        $name.push($crate::ins::BfIns::Putchar);
    };
    ($name:ident ..) => {
        $name.push($crate::ins::BfIns::Putchar);
        $name.push($crate::ins::BfIns::Putchar);
    };
    ($name:ident ...) => {
        $name.push($crate::ins::BfIns::Putchar);
        $name.push($crate::ins::BfIns::Putchar);
        $name.push($crate::ins::BfIns::Putchar);
    };
    ($name:ident ,) => {
        $name.push($crate::ins::BfIns::Getchar)
    };
    ($name:ident [$($t:tt)*]) => {
        $name.push($crate::ins::BfIns::Loop(bf!($($t)*)));
    };
    ($($t:tt)*) => {
        compile_error!(concat!("invalid token given to `bf` macro: ", $($t)*));
    };
}
