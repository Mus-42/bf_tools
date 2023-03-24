/// [`OptState`] - Optimization state
///
/// it just a collection of optimization passes
#[derive(Debug)]
pub struct OptState {
    passes: Vec<Box<dyn OptPass>>,
}

/// Builder for [`OptState`]
#[derive(Debug)]
pub struct OptStateBuilder(Vec<Box<dyn OptPass>>);

impl OptState {
    /// Create builder object for [`OptState`]
    #[inline]
    pub fn builder() -> OptStateBuilder {
        OptStateBuilder::new()
    }
    /// Run all state passes
    #[inline]
    pub fn run_passes(&mut self, mut code: OptCode) -> OptCode {
        loop {
            //TODO prevent deadloop using something like hash..?
            // (can be usable when we have transformations like
            // `[-]+++.[-]`  ->  `[-]+++.---`  ->  `[-]+++.[-]`
            // code changes between passes but result - not)
            let mut is_changed = false;
            for pass in &self.passes {
                let mut cur_changed = false;
                code = pass.optimize(code, &mut cur_changed);
                is_changed |= cur_changed;
            }
            //TODO also check code len? (debug_assert)
            if !is_changed {
                break;
            }
        }
        code
    }
}

impl OptStateBuilder {
    /// Create new builder for [`OptState`]
    #[inline]
    pub fn new() -> Self {
        OptStateBuilder(Vec::new())
    }
    /// Add default passes to state
    ///
    /// Now default passes is:
    /// [`passes::PassUseless`]
    #[inline]
    pub fn add_default_passes(self) -> Self {
        self.add_pass(Box::from(passes::GroupInstructions))
    }
    /// Add optimization pass to state
    #[inline]
    pub fn add_pass(mut self, pass: Box<dyn OptPass>) -> Self {
        self.0.push(pass);
        self
    }
    /// Finish building [`OptState`] and return them
    #[inline]
    pub fn build(self) -> OptState {
        OptState { passes: self.0 }
    }
}

impl Default for OptState {
    #[inline]
    fn default() -> Self {
        OptStateBuilder::default().build()
    }
}

impl Default for OptStateBuilder {
    #[inline]
    fn default() -> Self {
        Self::new().add_default_passes()
    }
}

/// Optimizer inner instruction representation
pub mod opt_ins {
    use std::collections::BTreeMap;

    use crate::ins::{BfCode, BfIns};

    /// Block of optimizer instruction
    #[derive(Debug, Clone)]
    pub struct OptCode(pub Vec<OptBlock>);

    #[derive(Debug, Clone)]
    /// Block type (loop or basic block)
    pub enum OptBlock {
        /// Loop over inner code
        Loop(OptCode),
        /// Block without loops inside
        Block(BasicBlock),
    }

    /// Block without loops inside & precalculated offset
    #[derive(Debug, Clone)]
    pub struct BasicBlock {
        /// Ptr offset per block
        pub ptr_offset: isize,
        /// vec of instruction for each offset
        pub ins: BTreeMap<isize, Vec<OptIns>>,
    }

    /// Instruction for optimizer
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum OptIns {
        /// wrapping add value to current cell
        Add(u8),
        /// wrapping sub value from current cell
        Sub(u8),
        /// print current cell value (as u8) to output stream
        Putchar,
        /// replace current cell value with value from input stream
        Getchar,
    }

