use super::ThumbInstruction;

pub struct Invalid;

impl ThumbInstruction for Invalid {
    fn execute(&self, cpu: &mut crate::cpu::Cpu, _: &mut crate::bus::Bus, instruction: u16) {
        // PC might be off
        let history = cpu.pc_history.iter().map(|addr| format!("{:X}", addr)).collect::<Vec<_>>().join(", ");
        panic!("Executed invalid thumb instruction {:016b} at PC {:X}.\nPC history is {}", instruction, cpu.get_reg(15), history);
    }

    fn disassembly(&self, _: u16) -> String {
        format!("Invalid")
    }
}
