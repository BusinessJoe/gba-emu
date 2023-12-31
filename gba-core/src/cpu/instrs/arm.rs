mod block_data_transfer;
mod branch;
mod branch_and_exchange;
mod data_processing;
mod halfword_transfer;
mod multiply;
mod psr_transfer;
mod single_data_swap;
mod single_data_transfer;
mod swi;

use crate::bus::Bus;
use crate::cpu::{Cpu, CPSR};
use crate::utils::AddressableBits;

pub trait ArmInstruction {
    fn execute(&self, cpu: &mut Cpu, bus: &mut Bus, instruction: u32);
    fn disassembly(&self, instruction: u32) -> String;
}

impl std::fmt::Debug for dyn ArmInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ArmInstruction")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MetaInstr {
    DataProcessing,
    PsrTransfer,
    Multiply,
    MultiplyLong,
    SingleDataSwap,
    BranchAndExchange,
    HalfwordTransReg,
    HalfwordTransImm,
    SingleDataTrans,
    Undefined,
    BlockDataTrans,
    Branch,
    CoprocDataTrans,
    CoprocDataOp,
    CoprocRegTrans,
    SoftwareInterrupt,
}

impl MetaInstr {
    /// Returns masks for bits in (high, low) format.
    /// High bits are 20-27 and low bits are 7-4.
    fn bit_format(&self) -> (u32, u32) {
        match *self {
            Self::DataProcessing => (0b0000_0000, 0b0000),
            Self::PsrTransfer => unimplemented!(),
            Self::Multiply => (0b0000_0000, 0b1001),
            Self::MultiplyLong => (0b0000_1000, 0b1001),
            Self::SingleDataSwap => (0b0001_0000, 0b1001),
            Self::BranchAndExchange => (0b0001_0010, 0b0001),
            Self::HalfwordTransReg => (0b0000_0000, 0b1001),
            Self::HalfwordTransImm => (0b0000_0100, 0b1001),
            Self::SingleDataTrans => (0b0100_0000, 0b0000),
            Self::Undefined => (0b0110_0000, 0b0001),
            Self::BlockDataTrans => (0b1000_0000, 0b0000),
            Self::Branch => (0b1010_0000, 0b0000),
            Self::CoprocDataTrans => (0b1100_0000, 0b0000),
            Self::CoprocDataOp => (0b1110_0000, 0b0000),
            Self::CoprocRegTrans => (0b1110_0000, 0b0001),
            Self::SoftwareInterrupt => (0b1111_0000, 0b0000),
        }
    }

    fn bit_mask(&self) -> (u32, u32) {
        match *self {
            Self::DataProcessing => (0b1100_0000, 0b0000),
            Self::PsrTransfer => unimplemented!(),
            Self::Multiply => (0b1111_1100, 0b1111),
            Self::MultiplyLong => (0b1111_1000, 0b1111),
            Self::SingleDataSwap => (0b1111_1011, 0b1111),
            Self::BranchAndExchange => (0b1111_1111, 0b1111),
            Self::HalfwordTransReg => (0b1110_0100, 0b1001),
            Self::HalfwordTransImm => (0b1110_0100, 0b1001),
            Self::SingleDataTrans => (0b1100_0000, 0b0000),
            Self::Undefined => (0b1110_0000, 0b0001),
            Self::BlockDataTrans => (0b1110_0000, 0b0000),
            Self::Branch => (0b1110_0000, 0b0000),
            Self::CoprocDataTrans => (0b1110_0000, 0b0000),
            Self::CoprocDataOp => (0b1111_0000, 0b0001),
            Self::CoprocRegTrans => (0b1111_0000, 0b0001),
            Self::SoftwareInterrupt => (0b1111_0000, 0b0000),
        }
    }

    pub(super) fn decode_arm(instruction: u32) -> Box<dyn ArmInstruction> {
        let high_bits = (instruction >> 20) & 0b1111_1111;
        let low_bits = (instruction >> 4) & 0b1111;

        let instrs = [
            Self::BranchAndExchange,
            Self::BlockDataTrans,
            Self::Branch,
            Self::SoftwareInterrupt,
            Self::Undefined,
            Self::SingleDataTrans,
            Self::SingleDataSwap,
            Self::Multiply,
            Self::MultiplyLong,
            Self::HalfwordTransReg,
            Self::HalfwordTransImm,
            Self::CoprocDataTrans,
            Self::CoprocDataOp,
            Self::CoprocRegTrans,
            Self::PsrTransfer,
            Self::DataProcessing,
        ];

        for meta_instr in instrs.into_iter() {
            if meta_instr != Self::PsrTransfer {
                let (high_mask, low_mask) = meta_instr.bit_mask();
                let (high_fmt, low_fmt) = meta_instr.bit_format();

                if high_bits & high_mask == high_fmt && low_bits & low_mask == low_fmt {
                    return meta_instr.get_arm_instruction(instruction);
                }
            } else {
                // PSR-specific check
                let opcode = instruction.bits(21, 24);
                let s = instruction.bit(20);
                let opcode_only_sets_flags = (0b1000..=0b1011).contains(&opcode);
                if instruction.bits(26, 27) == 0 && opcode_only_sets_flags && s == 0 {
                    return meta_instr.get_arm_instruction(instruction);
                }
            }
        }

        Box::new(UnimplementedInstruction)
    }

