use super::{
    opt_ins::{BasicBlock, OptBlock, OptCode, OptIns},
    OptPass,
};

/// Group instructions like Add(1), Add(1) into single instruction Add(2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GroupInstructions;

impl OptPass for GroupInstructions {
    fn optimize(&self, code: OptCode, is_changed: &mut bool) -> OptCode {
        let mut res = Vec::new();
        for block in code.0 {
            match block {
                block @ OptBlock::Block(_) => res.push(block),
                OptBlock::Loop(inner) => {
                    if matches!(res.last(), Some(OptBlock::Loop(_))) {
                        *is_changed = true;
                        continue; // No sense to do loops like [a][b] (second can't starts)
                    }
                    res.push(OptBlock::Loop(self.optimize(inner, is_changed)));
                }
            }
        }

        // TODO force concating `Block Block` into single block?
        debug_assert!(res.windows(2).all(|val|
            // valid only `Loop Block` or `Block Loop`
            matches!(val, [OptBlock::Loop(_), OptBlock::Block(_)] | [OptBlock::Block(_), OptBlock::Loop(_)])
        ));

        // Now our `res` looks like    ... Loop Block Loop Block Loop Block ...

        let mut prev_block: Option<&mut BasicBlock> = None;
        let mut prev_loop: Option<&OptCode> = None;

        for bl in res.iter_mut().rev() {
            match bl {
                OptBlock::Block(cur_block) => {
                    if let (Some(prev_loop), Some(prev_block)) = (&prev_loop, &mut prev_block) {
                        // code structure is: `bl` `prev_loop` `prev_block`
                        if matches!(prev_loop.offset(), Some(0)) {
                            use std::collections::BTreeSet;

                            // works only with 0-offset loops
                            fn cell_change_collector(
                                code: &OptCode,
                                edited_cells: &mut BTreeSet<isize>,
                            ) {
                                code.0.iter().fold(0isize, |offset, block| match block {
                                    OptBlock::Block(inner) => {
                                        inner.ins.iter().for_each(|(ins_offset, _)| {
                                            edited_cells.insert(offset + *ins_offset);
                                        });
                                        offset + inner.ptr_offset
                                    }
                                    OptBlock::Loop(inner) => {
                                        cell_change_collector(inner, edited_cells);
                                        offset
                                    }
                                });
                            }

                            let mut edited_cells = BTreeSet::new();
                            cell_change_collector(prev_loop, &mut edited_cells);

                            let loop_has_se = prev_loop.has_side_effects();
                            //we can move all unchanged cells related instructions
                            for (ins_offset, ins) in &mut prev_block.ins {
                                if edited_cells.contains(ins_offset) || ins.is_empty() {
                                    continue;
                                }
                                *is_changed = true;
                                let new_offset = cur_block.ptr_offset + *ins_offset;
                                if !loop_has_se {
                                    if let Some(cell_ins) = cur_block.ins.get_mut(&new_offset) {
                                        cell_ins.append(ins);
                                    } else {
                                        let mut moved_ins = Vec::new();
                                        std::mem::swap(&mut moved_ins, ins); // empty vector will be removed in next stage
                                        cur_block.ins.insert(new_offset, moved_ins);
                                    }
                                } else {
                                    let mut it = {
                                        let mut moved_ins = Vec::new();
                                        std::mem::swap(&mut moved_ins, ins);
                                        moved_ins.into_iter().peekable()
                                    };

                                    let cell_ins =
                                        cur_block.ins.entry(new_offset).or_insert(Vec::new());

                                    while let Some(ins) = it.peek() {
                                        if matches!(ins, OptIns::Getchar | OptIns::Putchar) {
                                            break;
                                        }
                                        cell_ins.push(it.next().unwrap_or(OptIns::Add(0)))
                                    }

                                    *ins = it.collect(); //rest
                                }
                            }
                        }
                    }
                    prev_block = Some(cur_block);
                }
                OptBlock::Loop(inner) => prev_loop = Some(inner),
            }
        }

        for bl in res.iter_mut() {
            if let OptBlock::Block(cur_block) = bl {
                //actually grouping
                cur_block.ins.iter_mut().for_each(|(_, ins)| {
                    let mut moved_ins = Vec::new();
                    std::mem::swap(&mut moved_ins, ins);
                    for v in moved_ins {
                        match v {
                            v @ (OptIns::Add(val) | OptIns::Sub(val)) => {
                                if let Some(prev @ (OptIns::Add(_) | OptIns::Sub(_))) =
                                    ins.last_mut()
                                {
                                    let add = matches!(
                                        (&v, &prev),
                                        (OptIns::Add(_), OptIns::Add(_))
                                            | (OptIns::Sub(_), OptIns::Sub(_))
                                    );
                                    *is_changed = true;
                                    match prev {
                                        OptIns::Add(prev_val) | OptIns::Sub(prev_val) => {
                                            if add {
                                                *prev_val = prev_val.wrapping_add(val);
                                            } else {
                                                *prev_val = prev_val.wrapping_sub(val);
                                            }
                                            if *prev_val == 0 {
                                                ins.pop();
                                            }
                                        }
                                        _ => unreachable!(),
                                    }
                                } else {
                                    ins.push(v);
                                }
                            }
                            v => ins.push(v),
                        }
                    }
                });

                cur_block.ins.retain(|_, ins| !ins.is_empty());

                //TODO try erase block
            }
        }

        OptCode(res)
    }
}
