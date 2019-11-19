const AW_CCM_BASE : u32 = 0x01c20000;
const SUNXI_UART0_BASE : u32 = 0x01C28000;
const APB2_GATE : u32 = AW_CCM_BASE + 0x06C;
const APB2_RESET : u32 = AW_CCM_BASE + 0x2D8;
const APB2_GATE_UART_SHIFT : u32 = 16;
const APB2_RESET_UART_SHIFT : u32 = 16;
const BAUD_115200 : u32 = 0xd; /* 24 * 1000 * 1000 / 16 / 115200 = 13 */
const NO_PARITY : u32 = 0;
const ONE_STOP_BIT : u32 = 0;
const DAT_LEN_8_BITS : u32 = 3;
const LC_8_N_1 : u32 = (NO_PARITY << 3 | ONE_STOP_BIT << 2 | DAT_LEN_8_BITS);
const UART0_RBR : u32 = (SUNXI_UART0_BASE + 0x0);    /* receive buffer register */
const UART0_THR : u32 = (SUNXI_UART0_BASE + 0x0);    /* transmit holding register */
const UART0_DLL : u32 = (SUNXI_UART0_BASE + 0x0);    /* divisor latch low register */
const UART0_LCR : u32 = (SUNXI_UART0_BASE + 0xc);    /* line control register */
const UART0_LSR : u32 = (SUNXI_UART0_BASE + 0x14);   /* line status register */
const UART0_IER : u32 = (SUNXI_UART0_BASE + 0x4);    /* interrupt enable reigster */

pub fn uart_init() {
    unsafe {
        /* init clocks */

        let gate = core::ptr::read_volatile(APB2_GATE as *mut u32);
        core::ptr::write_volatile(APB2_GATE as *mut u32, gate | (1 << APB2_GATE_UART_SHIFT));
        let reset = core::ptr::read_volatile(APB2_RESET as *mut u32);
        core::ptr::write_volatile(APB2_RESET as *mut u32, reset | (1 << APB2_RESET_UART_SHIFT));

        core::ptr::write_volatile(UART0_LCR as *mut u32, 0x80);
        core::ptr::write_volatile(UART0_IER as *mut u32, 0);
        core::ptr::write_volatile(UART0_DLL as *mut u32, BAUD_115200);
        core::ptr::write_volatile(UART0_LCR as *mut u32, LC_8_N_1);
    }
}

pub fn uart_puts(s :&str) {
    for c in s.chars() {
        unsafe {
            while core::ptr::read_volatile(UART0_LSR as *mut u32) & 0x20 == 0 {}
            core::ptr::write_volatile(UART0_THR as *mut u32, c as u32);
        }
    }
}

pub fn uart_getc() -> u32
{
    unsafe {
        while core::ptr::read_volatile(UART0_LSR as *mut u32) & 0x01 == 0 {}
        return core::ptr::read_volatile(UART0_RBR as *mut u32);
    }
}

