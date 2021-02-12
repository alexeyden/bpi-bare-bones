const REG_BASE: u32 = 0x01c20000;
const APB1_GATING: u32 = 0x6c;

pub const CCU_PLL1_CFG: u32 = 0x00;
pub const CCU_PLL5_CFG: u32 = 0x20;
pub const CCU_PLL6_CFG: u32 = 0x28;
pub const CCU_MBUS_CLK_CFG: u32 = 0x15c;

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
use crate::devices::uart::*;

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
    const CLK : u32 = 432;
    const MBUS_CLK : u32 = 300;

    pub fn get() -> CCU {
        CCU {}
    }

    pub fn clock_gate(&mut self, unit: u32) -> ClockGate {
        ClockGate { reg: REG_BASE + unit }
    }

    fn set_dram_clock(&mut self) {
        unsafe {
            let mut v = io::read(REG_BASE + CCU_PLL5_CFG);
            let mut uart = UART::get(0);
        
            io::write(REG_BASE + CCU_PLL6_CFG, 0xa1009911);

            self.clock_gate(CCU_AHB0).pass(6);

            v &= !(0x03 << 0);  // M = 0
            v &= !(0x03 << 4);  // K = 0
            v &= !(0x1f << 8);  // N = 0
            v &= !(0x03 << 16); // P = 0

            v |= 1 << 0;      // M = 2
            v |= 1 << 4;      // K = 2
            v |= (CCU::CLK/24) << 8; // N = 360/24

            v &= !(1 << 19); // disable VCO gain
            v |= 1 << 31; // enable PLL5

            io::write(REG_BASE + CCU_PLL5_CFG, v);
            _delay(2_000_000);

            /* enable PLL5 output clock */
            io::set_bit(REG_BASE + CCU_PLL5_CFG, 29, true);

            /* GPS RESET */
            io::clrset_bits(REG_BASE + 0xd0, 3, 0);
            self.clock_gate(CCU_AHB0).pass(26);
            _delay(0xfffff);
            self.clock_gate(CCU_AHB0).mask(26);

            let x = io::read(REG_BASE + CCU_PLL6_CFG);

            let n = (x >> 8) & 0x1f;
            let k = ((x >> 4) & 0x03) + 1;
            let x = 24 * n * k / 2;
            let x = (x * 2 + CCU::MBUS_CLK - 1)/CCU::MBUS_CLK;

            let v =
                (1 << 31) | // enable MBUS gating
                (1 << 24) | // source clock from PLL6
//              (0 << 16) | // N = 1
                (x - 1);  // M = x

            io::write(REG_BASE + CCU_MBUS_CLK_CFG, v);

            io::clrset_bits(REG_BASE + CCU_AHB0, 0, 1 << 25);
            _delay(0xfffff);

            io::clrset_bits(REG_BASE + CCU_AHB0, 1<<14 | 1<<15, 0);
            _delay(0xfffff);

            io::clrset_bits(REG_BASE + CCU_AHB0, 0, 1<<14 | 1<<15);
            _delay(0xfffff);
        }
    }

    pub unsafe fn dram_init(&mut self) {
        const DRAMC_BASE: u32 = 0x01c01000;
        const MCTL: u32 = 0x230;
        const DCR : u32 = 0x004;
        const DLLCR: u32 = 0x204;

        const TPR3: u32 = 0;
        const BUS_WIDTH: u32 = 32;
        const IO_WIDTH: u32 = 16;
        const DENSITY: u32 = 4; // 1024 = 2; 8192 = 5

        let mut uart = UART::get(0);

        self.set_dram_clock();

        /* disable_power_save */
        io::write(DRAMC_BASE + 0x23c, 0x16510000);

        /* mctl_set_drive() */
        io::clrset_bits(DRAMC_BASE + MCTL, 0x03 | 0x03 << 28, 0x03 << 13 | 0xffc);

        /* clock output disable */
        io::set_bit(DRAMC_BASE + MCTL, 16, false);

        /* ITM disable */
        io::clrset_bits(DRAMC_BASE, 1 << 31, 1 << 28);

        /* DLL control */
        io::clrset_bits(DRAMC_BASE + DLLCR, 0x3f << 6, ((TPR3 >> 16) & 0x3f) << 6);
        io::clrset_bits(DRAMC_BASE + DLLCR, 1 << 30, 1 << 31);
        _delay(0xfffff);

        io::clrset_bits(DRAMC_BASE + DLLCR, 1 << 31 | 1 << 30, 0);
        _delay(0xfffff);

        io::clrset_bits(DRAMC_BASE + DLLCR, 1 << 31, 1 << 30);
        _delay(0xfffff);

        /* configure external DRAM */
        let v =
            (1 << 0) | // DDR3
            ((IO_WIDTH >> 3) << 1) | // IO width = 16
            (DENSITY << 3)  | // density = 8192
            (3 << 6)  | // bus width = 32
            (1 << 12) | // rank all
            (1 << 13); // interleave

        io::write(DRAMC_BASE + 0x04, v);

        /* enable clock output */
        io::set_bit(DRAMC_BASE + MCTL, 16, true);

        /* set impendance */
        _delay(0xfffff);

        /* set initialization delay */
        io::clrset_bits(DRAMC_BASE + 0xb4, 0, 0x1ffff);

        /* reset ddr3 */
        io::clrset_bits(DRAMC_BASE + MCTL, 1 << 12, 0);
        _delay(0xfffff);
        io::clrset_bits(DRAMC_BASE + MCTL, 0, 1 << 12);
        _delay(0xfffff);

        while io::read(DRAMC_BASE) & (1<<31) != 0 {}

        /* DLL */

        let nlanes : u32 = if BUS_WIDTH == 32 { 4 } else { 2 };
        let mut phase : u32 = TPR3;

        for i in 1..=nlanes {
            let r = DRAMC_BASE + DLLCR + i * 4;
            io::clrset_bits(r, 0xf << 14, (phase & 0xf) << 14);
            io::clrset_bits(r, 1 << 30, 1 << 31);

            phase >>= 4;
        }
        _delay(0xfffff);

        for i in 1..=nlanes {
            let r = DRAMC_BASE + DLLCR + i * 4;
            io::clrset_bits(r, (1 << 30) | (1 << 31), 0);
        }

        _delay(0xfffff);

        for i in 1..=nlanes {
            let r = DRAMC_BASE + DLLCR + i * 4;
            io::clrset_bits(r, 1 << 31, 1 << 30);
        }

        _delay(0xfffff);

        /* set autorefresh cycle */

        let trfc_table : [u32; 6] = [ 90, 90, 110, 160, 300, 350 ];
        let rfc = (trfc_table[DENSITY as usize] * CCU::CLK + 999)/1000;
        let refi = (7987 * CCU::CLK) >> 10;
        io::write(DRAMC_BASE + 0x10, rfc & 0xff | (refi & 0xffff) << 8);

        /* DRAM CLK = 432 */

        /* set timing parameters */
        io::write(DRAMC_BASE + 0x14, 0x42d899b7);
        io::write(DRAMC_BASE + 0x18, 0xa090);
        io::write(DRAMC_BASE + 0x1c, 0x22a00);

        /* mode register */

        let twr_ck = (15 * CCU::CLK + 999) / 1000;
        let wr = if twr_ck < 5 { 1 } else if twr_ck <= 8 { twr_ck - 4 } else if twr_ck <= 10 { 5 } else { 6 };
        let v =
            1 << 12 | // power down
            5 << 4  | // cas = 6
            wr << 9  ; // write recovery

        io::write(DRAMC_BASE + 0x1f0, v);

        /* extended mode register */

        io::write(DRAMC_BASE + 0x1f4, 0x04);
        io::write(DRAMC_BASE + 0x1f8, 0x10);
        io::write(DRAMC_BASE + 0x1fc, 0);

        /* disable drift compensation */
        io::clrset_bits(DRAMC_BASE, 1<<17, 1<<14);

        /* init ext ram */
        io::clrset_bits(DRAMC_BASE, 0, 1<<31);
        while io::read(DRAMC_BASE) & (1<<31) != 0 {}

        /* itm enable */
        io::clrset_bits(DRAMC_BASE, 1<<28, 0);

        /* scan readpipe */
        io::clrset_bits(DRAMC_BASE + 0x0c, (1<<20) | (1<<21), 0);
        io::clrset_bits(DRAMC_BASE, 0, 1 << 30);
        while io::read(DRAMC_BASE) & (1<<30) != 0 {}

        /* DQS gating window type */
        io::clrset_bits(DRAMC_BASE, 0, 1 << 14);

        /* ITM RESET */

        io::clrset_bits(DRAMC_BASE, 1 << 31, 1 << 28);
        _delay(0xfffff);

        io::clrset_bits(DRAMC_BASE, 1 << 28, 0);
        _delay(0xfffff);

        let hpcr: [u32; 32] = [
            0x0301, 0x0301, 0x0301, 0x0301,
            0x0301, 0x0301, 0x0301, 0x0301,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0x1031, 0x1031, 0x0735, 0x1035,
            0x1035, 0x0731, 0x1031, 0x0735,
            0x1035, 0x1031, 0x0731, 0x1035,
            0x0001, 0x1031, 0, 0x1031
        ];

        for i in 0..32 {
            io::write(DRAMC_BASE + 0x250 + i * 4, hpcr[i as usize]);
        }

        /*

        printf!(uart, "RAM initialized, test write...\r\n");

        const FILL_SIZE : u32 = 1024 * 1024 * 1024 / 4;
        const FILL_BASE : u32 = 0x4000_0000;

        for i in 0..FILL_SIZE {
            if i % (1024 * 1024 * 16) == 0 {
                printf!(uart, "\r\nAt 0x%x\r\n", FILL_BASE + i * 4);
            }
            if i % (1024 * 1024) == 0 {
                uart.put_char(b'.');
            }
            io::write(FILL_BASE + i * 4, 0xdeadbeef);
        }

        printf!(uart, "\r\nVerifying...\r\n");

        for i in 0..FILL_SIZE {
            if i % (1024 * 1024 * 16) == 0 {
                printf!(uart, "\r\nAt 0x%x\r\n", FILL_BASE + i * 4);
            }
            if i % (1024 * 1024) == 0 {
                uart.put_char(b'.');
            }

            let v = io::read(FILL_BASE + i * 4);
            if v != 0xdeadbeef {
                printf!(uart, "Test failed: *0x%x = 0x%x\r\n", i as u32, v as u32);
                break;
            }
        }

        printf!(uart, "\r\nOK\r\n");
        */
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

