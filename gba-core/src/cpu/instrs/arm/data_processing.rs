use crate::cpu::CPSR;
use crate::utils::add_overflows;
use crate::utils::sub_overflows;
use crate::utils::AddressableBits;
use crate::Bus;
use crate::Cpu;
use std::fmt::Debug;

use super::ArmInstruction;

#[derive(Clone, Copy)]
enum ShiftSource {
    Immediate(u32),
    Register(u32),
}

impl ShiftSource {
    fn get_amt(&self, cpu: &Cpu) -> u32 {
        match *self {
            Self::Immediate(imm) => imm,
            Self::Register(reg) => cpu.get_reg(reg).bits(0, 7),
        }
    }
}
impl Debug for ShiftSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ShiftSource::Immediate(x) => write!(f, "{:x}", x),
            ShiftSource::Register(x) => write!(f, "r{}", x),
        }
    }
}

#[derive(Clone, Copy)]
enum ShifterOperand {
    Imm { imm: u32, c: Option<bool> },
    LSL { rm: u32, shift_source: ShiftSource },
    LSR { rm: u32, shift_source: ShiftSource },
    ASR { rm: u32, shift_source: ShiftSource },
    ROR { rm: u32, shift_source: ShiftSource },
    RRX { rm: u32 },
}

impl Debug for ShifterOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Imm { imm, .. } => write!(f, "#{:x}", imm),
            Self::LSL { rm, shift_source } => match shift_source {
                ShiftSource::Immediate(0) => write!(f, "r{}", rm),
                ShiftSource::Immediate(imm) => write!(f, "r{}, LSL #{:x}", rm, imm),
                ShiftSource::Register(reg) => write!(f, "r{}, LSL r{}", rm, reg),
            },
            Self::LSR { rm, shift_source } => match shift_source {
                ShiftSource::Immediate(imm) => write!(f, "r{}, LSR #{:x}", rm, imm),
                ShiftSource::Register(reg) => write!(f, "r{}, LSR r{}", rm, reg),
            },
            Self::ASR { rm, shift_source } => match shift_source {
                ShiftSource::Immediate(imm) => write!(f, "r{}, ASR #{:x}", rm, imm),
                ShiftSource::Register(reg) => write!(f, "r{}, ASR r{}", rm, reg),
            },
            Self::ROR { rm, shift_source } => match shift_source {
                ShiftSource::Immediate(imm) => write!(f, "r{}, ROR #{:x}", rm, imm),
                ShiftSource::Register(reg) => write!(f, "r{}, ROR r{}", rm, reg),
            },
            Self::RRX { rm } => write!(f, "r{}, RRX", rm),
        }
    }
}

impl ShifterOperand {
    fn parse_immediate(instruction: u32) -> ShifterOperand {
        let imm = instruction.bits(0, 7);
        let rot = instruction.bits(8, 11);
        let value = imm.rotate_right(2 * rot);

        ShifterOperand::Imm {
            imm: value,
            c: if rot == 0 {
                None
            } else {
                Some(value.bit(31) == 1)
            },
        }
    }

    fn parse_shift(instruction: u32) -> ShifterOperand {
        let low_bit = (instruction >> 4) & 1;
        let shift_type_bits = (instruction >> 5) & 0b11;
        let rm = instruction & 0xf;

        let shift_source;
        if low_bit == 0 {
            // Read shift_amt from immediate field
            let shift_amt = instruction.bits(7, 11);
            shift_source = ShiftSource::Immediate(shift_amt);
        } else {
            // Read shift_amt from bottom byte of register
            let shift_reg = instruction.bits(8, 11);

            // TODO: Is this check necessary - maybe the format of instructions prevents this
            if shift_reg == 15 {
                panic!("shift reg cannot be r15")
            }

            shift_source = ShiftSource::Register(shift_reg);
        };

        match shift_type_bits {
            0b00 => Self::LSL { rm, shift_source },
            0b01 => Self::LSR { rm, shift_source },
            0b10 => Self::ASR { rm, shift_source },
            0b11 => {
                if let ShiftSource::Immediate(0) = shift_source {
                    Self::RRX { rm }
                } else {
                    Self::ROR { rm, shift_source }
                }
            }
            _ => unreachable!(),
        }
    }