    fn get_arm_instruction(&self, instruction: u32) -> Box<dyn ArmInstruction> {
        match *self {
            Self::DataProcessing => match instruction.bits(21, 24) {
                0b0000 => Box::new(data_processing::And),
                0b0001 => Box::new(data_processing::Eor),
                0b0010 => Box::new(data_processing::Sub),
                0b0011 => Box::new(data_processing::Rsb),
                0b0101 => Box::new(data_processing::Adc),
                0b0100 => Box::new(data_processing::Add),
                0b0110 => Box::new(data_processing::Sbc),
                0b0111 => Box::new(data_processing::Rsc),
                0b1000 => Box::new(data_processing::Tst),
                0b1001 => Box::new(data_processing::Teq),
                0b1010 => Box::new(data_processing::Cmp),
                0b1011 => Box::new(data_processing::Cmn),
                0b1100 => Box::new(data_processing::Orr),
                0b1101 => Box::new(data_processing::Mov),
                0b1110 => Box::new(data_processing::Bic),
                0b1111 => Box::new(data_processing::Mvn),
                _ => unreachable!(),
            },
            Self::BlockDataTrans => Self::decode_block_data_transfer(instruction),
            Self::Branch => Box::new(branch::Branch),
            Self::BranchAndExchange => Box::new(branch_and_exchange::BranchAndExchange),
            Self::HalfwordTransImm => Self::decode_halfword_transfer(instruction),
            Self::HalfwordTransReg => Self::decode_halfword_transfer(instruction),
            Self::PsrTransfer => Self::decode_psr_transfer(instruction),
            Self::SingleDataTrans => Self::decode_single_data_transfer(instruction),
            Self::SoftwareInterrupt => Box::new(swi::Swi),
            Self::Multiply => multiply::decode_multiply(instruction),
            Self::MultiplyLong => multiply::decode_multiply_long(instruction),
            Self::SingleDataSwap => single_data_swap::decode_swap(instruction),

            // TODO
            Self::CoprocDataOp => Box::new(TodoInstruction::new_message(format!("{:?}", self))),
            Self::CoprocRegTrans => Box::new(TodoInstruction::new_message(format!("{:?}", self))),
            Self::CoprocDataTrans => Box::new(TodoInstruction::new_message(format!("{:?}", self))),
            Self::Undefined => Box::new(TodoInstruction::new_message(format!("{:?}", self))),
        }
    }
}

pub struct TodoInstruction(pub String);
impl TodoInstruction {
    pub fn new_message(message: String) -> Self {
        Self(message)
    }
}

impl ArmInstruction for TodoInstruction {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, _: u32) {
        todo!(
            "TODO: {} at PC: {:x}",
            self.0,
            cpu.get_executing_instruction_pc()
        )
    }

    fn disassembly(&self, _: u32) -> String {
        format!("TODO: {}", self.0)
    }
}

struct UnimplementedInstruction;
impl ArmInstruction for UnimplementedInstruction {
    fn execute(&self, cpu: &mut Cpu, _: &mut Bus, instruction: u32) {
        panic!(
            "Unimplemented instruction {:x} at PC {:x}",
            instruction,
            cpu.get_reg(15) - 8
        );
    }

    fn disassembly(&self, _instruction: u32) -> String {
        "Unimplemented".to_string()
    }
}

impl Cpu {
    pub fn check_cond(&mut self, cond_bits: u32) -> bool {
        match cond_bits {
            0b0000 => self.get_cpsr_bit(CPSR::Z) != 0,
            0b0001 => self.get_cpsr_bit(CPSR::Z) == 0,
            0b0010 => self.get_cpsr_bit(CPSR::C) != 0,
            0b0011 => self.get_cpsr_bit(CPSR::C) == 0,
            0b0100 => self.get_cpsr_bit(CPSR::N) != 0,
            0b0101 => self.get_cpsr_bit(CPSR::N) == 0,
            0b0110 => self.get_cpsr_bit(CPSR::V) != 0,
            0b0111 => self.get_cpsr_bit(CPSR::V) == 0,
            0b1000 => self.get_cpsr_bit(CPSR::C) != 0 && self.get_cpsr_bit(CPSR::Z) == 0,
            0b1001 => self.get_cpsr_bit(CPSR::C) == 0 || self.get_cpsr_bit(CPSR::Z) != 0,
            0b1010 => self.get_cpsr_bit(CPSR::N) == self.get_cpsr_bit(CPSR::V),
            0b1011 => self.get_cpsr_bit(CPSR::N) != self.get_cpsr_bit(CPSR::V),
            0b1100 => {
                self.get_cpsr_bit(CPSR::Z) == 0
                    && (self.get_cpsr_bit(CPSR::N) == self.get_cpsr_bit(CPSR::V))
            }
            0b1101 => {
                self.get_cpsr_bit(CPSR::Z) != 0
                    || (self.get_cpsr_bit(CPSR::N) != self.get_cpsr_bit(CPSR::V))
            }
            0b1110 => true,
            0b1111 => false, // Undefined behaviour
            _ => unreachable!(),
        }
    }

    pub fn disassemble_cond(cond_bits: u32) -> &'static str {
        match cond_bits {
            0b0000 => "EQ",
            0b0001 => "NE",
            0b0010 => "CS/HS",
            0b0011 => "CC/LO",
            0b0100 => "MI",
            0b0101 => "PL",
            0b0110 => "VS",
            0b0111 => "VC",
            0b1000 => "HI",
            0b1001 => "LS",
            0b1010 => "GE",
            0b1011 => "LT",
            0b1100 => "GT",
            0b1101 => "LE",
            0b1110 => "",
            0b1111 => "Unpredictable COND",
            _ => unreachable!(),
        }
    }

    pub fn decode_arm(instruction: u32) -> Box<dyn ArmInstruction> {
        MetaInstr::decode_arm(instruction)
    }
}