    impl From<BfCode> for OptCode {
        fn from(value: BfCode) -> Self {
            let mut offset = 0isize;
            let mut cells = BTreeMap::new();
            let add_cell = |offset, ins, cells: &mut BTreeMap<isize, Vec<OptIns>>| {
                if let Some(v) = cells.get_mut(&offset) {
                    v.push(ins);
                } else {
                    cells.insert(offset, vec![ins]);
                }
            };
            let mut res = Vec::new();
            for ins in value.0 {
                match ins {
                    BfIns::Add(val) => add_cell(offset, OptIns::Add(val), &mut cells),
                    BfIns::Sub(val) => add_cell(offset, OptIns::Sub(val), &mut cells),
                    BfIns::PtrAdd(d) => offset += d as isize,
                    BfIns::PtrSub(d) => offset -= d as isize,
                    BfIns::Putchar => add_cell(offset, OptIns::Putchar, &mut cells),
                    BfIns::Getchar => add_cell(offset, OptIns::Getchar, &mut cells),
                    BfIns::Loop(inner) => {
                        if !cells.is_empty() || offset != 0 {
                            let mut ins = BTreeMap::new();
                            std::mem::swap(&mut ins, &mut cells);
                            res.push(OptBlock::Block(BasicBlock {
                                ptr_offset: offset,
                                ins,
                            }));
                            offset = 0;
                        }
                        res.push(OptBlock::Loop(inner.into()));
                    }
                }
            }
            if !cells.is_empty() || offset != 0 {
                res.push(OptBlock::Block(BasicBlock {
                    ptr_offset: offset,
                    ins: cells,
                }));
            }
            OptCode(res)
        }
    }

    impl From<OptCode> for BfCode {
        fn from(value: OptCode) -> Self {
            let mut code = Vec::new();
            for v in value.0 {
                match v {
                    OptBlock::Loop(inner) => code.push(BfIns::Loop(BfCode::from(inner))),
                    OptBlock::Block(bb) => {
                        let mut offset = 0isize;
                        for (new_offset, ins) in bb.ins {
                            if offset != new_offset {
                                let d = new_offset - offset;
                                offset = new_offset;
                                if d > 0 {
                                    code.push(BfIns::PtrAdd(d as usize));
                                } else {
                                    code.push(BfIns::PtrSub(-d as usize));
                                }
                            }
                            for v in ins {
                                code.push(match v {
                                    OptIns::Add(cnt) => BfIns::Add(cnt),
                                    OptIns::Sub(cnt) => BfIns::Sub(cnt),
                                    OptIns::Putchar => BfIns::Putchar,
                                    OptIns::Getchar => BfIns::Getchar,
                                });
                            }
                        }
                        if offset != bb.ptr_offset {
                            let d = bb.ptr_offset - offset;
                            if d > 0 {
                                code.push(BfIns::PtrAdd(d as usize));
                            } else {
                                code.push(BfIns::PtrSub(-d as usize));
                            }
                        }
                    }
                }
            }
            BfCode(code)
        }
    }

    impl OptCode {
        /// OptCode len in instruction (without offset's counting)
        pub fn ins_len(&self) -> usize {
            self.0.iter().fold(0usize, |l, b| {
                l + match b {
                    OptBlock::Block(b) => {
                        b.ins.iter().fold(0usize, |l, (_offset, ins)| l + ins.len())
                    }
                    OptBlock::Loop(inner) => inner.ins_len(),
                }
            })
        }
        /// Get data poiner offset
        pub fn offset(&self) -> Option<isize> {
            let mut offset = 0;
            for ins in &self.0 {
                match ins {
                    OptBlock::Block(bb) => offset += bb.ptr_offset,
                    OptBlock::Loop(inner) => {
                        //TODO fix for loops like [[-]]? (with single loop instruction inside)
                        if !matches!(inner.offset(), Some(0)) {
                            return None;
                        }
                    }
                }
            }
            Some(offset)
        }
        /// Check for Putchar|Getchar instructions in code block
        pub fn has_side_effects(&self) -> bool {
            self.0.iter().any(|b| match b {
                OptBlock::Block(b) => b.ins.iter().any(|(_offset, ins)| {
                    ins.iter()
                        .any(|v| matches!(v, OptIns::Putchar | OptIns::Getchar))
                }),
                OptBlock::Loop(inner) => inner.has_side_effects(),
            })
        }
    }
}

pub use opt_ins::OptCode;

/// Optimization pass trait
pub trait OptPass: std::fmt::Debug {
    /// Function for pass invocation
    ///
    /// is_changed - mark for [`OptState::run_passes`] when them needs to stop
    fn optimize(&self, code: OptCode, is_changed: &mut bool) -> OptCode;
}

/// Useless instruction pass
pub mod group_instructions;

/// All built-in passes grouped in one module
pub mod passes {
    pub use super::group_instructions::GroupInstructions;
}