    fn parse(instruction: u32) -> ShifterOperand {
        let is_immediate = instruction.bit(25) == 1;
        if is_immediate {
            Self::parse_immediate(instruction)
        } else {
            Self::parse_shift(instruction)
        }
    }

    /// Returns the value of op2 as well as the new carry flag
    fn op2(&self, cpu: &Cpu) -> (u32, bool) {
        match *self {
            Self::Imm { imm, c } => (
                imm,
                match c {
                    None => cpu.get_cpsr_bit(CPSR::C) == 1,
                    Some(b) => b,
                },
            ),
            Self::LSL { rm, shift_source } => {
                let shift_amt = shift_source.get_amt(cpu);
                let mut rm_val = cpu.get_reg(rm);
                if self.takes_extra_cycle() && rm == 15 {
                    rm_val += 4;
                }

                if shift_amt == 0 {
                    (rm_val, cpu.get_cpsr_bit(CPSR::C) == 1)
                } else if shift_amt < 32 {
                    (
                        rm_val << shift_amt,
                        rm_val.bit((32 - shift_amt).try_into().unwrap()) == 1,
                    )
                } else if shift_amt == 32 {
                    (0, rm_val.bit(0) == 1)
                } else {
                    (0, false)
                }
            }
            Self::LSR { rm, shift_source } => {
                let mut rm_val = cpu.get_reg(rm);
                if self.takes_extra_cycle() && rm == 15 {
                    rm_val += 4;
                }

                match shift_source {
                    ShiftSource::Immediate(imm) => {
                        if imm == 0 {
                            (0, rm_val.bit(31) == 1)
                        } else {
                            (
                                rm_val >> imm,
                                rm_val.bit((imm - 1).try_into().unwrap()) == 1,
                            )
                        }
                    }
                    ShiftSource::Register(reg) => {
                        let rs = cpu.get_reg(reg);
                        if rs.bits(0, 7) == 0 {
                            (rm_val, cpu.get_cpsr_bit(CPSR::C) == 1)
                        } else if rs.bits(0, 7) < 32 {
                            (
                                rm_val >> rs.bits(0, 7),
                                rm_val.bit((rs.bits(0, 7) - 1).try_into().unwrap()) == 1,
                            )
                        } else if rs.bits(0, 7) == 32 {
                            (0, rm_val.bit(31) == 1)
                        } else {
                            (0, false)
                        }
                    }
                }
            }
            Self::ASR { rm, shift_source } => {
                let mut rm_val = cpu.get_reg(rm);
                if self.takes_extra_cycle() && rm == 15 {
                    rm_val += 4;
                }

                match shift_source {
                    ShiftSource::Immediate(imm) => {
                        if imm == 0 {
                            if rm_val.bit(31) == 0 {
                                (0, false)
                            } else {
                                (0xffffffff, true)
                            }
                        } else {
                            (
                                ((rm_val as i32) >> imm) as u32,
                                rm_val.bit((imm - 1).try_into().unwrap()) == 1,
                            )
                        }
                    }
                    ShiftSource::Register(reg) => {
                        let lower_bits = cpu.get_reg(reg).bits(0, 7);
                        if lower_bits == 0 {
                            (rm_val, cpu.get_cpsr_bit(CPSR::C) == 1)
                        } else if lower_bits < 32 {
                            (
                                ((rm_val as i32) >> lower_bits) as u32,
                                rm_val.bit((lower_bits - 1).try_into().unwrap()) == 1,
                            )
                        } else {
                            if rm_val.bit(31) == 0 {
                                (0, false)
                            } else {
                                (0xffffffff, true)
                            }
                        }
                    }
                }
            }
            Self::ROR { rm, shift_source } => {
                let mut rm_val = cpu.get_reg(rm);
                if self.takes_extra_cycle() && rm == 15 {
                    rm_val += 4;
                }

                match shift_source {
                    ShiftSource::Immediate(imm) => (
                        rm_val.rotate_right(imm),
                        rm_val.bit((imm - 1).try_into().unwrap()) == 1,
                    ),
                    ShiftSource::Register(reg) => {
                        let rs = cpu.get_reg(reg);
                        if rs.bits(0, 7) == 0 {
                            (rm_val, cpu.get_cpsr_bit(CPSR::C) == 1)
                        } else if rs.bits(0, 4) == 0 {
                            (rm_val, cpu.get_reg(rm).bit(31) == 1)
                        } else {
                            (
                                rm_val.rotate_right(rs.bits(0, 4)),
                                cpu.get_reg(rm).bit((rs.bits(0, 4) - 1).try_into().unwrap()) == 1,
                            )
                        }
                    }
                }
            }
            Self::RRX { rm } => {
                let carry_in = cpu.get_cpsr_bit(CPSR::C);
                (
                    (cpu.get_reg(rm) >> 1).bits(0, 30) | carry_in << 31,
                    cpu.get_reg(rm).bit(0) == 1,
                )
            }
        }
    }

