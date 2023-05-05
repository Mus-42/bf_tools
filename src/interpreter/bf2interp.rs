use super::{InterpCode, InterpIns};
use crate::optimizer::{
    opt_ins::{OptBlock, IOOptIns},
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

                for (offset, val) in inner.ins {
                    let offset = (max_ptr_offset - offset) as u32;
                    ret.push(InterpIns::Add { val, offset });
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

            OptBlock::IOIns(IOOptIns::Putchar(offset)) => { 
                if offset > 0 {
                    ret.push(InterpIns::PtrAdd { offset: offset as u32 });
                    ret.push(InterpIns::Putchar { offset: 0 }); 
                    ret.push(InterpIns::PtrSub { offset: offset as u32 });
                } else {
                    ret.push(InterpIns::Putchar { offset: -offset as u32 }); 
                }
            }
            OptBlock::IOIns(IOOptIns::Getchar(offset)) => { 
                if offset > 0 {
                    ret.push(InterpIns::PtrAdd { offset: offset as u32 });
                    ret.push(InterpIns::Getchar { offset: 0 }); 
                    ret.push(InterpIns::PtrSub { offset: offset as u32 });
                } else {
                    ret.push(InterpIns::Getchar { offset: -offset as u32 }); 
                }
            }

            OptBlock::Loop(mut inner) => {
                while matches!(inner.0.as_slice(), [OptBlock::Loop(_)]) {
                    match inner.0.into_iter().next() {
                        Some(OptBlock::Loop(new_inner)) => {
                            inner = new_inner;
                        }
                        _ => unreachable!(),
                    }
                }
                match inner.0.as_slice() {
                    
                    // deadloop
                    [] => {
                        let at = ret.len();
                        ret.push(InterpIns::Jmp { dest: at as u32 }); //TODO indicate in some way about deadloop?
                        break;
                    }
                    // [-] or [+]
                    [OptBlock::Block(inner)]
                        if inner.ins.len() == 1
                            && inner.ptr_offset == 0
                            && inner.ins.get(&0).map(|v| {
                                *v == 1 || *v == 255
                            }).unwrap_or_default() =>
                    {
                        ret.push(InterpIns::Set { val: 0, offset: 0 });
                    }
                    // something like [>++<-]
                    [OptBlock::Block(inner)]
                        if inner.ins.len() == 2
                            && inner.ptr_offset == 0
                            && inner.ins.iter().any(|(pos, val)| *pos == 0 && (*val == 1 || *val == 2)) =>
                    {
                        let mut offset = 0;
                        let mut is_add = false;
                        let mut mul = 1;

                        inner.ins.iter().for_each(|(pos, v)| {
                            is_add ^= *v > 127;
                            if *pos != 0 {
                                offset = *pos;
                                mul = *v;
                            }
                        });

                        if mul == 0 {
                            ret.push(InterpIns::Set { val: 0, offset: 0 });
                            // TODO ..?
                        } else {
                            if offset > 0 {
                                ret.push(InterpIns::PtrAdd { offset: offset as u32 });
                                ret.push(InterpIns::SetInputOffset { new_input_offset: offset as u32 });
                                ret.push(if is_add {
                                    InterpIns::AddMove { mul, to: 0 }
                                } else {
                                    InterpIns::SubMove { mul, to: 0 }
                                });
                                ret.push(InterpIns::PtrSub { offset: offset as u32 });
                            } else {
                                ret.push(InterpIns::SetInputOffset { new_input_offset: 0 });
                                ret.push(if is_add {
                                    InterpIns::AddMove { mul, to: -offset as u32 }
                                } else {
                                    InterpIns::SubMove { mul, to: -offset as u32 }
                                });
                            }
                        }
                    }
                    // TODO matcher for multiplication
                    _ => {
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
