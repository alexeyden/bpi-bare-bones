const REG_BASE: u32 = 0x01f03400;
const SUNXI_PRCM_BASE: u32 = 0x01f01400;

use crate::devices::io;

pub struct RSB;

impl RSB {
    pub fn get() -> RSB {
        RSB {}
    }

    pub fn init() {
        const APB0_GATE : u32 = 0x028;
        const GATE_PIO : u32 = 1;
        const GATE_RSB : u32 = 1 << 3;

        unsafe {
            io::set_bits(REG_BASE + APB0_GATE, GATE_PIO | GATE_RSB);
        }
    }
}