    fn takes_extra_cycle(&self) -> bool {
        let shift_source = match *self {
            Self::Imm { .. } | Self::RRX { .. } => return false,
            Self::LSL { shift_source, .. } => shift_source,
            Self::LSR { shift_source, .. } => shift_source,
            Self::ASR { shift_source, .. } => shift_source,
            Self::ROR { shift_source, .. } => shift_source,
        };
        if let ShiftSource::Register(_) = shift_source {
            return true;
        }
        return false;
    }
}

pub(super) struct DataProcessingFields {
    set: bool,
    rn: u32,
    rd: u32,
    shifter: ShifterOperand,
}

impl DataProcessingFields {
    fn parse(instruction: u32) -> DataProcessingFields {
        let set = (instruction >> 20) & 1 != 0;
        let rn = (instruction >> 16) & 0xf;
        let rd = (instruction >> 12) & 0xf;

        DataProcessingFields {
            set,
            rn,
            rd,
            shifter: ShifterOperand::parse(instruction),
        }
    }
}

pub struct And;
pub struct Eor;
pub struct Sub;
pub struct Rsb;
pub struct Add;
pub struct Sbc;
pub struct Rsc;
pub struct Adc;
pub struct Tst;
pub struct Teq;
pub struct Cmp;
pub struct Cmn;
pub struct Orr;
pub struct Mov;
pub struct Bic;
pub struct Mvn;

struct FlagUpdates {
    n: Option<bool>,
    z: Option<bool>,
    c: Option<bool>,
    v: Option<bool>,
}

#[inline]
fn execute_op<F>(cpu: &mut Cpu, instruction: u32, flag_only: bool, op_closure: F)
where
    // op1, op2, shifter carry
    F: Fn(u32, u32, bool) -> (u32, FlagUpdates),
{
    let fields = DataProcessingFields::parse(instruction);
    let (op2, c) = fields.shifter.op2(cpu);

    let op1 = if fields.shifter.takes_extra_cycle() && fields.rn == 15 {
        cpu.get_reg(fields.rn) + 4
    } else {
        cpu.get_reg(fields.rn)
    };

    let (output, flags) = op_closure(op1, op2, c);

    if fields.set && fields.rd == 15 {
        if cpu.mode_has_spsr() {
            cpu.regs.cpsr = cpu.regs.spsr(&cpu.get_mode());
        } else {
            //todo!("unpredictable {:04b}", instruction.bits(21, 24))
            cpu.regs.cpsr = cpu.regs.spsr(&cpu.get_mode());
        }
    } else if fields.set {
        if let Some(b) = flags.n {
            cpu.set_flag(CPSR::N, b);
        }
        if let Some(b) = flags.z {
            cpu.set_flag(CPSR::Z, b);
        }
        if let Some(b) = flags.c {
            cpu.set_flag(CPSR::C, b);
        }
        if let Some(b) = flags.v {
            cpu.set_flag(CPSR::V, b);
        }
    }

    if !flag_only {
        cpu.set_reg(fields.rd, output);
        if fields.rd == 15 {
            cpu.flush_pipeline();
        }
    }
}

