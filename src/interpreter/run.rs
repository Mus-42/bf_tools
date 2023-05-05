use super::{InterpCode, InterpIns, InterpreteError, Interpreter};

impl Interpreter<'_> {
    /// Execute all instructions from [`InterpCode`]
    pub fn run<C: Into<InterpCode>>(&mut self, code: C) -> Result<(), InterpreteError> {
        let mut ip = 0usize;
        let mut input_offset = 0u32;
        let code = code.into().0;
        self.reserve_storage()?;
        //TODO target feature to disable offset checks?
        while ip < code.len() {
            //println!("ip: {ip}, ins: {:?}, tape: {:?}, ptr: {:?}", &code[ip], &self.tape, self.data_pointer);
            match &code[ip] {
                InterpIns::Set { val, offset } => {
                    if self.data_pointer < *offset as usize {
                        println!("ip: {ip}, ins: {:?}, tape: {:?}", &code[ip], &self.tape);
                        return Err(InterpreteError::InvalidOffset);
                    }
                    self.tape[self.data_pointer - *offset as usize] = *val;
                }
                InterpIns::Add { val, offset } => {
                    if self.data_pointer < *offset as usize {
                        println!("ip: {ip}, ins: {:?}, tape: {:?}", &code[ip], &self.tape);
                        return Err(InterpreteError::InvalidOffset);
                    }
                    self.tape[self.data_pointer - *offset as usize] =
                        self.tape[self.data_pointer - *offset as usize].wrapping_add(*val);
                }
                InterpIns::Sub { val, offset } => {
                    if self.data_pointer < *offset as usize {
                        println!("ip: {ip}, ins: {:?}, tape: {:?}", &code[ip], &self.tape);
                        return Err(InterpreteError::InvalidOffset);
                    }
                    self.tape[self.data_pointer - *offset as usize] =
                        self.tape[self.data_pointer - *offset as usize].wrapping_sub(*val);
                }
                InterpIns::Mul { val, offset } => {
                    if self.data_pointer < *offset as usize {
                        println!("ip: {ip}, ins: {:?}, tape: {:?}", &code[ip], &self.tape);
                        return Err(InterpreteError::InvalidOffset);
                    }
                    self.tape[self.data_pointer - *offset as usize] =
                        self.tape[self.data_pointer - *offset as usize].wrapping_mul(*val);
                }

                InterpIns::PtrAdd { offset } => {
                    self.data_pointer += *offset as usize;
                    self.reserve_storage()?;
                }
                InterpIns::PtrSub { offset } => {
                    self.data_pointer = self
                        .data_pointer
                        .checked_sub(*offset as usize)
                        .ok_or(InterpreteError::DataPointerUnderflow)?;
                }

                InterpIns::SetInputOffset { new_input_offset } => {
                    input_offset = *new_input_offset;
                }

                InterpIns::AddMove { mul, to } => {
                    if self.data_pointer < *to as usize || self.data_pointer < input_offset as usize {
                        println!("ip: {ip}, ins: {:?}, tape: {:?}", &code[ip], &self.tape);
                        return Err(InterpreteError::InvalidOffset);
                    }
                    self.tape[self.data_pointer - *to as usize] =
                        self.tape[self.data_pointer - *to as usize].wrapping_add(
                            self.tape[self.data_pointer - input_offset as usize].wrapping_mul(*mul),
                        );
                    self.tape[self.data_pointer - input_offset as usize] = 0;
                }
                InterpIns::SubMove { mul, to } => {
                    if self.data_pointer < *to as usize || self.data_pointer < input_offset as usize
                    {
                        return Err(InterpreteError::InvalidOffset);
                    }
                    self.tape[self.data_pointer - *to as usize] =
                        self.tape[self.data_pointer - *to as usize].wrapping_sub(
                            self.tape[self.data_pointer - input_offset as usize].wrapping_mul(*mul),
                        );
                    self.tape[self.data_pointer - input_offset as usize] = 0;
                }
                InterpIns::MulMove { mul, to } => {
                    if self.data_pointer < *to as usize || self.data_pointer < input_offset as usize
                    {
                        return Err(InterpreteError::InvalidOffset);
                    }
                    self.tape[self.data_pointer - *to as usize] =
                        self.tape[self.data_pointer - *to as usize].wrapping_mul(
                            self.tape[self.data_pointer - input_offset as usize].wrapping_mul(*mul),
                        );
                    self.tape[self.data_pointer - input_offset as usize] = 0;
                }
                InterpIns::Move { to } => {
                    if self.data_pointer < *to as usize || self.data_pointer < input_offset as usize
                    {
                        return Err(InterpreteError::InvalidOffset);
                    }
                    self.tape[self.data_pointer - *to as usize] =
                        self.tape[self.data_pointer - input_offset as usize];
                    self.tape[self.data_pointer - input_offset as usize] = 0;
                }
                InterpIns::Copy { to } => {
                    if self.data_pointer < *to as usize || self.data_pointer < input_offset as usize
                    {
                        return Err(InterpreteError::InvalidOffset);
                    }
                    self.tape[self.data_pointer - *to as usize] +=
                        self.tape[self.data_pointer - input_offset as usize];
                }
                InterpIns::Putchar { offset } => {
                    if self.data_pointer < *offset as usize {
                        return Err(InterpreteError::InvalidOffset);
                    }
                    let ch = self.tape[self.data_pointer - *offset as usize];
                    self.io_out.putchar(ch).map_err(InterpreteError::IOError)?;
                }
                InterpIns::Getchar { offset } => {
                    if self.data_pointer < *offset as usize {
                        return Err(InterpreteError::InvalidOffset);
                    }
                    let ch = self.io_in
                        .getchar()
                        .map_err(InterpreteError::IOError)?;
                    self.tape[self.data_pointer - *offset as usize] = ch;
                }
                InterpIns::JmpT { dest } => {
                    if self.data_pointer < input_offset as usize {
                        return Err(InterpreteError::InvalidOffset);
                    }
                    if self.tape[self.data_pointer - input_offset as usize] != 0 {
                        ip = *dest as usize;
                        continue;
                    }
                }
                InterpIns::JmpF { dest } => {
                    if self.data_pointer < input_offset as usize {
                        return Err(InterpreteError::InvalidOffset);
                    }
                    if self.tape[self.data_pointer - input_offset as usize] == 0 {
                        ip = *dest as usize;
                        continue;
                    }
                }
                InterpIns::Jmp { dest } => {
                    ip = *dest as usize;
                    continue;
                }
            }
            ip += 1;
        }
        self.io_out.flush().map_err(InterpreteError::IOError)?;
        Ok(())
    }
    #[inline(always)]
    fn reserve_storage(&mut self) -> Result<(), InterpreteError> {
        let l = self.tape.len();
        let ptr = self.data_pointer;
        if ptr >= l {
            self.tape.resize((1 + ptr as usize).next_power_of_two(), 0)
        }
        Ok(())
    }
}
