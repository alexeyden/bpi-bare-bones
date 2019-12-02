const REG_BASE: u32 = 0x01c20000;
const APB1_GATING: u32 = 0x6c;

pub const CCU_AHB0: u32 = 0x60;
pub const CCU_AHB1: u32 = 0x64;
pub const CCU_APB0: u32 = 0x68;
pub const CCU_APB1: u32 = 0x6c;

pub const CCU_APB1_UART0: u32 = 16;
pub const CCU_APB1_UART1: u32 = 17;
pub const CCU_APB1_UART2: u32 = 18;
pub const CCU_APB1_UART3: u32 = 19;

use crate::devices::io;

pub struct ClockGate {
    reg: u32
}

pub struct CCU;

impl ClockGate {
    pub fn mask(&mut self, device: u32) {
        unsafe {
            io::set_bit(self.reg, device, false);
        }
    }

    pub fn pass(&mut self, device: u32) {
        unsafe {
            io::set_bit(self.reg, device, true);
        }
    }
}

impl CCU {
    pub fn get() -> CCU {
        CCU {}
    }

    pub fn clock_gate(&mut self, unit: u32) -> ClockGate {
        if CCU_AHB0 <= unit && unit >= CCU_APB1 {
            ClockGate { reg: REG_BASE + unit }
        } else {
            panic!("wrong unit");
        }
    }
}