impl ArmInstruction for And {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, instruction: u32) {
        execute_op(cpu, instruction, false, |op1, op2, shift_carry| {
            let result = op1 & op2;
            (
                result,
                FlagUpdates {
                    n: Some(result.bit(31) == 1),
                    z: Some(result == 0),
                    c: Some(shift_carry),
                    v: None,
                },
            )
        });
    }

    fn disassembly(&self, instruction: u32) -> String {
        let fields = DataProcessingFields::parse(instruction);
        format!("AND r{}, r{}, {:?}", fields.rd, fields.rn, fields.shifter)
    }
}

impl ArmInstruction for Eor {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, instruction: u32) {
        execute_op(cpu, instruction, false, |op1, op2, shift_carry| {
            let result = op1 ^ op2;
            (
                result,
                FlagUpdates {
                    n: Some(result.bit(31) == 1),
                    z: Some(result == 0),
                    c: Some(shift_carry),
                    v: None,
                },
            )
        });
    }

    fn disassembly(&self, instruction: u32) -> String {
        let fields = DataProcessingFields::parse(instruction);
        format!("EOR r{}, r{}, {:?}", fields.rd, fields.rn, fields.shifter)
    }
}

impl ArmInstruction for Sub {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, instruction: u32) {
        execute_op(cpu, instruction, false, |op1, op2, _| {
            let (result, borrow) = op1.overflowing_sub(op2);
            (
                result,
                FlagUpdates {
                    n: Some(result.bit(31) == 1),
                    z: Some(result == 0),
                    c: Some(!borrow),
                    v: Some(op1.bit(31) != op2.bit(31) && op1.bit(31) != result.bit(31)),
                },
            )
        })
    }

    fn disassembly(&self, instruction: u32) -> String {
        let fields = DataProcessingFields::parse(instruction);
        format!("SUB r{}, r{}, {:?}", fields.rd, fields.rn, fields.shifter)
    }
}

impl ArmInstruction for Rsb {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, instruction: u32) {
        execute_op(cpu, instruction, false, |op1, op2, _| {
            let (result, borrow) = op2.overflowing_sub(op1);
            (
                result,
                FlagUpdates {
                    n: Some(result.bit(31) == 1),
                    z: Some(result == 0),
                    c: Some(!borrow),
                    v: Some(op2.bit(31) != op1.bit(31) && op2.bit(31) != result.bit(31)),
                },
            )
        })
    }

    fn disassembly(&self, instruction: u32) -> String {
        let fields = DataProcessingFields::parse(instruction);
        format!("RSB r{}, r{}, {:?}", fields.rd, fields.rn, fields.shifter)
    }
}

impl ArmInstruction for Add {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, instruction: u32) {
        execute_op(cpu, instruction, false, |op1, op2, _| {
            let (result, c) = op1.overflowing_add(op2);
            let n = result.bit(31) == 1;
            let z = result == 0;
            let v = op1.bit(31) == op2.bit(31) && op1.bit(31) != result.bit(31);
            (
                result,
                FlagUpdates {
                    n: Some(n),
                    z: Some(z),
                    c: Some(c),
                    v: Some(v),
                },
            )
        });
    }

    fn disassembly(&self, instruction: u32) -> String {
        let fields = DataProcessingFields::parse(instruction);
        format!("ADD r{}, r{}, {:?}", fields.rd, fields.rn, fields.shifter)
    }
}

