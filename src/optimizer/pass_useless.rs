#![doc(hidden)]

use super::*;
use crate::ins::{BfCode, BfIns};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PassUseless;

impl OptPass for PassUseless {
    fn optimize(&self, code: BfCode, is_changed: &mut bool) -> BfCode {
        fn optimize_impl(mut code: BfCode, is_changed: &mut bool) -> BfCode {
            loop {
                let prev_len = code.ins_len();
                let mut optimized = Vec::new();
                for ins in code.0 {
                    match (ins, optimized.last_mut()) {
                        (ins, None) => optimized.push(ins),
                        // [last][cur] - second loop is never executed (it can't start because cell value always 0)
                        (BfIns::Loop(_), Some(BfIns::Loop(_))) => {}
                        (BfIns::Loop(inner), _) => {
                            optimized.push(BfIns::Loop(optimize_impl(BfCode(inner), is_changed).0))
                        }
                        //group unstructions: `++-+-+` -> `++`
                        (
                            ins @ (BfIns::Add(_) | BfIns::Sub(_)),
                            Some(prev @ (BfIns::Add(_) | BfIns::Sub(_))),
                        ) => {
                            let cur_change = match ins {
                                BfIns::Add(v) => v,
                                BfIns::Sub(v) => 0u8.wrapping_sub(v), //using modular arithmetics
                                _ => unreachable!(),
                            };
                            let prev_change = match prev {
                                BfIns::Add(v) => *v,
                                BfIns::Sub(v) => 0u8.wrapping_sub(*v),
                                _ => unreachable!(),
                            };
                            let change = cur_change.wrapping_add(prev_change);
                            if change == 0 {
                                optimized.pop();
                            } else {
                                const CHANGE_THRESHOLD: u8 = u8::MAX / 2;
                                if change < CHANGE_THRESHOLD {
                                    *prev = BfIns::Add(change);
                                } else {
                                    *prev = BfIns::Sub(0u8.wrapping_sub(change));
                                }
                            }
                        }
                        (
                            ins @ (BfIns::PtrAdd(_) | BfIns::PtrSub(_)),
                            Some(prev @ (BfIns::PtrAdd(_) | BfIns::PtrSub(_))),
                        ) => {
                            let cur_change = match ins {
                                BfIns::PtrAdd(v) => v as isize,
                                BfIns::PtrSub(v) => -(v as isize),
                                _ => unreachable!(),
                            };
                            let prev_change = match prev {
                                BfIns::PtrAdd(v) => *v as isize,
                                BfIns::PtrSub(v) => -(*v as isize),
                                _ => unreachable!(),
                            };
                            let change = cur_change + prev_change;
                            if change == 0 {
                                optimized.pop();
                                continue;
                            }
                            if change > 0 {
                                *prev = BfIns::PtrAdd(change as usize)
                            } else {
                                *prev = BfIns::PtrSub(-change as usize)
                            }
                        }
                        (ins, _) => optimized.push(ins),
                    }
                }
                let optimized = BfCode(optimized);
                let len = optimized.ins_len();
                code = optimized;
                if prev_len == len {
                    break;
                }
                *is_changed = true;
            }
            code
        }
        optimize_impl(code, is_changed)
    }
}

#[cfg(test)]
mod tests {
    use super::PassUseless;
    use crate::{bf, optimizer::OptPass};

    #[test]
    fn pass_useless() {
        let mut is_changed = Default::default();
        //remove useless loops
        assert_eq!(
            PassUseless.optimize(bf!([-][-][-][-][-]), &mut is_changed),
            bf!([-])
        );
        assert_eq!(
            PassUseless.optimize(bf!([>][+->>>[-]]), &mut is_changed),
            bf!([>])
        );
        assert_eq!(
            PassUseless.optimize(bf!([>][+->>>[-]]), &mut is_changed),
            bf!([>])
        );
        // grouping
        assert_eq!(PassUseless.optimize(bf!(+++---), &mut is_changed), bf!());
        assert_eq!(PassUseless.optimize(bf!(+++--), &mut is_changed), bf!(+));
        assert_eq!(PassUseless.optimize(bf!(++---), &mut is_changed), bf!(-));
        assert_eq!(PassUseless.optimize(bf!(--+++), &mut is_changed), bf!(+));
        assert_eq!(PassUseless.optimize(bf!(>><<), &mut is_changed), bf!());
        assert_eq!(PassUseless.optimize(bf!(>><><), &mut is_changed), bf!(>));
        // don't reorder:
        assert_eq!(
            PassUseless.optimize(bf!(>+<->+<->+<), &mut is_changed),
            bf!(>+<->+<->+<)
        );
    }
}
