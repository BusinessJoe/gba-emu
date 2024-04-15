use super::ThumbInstruction;

pub struct Invalid;

impl ThumbInstruction for Invalid {
    fn execute(&self, cpu: &mut crate::cpu::Cpu, _: &mut crate::bus::Bus, instruction: u16) {
        // PC might be off
        panic!("Executed invalid thumb instruction {:016b} at PC {:X}", instruction, cpu.get_reg(15));
    }

    fn disassembly(&self, _: u16) -> String {
        format!("Invalid")
    }
}