impl ArmInstruction for Sbc {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, instruction: u32) {
        execute_op(cpu, instruction, false, |op1, op2, shift_carry| {
            let (mut result, mut borrow) = op1.overflowing_sub(op2);
            let mut overflow = op1.bit(31) != op2.bit(31) && op1.bit(31) != result.bit(31);
            if !shift_carry {
                let (final_result, b2) = result.overflowing_sub(1);
                let overflow2 = result.bit(31) == 1 && final_result.bit(31) == 0;
                result = final_result;
                borrow |= b2;
                overflow |= overflow2;
            }

            (
                result,
                FlagUpdates {
                    n: Some(result.bit(31) == 1),
                    z: Some(result == 0),
                    c: Some(!borrow),
                    v: Some(overflow),
                },
            )
        });
    }

    fn disassembly(&self, instruction: u32) -> String {
        let fields = DataProcessingFields::parse(instruction);
        format!("SBC r{}, r{}, {:?}", fields.rd, fields.rn, fields.shifter)
    }
}

impl ArmInstruction for Rsc {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, instruction: u32) {
        execute_op(cpu, instruction, false, |op2, op1, shift_carry| {
            let (mut result, mut borrow) = op1.overflowing_sub(op2);
            let mut overflow = sub_overflows(op1, op2, result);
            if !shift_carry {
                let (final_result, b2) = result.overflowing_sub(1);
                let overflow2 = sub_overflows(result, 1, final_result);
                result = final_result;
                borrow |= b2;
                overflow |= overflow2;
            }

            (
                result,
                FlagUpdates {
                    n: Some(result.bit(31) == 1),
                    z: Some(result == 0),
                    c: Some(!borrow),
                    v: Some(overflow),
                },
            )
        });
    }

    fn disassembly(&self, instruction: u32) -> String {
        let fields = DataProcessingFields::parse(instruction);
        format!("RSC r{}, r{}, {:?}", fields.rd, fields.rn, fields.shifter)
    }
}

impl ArmInstruction for Adc {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, instruction: u32) {
        let c_flag = cpu.get_cpsr_bit(CPSR::C);
        execute_op(cpu, instruction, false, |op1, op2, _| {
            let (mut result, mut carry) = op1.overflowing_add(op2);
            let mut overflow = add_overflows(op1, op2, result);
            if c_flag == 1 {
                let (final_result, c2) = result.overflowing_add(1);
                let overflow2 = add_overflows(result, 1, final_result);
                result = final_result;
                carry |= c2;
                overflow |= overflow2;
            }

            (
                result,
                FlagUpdates {
                    n: Some(result.bit(31) == 1),
                    z: Some(result == 0),
                    c: Some(carry),
                    v: Some(overflow),
                },
            )
        });
    }

    fn disassembly(&self, instruction: u32) -> String {
        let fields = DataProcessingFields::parse(instruction);
        format!("ADC r{}, r{}, {:?}", fields.rd, fields.rn, fields.shifter)
    }
}

impl ArmInstruction for Tst {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, instruction: u32) {
        execute_op(cpu, instruction, true, |op1, op2, shift_carry| {
            let result = op1 & op2;
            (
                result,
                FlagUpdates {
                    n: Some(result.bit(31) == 1),
                    z: Some(result == 0),
                    c: Some(shift_carry),
                    v: None,
                },
            )
        });
    }

    fn disassembly(&self, instruction: u32) -> String {
        let fields = DataProcessingFields::parse(instruction);
        format!("TST r{}, {:?}", fields.rn, fields.shifter)
    }
}

impl ArmInstruction for Teq {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, instruction: u32) {
        execute_op(cpu, instruction, true, |op1, op2, shift_carry| {
            let result = op1 ^ op2;
            (
                result,
                FlagUpdates {
                    n: Some(result.bit(31) == 1),
                    z: Some(result == 0),
                    c: Some(shift_carry),
                    v: None,
                },
            )
        });
    }

    fn disassembly(&self, instruction: u32) -> String {
        let fields = DataProcessingFields::parse(instruction);
        format!("TEQ r{}, {:?}", fields.rn, fields.shifter)
    }
}

