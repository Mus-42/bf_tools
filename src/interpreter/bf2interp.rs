use super::{InterpCode, InterpIns};
use crate::optimizer::{
    opt_ins::{OptBlock, OptIns},
    OptCode,
};

/*
    bf_to_interp matches:
    `++++++` as `add 6, [0]`
    `[-]` as `set 0, [0]`
    `>[-]+++<` as `set 3, [1]`
    `>[-]+++[-<+>]<` as `set 0, [1]` `mul 3, [0]`
*/

#[doc(hidden)]
#[inline]
pub fn bf_to_interp(code: impl Into<OptCode>) -> InterpCode {
    let code: OptCode = code.into();
    let ret = bf_to_interp_translate_impl(code);
    //TODO remove useless repeating like "SetInputOffset 0"
    InterpCode(ret)
}

fn bf_to_interp_translate_impl(code: OptCode) -> Vec<InterpIns> {
    let mut ret = Vec::new();
    let mut cur_offset: Option<u32> = None;// TODO use by pass matcher
    for bl in code.0 {
        match bl {
            OptBlock::Block(inner) => {
                let max_ptr_offset = inner
                    .ins
                    .last_key_value()
                    .map(|(offset, _)| *offset)
                    .unwrap_or_default();
                if max_ptr_offset != 0 {
                    ret.push(if max_ptr_offset > 0 {
                        InterpIns::PtrAdd {
                            offset: max_ptr_offset as u32,
                        }
                    } else {
                        InterpIns::PtrSub {
                            offset: -max_ptr_offset as u32,
                        }
                    });
                }

                for (offset, cell_ins) in inner.ins {
                    let offset = (max_ptr_offset - offset) as u32;
                    for ins in cell_ins {
                        ret.push(match ins {
                            OptIns::Add(val) => InterpIns::Add { val, offset },
                            OptIns::Sub(val) => InterpIns::Sub { val, offset },
                            OptIns::Putchar => InterpIns::Putchar { offset },
                            OptIns::Getchar => InterpIns::Getchar { offset },
                        });
                    }
                }

                let rest_ptr_offset = inner.ptr_offset - max_ptr_offset;
                if rest_ptr_offset != 0 {
                    ret.push(if rest_ptr_offset > 0 {
                        InterpIns::PtrAdd {
                            offset: rest_ptr_offset as u32,
                        }
                    } else {
                        InterpIns::PtrSub {
                            offset: -rest_ptr_offset as u32,
                        }
                    });
                }
            }

            OptBlock::Loop(mut inner) => {
                match inner.0.as_slice() {
                    // deadloop
                    [] => { todo!() }
                    // [-] or [+]
                    [OptBlock::Block(inner)]
                        if inner.ins.len() == 1
                            && inner.ins.get(&0).map(|v| {
                                matches!(v.as_slice(), [OptIns::Add(1) | OptIns::Sub(1)])
                            }) == Some(true) =>
                    {
                        ret.push(InterpIns::Set { val: 0, offset: 0 });
                    }
                    // something like [>+<-]
                    //[OptBlock::Block(inner)] => {}
                    _ => {
                        while matches!(inner.0.as_slice(), [OptBlock::Loop(_)]) {
                            match inner.0.into_iter().next() {
                                Some(OptBlock::Loop(new_inner)) => {
                                    inner = new_inner;
                                }
                                _ => unreachable!()
                            }
                        }

                        let mut inner = bf_to_interp_translate_impl(inner);
                        let loop_body_end = ret.len() + 4 + inner.len();
                        ret.push(InterpIns::SetInputOffset {
                            new_input_offset: 0,
                        });
                        ret.push(InterpIns::JmpF {
                            dest: loop_body_end as u32,
                        });
                        let loop_body_beg = ret.len();
                        //update jump location
                        inner.iter_mut().for_each(|ins| match ins {
                            InterpIns::Jmp { dest }
                            | InterpIns::JmpT { dest }
                            | InterpIns::JmpF { dest } => {
                                *dest += loop_body_beg as u32;
                            }
                            _ => {}
                        });
                        ret.append(&mut inner);
                        ret.push(InterpIns::SetInputOffset {
                            new_input_offset: 0,
                        });
                        ret.push(InterpIns::JmpT {
                            dest: loop_body_beg as u32,
                        });
                        cur_offset = Some(0);
                    }
                }
            }
        }
    }
    ret
}

impl<T: Into<OptCode>> From<T> for InterpCode {
    #[inline]
    fn from(value: T) -> Self {
        bf_to_interp(value)
    }
}
