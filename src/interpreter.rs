// extended & flat instruction set (to speedup code)
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpIns {
    Set { val: u8, offset: u32 }, // cells[ptr - offset] = val
    Add { val: u8, offset: u32 }, // cells[ptr - offset] += val
    Sub { val: u8, offset: u32 }, // cells[ptr - offset] -= val
    Mul { val: u8, offset: u32 }, // cells[ptr - offset] *= val

    PtrAdd { offset: u32 }, // ptr += offset
    PtrSub { offset: u32 }, // ptr -= offset

    //TODO instructions for loops like [>] [<]?

    //TODO more register-like variables like input_offset?

    SetInputOffset { new_input_offset: u32 }, // input_offset = new_input_offset

    AddMove { to: u32 }, // cells[ptr - to] += cells[ptr - input_offset]; cells[ptr - input_offset] = 0
    SubMove { to: u32 }, // cells[ptr - to] -= cells[ptr - input_offset]; cells[ptr - input_offset] = 0
    MulMove { to: u32 }, // cells[ptr - to] *= cells[ptr - input_offset]; cells[ptr - input_offset] = 0
    Move { to: u32 }, // cells[ptr - to] = cells[ptr - input_offset]; cells[ptr - input_offset] = 0
    Copy { to: u32 }, // cells[ptr - to] = cells[ptr - input_offset];

    Putchar { offset: u32 }, // putchar(cells[ptr - offset])
    Getchar { offset: u32 }, // cells[ptr - offset] = getchar()

    JmpT { dest: u32 }, // if cells[ptr - input_offset] != 0 { ip = dest; }
    JmpF { dest: u32 }, // if cells[ptr - input_offset] == 0 { ip = dest; }
    Jmp { dest: u32 },  // ip = dest;
}

/// Collection of [`InterpIns`] instructions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterpCode(pub Vec<InterpIns>);

impl std::fmt::Display for InterpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ins in &self.0 {
            match ins {
                InterpIns::Set { val, offset } => {
                    f.write_fmt(format_args!("set {val}, [{offset}]\n"))?
                }
                InterpIns::Add { val, offset } => {
                    f.write_fmt(format_args!("add {val}, [{offset}]\n"))?
                }
                InterpIns::Sub { val, offset } => {
                    f.write_fmt(format_args!("sub {val}, [{offset}]\n"))?
                }
                InterpIns::Mul { val, offset } => {
                    f.write_fmt(format_args!("mul {val}, [{offset}]\n"))?
                }

                InterpIns::PtrAdd { offset } => f.write_fmt(format_args!("ptr_add {offset}\n"))?,
                InterpIns::PtrSub { offset } => f.write_fmt(format_args!("ptr_sub {offset}\n"))?,

                InterpIns::SetInputOffset { new_input_offset } => {
                    f.write_fmt(format_args!("set_input_offset {new_input_offset}\n"))?
                }

                InterpIns::AddMove { to } => {
                    f.write_fmt(format_args!("add_move [input_offset], [{to}]\n"))?
                }
                InterpIns::SubMove { to } => {
                    f.write_fmt(format_args!("sub_move [input_offset], [{to}]\n"))?
                }
                InterpIns::MulMove { to } => {
                    f.write_fmt(format_args!("mul_move [input_offset], [{to}]\n"))?
                }
                InterpIns::Move { to } => {
                    f.write_fmt(format_args!("move [input_offset], [{to}]\n"))?
                }
                InterpIns::Copy { to } => {
                    f.write_fmt(format_args!("copy [input_offset], [{to}]\n"))?
                }

                InterpIns::Putchar { offset } => {
                    f.write_fmt(format_args!("putchar [{offset}]\n"))?
                }
                InterpIns::Getchar { offset } => {
                    f.write_fmt(format_args!("getchar [{offset}]\n"))?
                }

                InterpIns::JmpT { dest } => {
                    f.write_fmt(format_args!("jmp_t [input_offset], '{dest}\n"))?
                }
                InterpIns::JmpF { dest } => {
                    f.write_fmt(format_args!("jmp_f [input_offset], '{dest}\n"))?
                }
                InterpIns::Jmp { dest } => f.write_fmt(format_args!("jmp '{dest}\n"))?,
            }
        }
        Ok(())
    }
}

/// Converter from [`crate::ins::BfCode`] to [`InterpCode`]
pub mod bf2interp;
/// Interpreter run implementation
pub mod run;

/// InterpreteError
#[derive(Debug)]
pub enum InterpreteError {
    /// invalid data pointer index (-1 etc.)
    DataPointerUnderflow,
    /// data pointer - offset < 0
    InvalidOffset,
    /// Getchar | Putchar fails with io error
    IOError(std::io::Error),
}

impl std::fmt::Display for InterpreteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("InterpreteError") //TODO
    }
}

impl std::error::Error for InterpreteError {}

/// Types which can be passed as stdin to [`Interpreter`]
pub trait InterprIOIn: std::io::Read + std::fmt::Debug {}
/// Types which can be passed as stdout to [`Interpreter`]
pub trait InterprIOOut: std::io::Write + std::fmt::Debug {}

impl<T: std::io::Read + std::fmt::Debug> InterprIOIn for T {}
impl<T: std::io::Write + std::fmt::Debug> InterprIOOut for T {}

/// Interpreter
#[derive(Debug)]
pub struct Interpreter<'a> {
    /// Data tape for interpreter
    pub tape: Vec<u8>,
    /// Current pointer location on tape
    pub data_pointer: usize,
    /// input for Getchar instuction
    pub io_in: Box<dyn InterprIOIn + 'a>,
    /// output for Putchar instuction
    pub io_out: Box<dyn InterprIOOut + 'a>,
}

/// Builder for [`Interpreter`]
#[derive(Debug)]
pub struct InterpreterBuilder<'a> {
    /// input for Getchar instuction
    io_in: Box<dyn InterprIOIn + 'a>,
    /// output for Putchar instuction
    io_out: Box<dyn InterprIOOut + 'a>,
}

impl<'a> Interpreter<'a> {
    /// Create builder for [`Interpreter`]
    #[inline]
    pub fn builder() -> InterpreterBuilder<'a> {
        InterpreterBuilder::new()
    }
    /// Clear interpreter's tape & set data pointer to 0
    #[inline]
    pub fn reset(&mut self) {
        self.data_pointer = 0;
        self.tape.clear();
    }
}

impl Default for Interpreter<'_> {
    #[inline]
    fn default() -> Self {
        InterpreterBuilder::default().build()
    }
}

impl<'a> InterpreterBuilder<'a> {
    /// create new [`InterpreterBuilder`]
    #[inline]
    pub fn new() -> Self {
        Self {
            io_in: Box::from(std::io::stdin()),
            io_out: Box::from(std::io::stdout()),
        }
    }
    /// finish building [`Interpreter`] and return result
    #[inline]
    pub fn build(self) -> Interpreter<'a> {
        Interpreter {
            tape: Vec::new(),
            data_pointer: 0,
            io_in: self.io_in,
            io_out: self.io_out,
        }
    }
    /// set input stream
    #[inline]
    pub fn set_stdin(mut self, io_in: impl InterprIOIn + 'a) -> Self {
        self.io_in = Box::from(io_in);
        self
    }
    /// set output stream
    #[inline]
    pub fn set_stdout(mut self, io_out: impl InterprIOOut + 'a) -> Self {
        self.io_out = Box::from(io_out);
        self
    }
}

impl Default for InterpreterBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}
