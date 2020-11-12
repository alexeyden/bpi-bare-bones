const REG_BASE: u32 = 0x01c20000;
const APB1_GATING: u32 = 0x6c;

pub const CCU_PLL1_CFG: u32 = 0x00;
pub const CCU_CPU_AHB_APB0_CFG: u32 = 0x54;
pub const CCU_AHB0: u32 = 0x60;
pub const CCU_AHB1: u32 = 0x64;
pub const CCU_APB0: u32 = 0x68;
pub const CCU_APB1: u32 = 0x6c;

pub const CCU_APB1_UART0: u32 = 16;
pub const CCU_APB1_UART1: u32 = 17;
pub const CCU_APB1_UART2: u32 = 18;
pub const CCU_APB1_UART3: u32 = 19;

pub const CCU_APB1_TWI3:  u32 = 3;
pub const CCU_APB1_TWI2:  u32 = 2;
pub const CCU_APB1_TWI1:  u32 = 1;
pub const CCU_APB1_TWI0:  u32 = 0;

pub const CCU_CPU_CLOCK_SRC_PLL1: u32 = 2;

use crate::devices::io;

extern "C" {
    fn _delay(ticks : u32);
}

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

    pub fn set_cpu_1500mhz(&mut self) {
        let cfg_pll1 : u32 =
            1 << 31  | // ENABLE = 1
            19 << 8  | // N = 31
            1 << 4   | // K = 1
            8 << 26  | // VCO_BIAS = 8
            16 << 20 | // BIAS_CUR = 16
            2 << 13  ; // LOCK_TMR = 2

        let cfg_cpu_1500 : u32 =
            2 << 0 | // axi = 3
            1 << 4 | // ahb = 1
            1 << 8 ; // pb0 = 1

        let cfg_cpu_24 : u32 =
            1 << 0 |
            2 << 4 |
            1 << 8;

        let cfg_cpu_osc24m : u32 = 1 << 16;
        let cfg_cpu_pll1   : u32 = 2 << 16;

        unsafe {
            io::write(REG_BASE + CCU_CPU_AHB_APB0_CFG, cfg_cpu_24 | cfg_cpu_osc24m);

            _delay(100);

            io::write(REG_BASE + CCU_CPU_AHB_APB0_CFG, cfg_cpu_1500 | cfg_cpu_osc24m);
            io::write(REG_BASE + CCU_PLL1_CFG, cfg_pll1);

            _delay(100);

            io::write(REG_BASE + CCU_CPU_AHB_APB0_CFG, cfg_cpu_1500 | cfg_cpu_pll1);

            _delay(100);
        }
    }
}