impl ArmInstruction for Cmp {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, instruction: u32) {
        execute_op(cpu, instruction, true, |op1, op2, _| {
            let (result, borrow) = op1.overflowing_sub(op2);

            (
                result,
                FlagUpdates {
                    n: Some(result.bit(31) == 1),
                    z: Some(result == 0),
                    c: Some(!borrow),
                    v: Some(sub_overflows(op1, op2, result)),
                },
            )
        });
    }

    fn disassembly(&self, instruction: u32) -> String {
        let fields = DataProcessingFields::parse(instruction);
        format!("CMP r{}, {:?}", fields.rn, fields.shifter)
    }
}

impl ArmInstruction for Cmn {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, instruction: u32) {
        execute_op(cpu, instruction, true, |op1, op2, _| {
            let (result, carry) = op1.overflowing_add(op2);
            (
                result,
                FlagUpdates {
                    n: Some(result.bit(31) == 1),
                    z: Some(result == 0),
                    c: Some(carry),
                    v: Some(add_overflows(op1, op2, result)),
                },
            )
        });
    }

    fn disassembly(&self, instruction: u32) -> String {
        let fields = DataProcessingFields::parse(instruction);
        format!("CMN r{}, {:?}", fields.rn, fields.shifter)
    }
}

impl ArmInstruction for Orr {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, instruction: u32) {
        execute_op(cpu, instruction, false, |op1, op2, shift_carry| {
            let result = op1 | op2;
            let n = result.bit(31) == 1;
            let z = result == 0;
            (
                result,
                FlagUpdates {
                    n: Some(n),
                    z: Some(z),
                    c: Some(shift_carry),
                    v: None,
                },
            )
        })
    }
    fn disassembly(&self, instruction: u32) -> String {
        let fields = DataProcessingFields::parse(instruction);
        format!("ORR r{}, r{}, {:?}", fields.rd, fields.rn, fields.shifter)
    }
}

impl ArmInstruction for Mov {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, instruction: u32) {
        execute_op(cpu, instruction, false, |_, op2, shift_carry| {
            (
                op2,
                FlagUpdates {
                    n: Some(op2.bit(31) == 1),
                    z: Some(op2 == 0),
                    c: Some(shift_carry),
                    v: None,
                },
            )
        });
    }

    fn disassembly(&self, instruction: u32) -> String {
        let fields = DataProcessingFields::parse(instruction);
        format!("MOV r{}, {:?}", fields.rd, fields.shifter)
    }
}

impl ArmInstruction for Bic {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, instruction: u32) {
        execute_op(cpu, instruction, false, |op1, op2, shift_carry| {
            let result = op1 & !op2;
            (
                result,
                FlagUpdates {
                    n: Some(result.bit(31) == 1),
                    z: Some(result == 0),
                    c: Some(shift_carry),
                    v: None,
                },
            )
        });
    }

    fn disassembly(&self, instruction: u32) -> String {
        let fields = DataProcessingFields::parse(instruction);
        format!("BIC r{}, rd{}, {:?}", fields.rd, fields.rn, fields.shifter)
    }
}

impl ArmInstruction for Mvn {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, instruction: u32) {
        execute_op(cpu, instruction, false, |_, op2, shift_carry| {
            let result = !op2;
            (
                result,
                FlagUpdates {
                    n: Some(result.bit(31) == 1),
                    z: Some(result == 0),
                    c: Some(shift_carry),
                    v: None,
                },
            )
        })
    }

    fn disassembly(&self, instruction: u32) -> String {
        let fields = DataProcessingFields::parse(instruction);
        format!("MVN r{}, {:?}", fields.rd, fields.shifter)
    }
}
