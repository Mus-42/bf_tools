
/// Enumeration for single instruction
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
    Loop(Vec<BfIns>)
}

/// [`BfCode`] - collection of [`BfIns`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BfCode(pub Vec<BfIns>);
/// Creates [`BfCode`] object
/// 
/// Example:
/// ```
/// # use bf_tools::{ ins::{ BfIns, BfCode }, bf };
/// assert_eq!(bf!(+[-]), BfCode(vec![BfIns::Add(1), BfIns::Loop(vec![BfIns::Sub(1)])]));
/// ```
#[macro_export]
macro_rules! bf {
    (__impl $name:ident +) => {
        $name.push($crate::ins::BfIns::Add(1));
    };
    (__impl $name:ident -) => {
        $name.push($crate::ins::BfIns::Sub(1));
    };
    (__impl $name:ident ->) => {
        $name.push($crate::ins::BfIns::Sub(1));
        $name.push($crate::ins::BfIns::PtrAdd(1));
    };
    (__impl $name:ident <-) => {
        $name.push($crate::ins::BfIns::PtrSub(1));
        $name.push($crate::ins::BfIns::Sub(1));
    };
    (__impl $name:ident >) => {
        $name.push($crate::ins::BfIns::PtrAdd(1));
    };
    (__impl $name:ident >>) => {
        $name.push($crate::ins::BfIns::PtrAdd(2));
    };
    (__impl $name:ident <) => {
        $name.push($crate::ins::BfIns::PtrSub(1));
    };
    (__impl $name:ident <<) => {
        $name.push($crate::ins::BfIns::PtrSub(2));
    };
    (__impl $name:ident .) => {
        $name.push($crate::ins::BfIns::Putchar);
    };
    (__impl $name:ident ..) => {
        $name.push($crate::ins::BfIns::Putchar);
        $name.push($crate::ins::BfIns::Putchar);
    };
    (__impl $name:ident ...) => {
        $name.push($crate::ins::BfIns::Putchar);
        $name.push($crate::ins::BfIns::Putchar);
        $name.push($crate::ins::BfIns::Putchar);
    };
    (__impl $name:ident ,) => {
        $name.push($crate::ins::BfIns::Getchar)
    };
    (__impl $name:ident [$($t:tt)*]) => {
        $name.push($crate::ins::BfIns::Loop(bf!($($t)*).0));
    };
    (__impl $($t:tt)*) => {
        compile_error!(concat!("invalid token given to `bf` macro: ", $($t)*));
    };
    ($($t:tt)*) => {
        {
            let mut res: Vec<$crate::ins::BfIns> = Vec::new();
            $(
                bf!(__impl res $t);
            )*
            $crate::ins::BfCode(res)
        }
    };
}