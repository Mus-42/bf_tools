use super::{
    opt_ins::{BasicBlock, OptBlock, OptCode, IOOptIns},
    OptPass,
};

/// Group instructions like Add(1), Add(1) into single instruction Add(2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GroupInstructions;

impl OptPass for GroupInstructions {
    fn optimize(&self, code: OptCode, is_changed: &mut bool) -> OptCode {
        let ins_count = code.ins_len();
        
        let mut res = Vec::new();
        for block in code.0 {
            match block {
                OptBlock::Loop(inner) => {
                    // No sense to do loops like [a][b] (second never starts)
                    if !matches!(res.last(), Some(OptBlock::Loop(_))) {
                        res.push(OptBlock::Loop(self.optimize(inner, is_changed)));
                    }
                }
                OptBlock::Block(mut block) => {
                    block.ins.retain(|_offset, change| *change != 0);
                    if let Some(OptBlock::Block(last)) = res.last_mut() {
                        last.ptr_offset += block.ptr_offset;
                        for (offset, change) in block.ins {
                            *last.ins.entry(offset).or_default() += change;
                        }
                    } else {
                        res.push(OptBlock::Block(block));
                    }
                }
                io @ OptBlock::IOIns(_) => res.push(io),
            }
        }
        
        // TODO try group as many as possible instructions into single BB (concat BB with loop|io between if possible)

        let res = OptCode(res);

        *is_changed |= res.ins_len() < ins_count; // this pass only reduce instruction count

        res
    }
}
