#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(naked_functions)]
#![feature(const_fn)]

mod panic;
mod devices;

use core::slice;
use core::str;

use devices::uart::*;
use devices::ccu::*;
use devices::gpio::*;
use devices::i2c::*;
use devices::io;

const LED_PIN: u32 = 24;

global_asm!(include_str!("boot.S"));

struct Main {
    ccu: CCU,
    console: UART,
    pio_ph: GPIO
}

static mut G_TICKS : u32 = 0;
static mut G_TIMER_EVENT : u32 = 0;

extern "C" {
    #[no_mangle]
    fn _delay(ticks : u32);
}

#[no_mangle]
extern "C" fn handle_swi() {
    let mut uart = UART::get(0);
    uart.write_str("\r\nSWI");
}

#[no_mangle]
extern "C" fn handle_irq() {
    let n = gic_getack();

    timer_read();

    unsafe {
        G_TICKS += 1;

        core::ptr::write_volatile(&mut G_TIMER_EVENT, 1);

        if G_TICKS > 400 {
            let mut ph = GPIO::get(GPIO_PH);

            if ph.is_high(LED_PIN) {
                ph.set_low(LED_PIN);
            } else {
                ph.set_high(LED_PIN);
            }

            G_TICKS = 0;
        }
    }

    gic_eoi(n);
}

fn enable_irq() {
    unsafe {
        asm!(
            "push {{r0}}",
            "mrs r0,cpsr",
            "bic r0,r0,#0x80",
            "msr cpsr_c,r0",
            "pop {{r0}}"
        );
    }
}

fn disable_irq() {
    unsafe {
        asm!(
            "push {{r0}}",
            "mrs     r0, cpsr",
            "orr r0, r0, #0x80",
            "msr     cpsr_c, r0",
            "pop {{r0}}"
        );
    }
}

fn blinker() {
    timer_on(24_000);

    unsafe {
        let x : u32 = io::read(0x01c20c00 + 0x0);
        io::write(0x01c20c00 + 0x0, x | 1);
    }
}

fn timer_on(interval : u32) {
    unsafe {
        io::write(0x01c20c00 + 0x10, 4);

        /* wait a little */
        for _i in 0..0xffff {
            io::read(0x01c20c00 + 0x10);
        }

        io::write(0x01c20c00 + 0x14, interval);
        io::write(0x01c20c00 + 0x10, 7);

        irq_en(54);
    }
}

fn timer_read() -> bool {
    unsafe {
        if io::get_bit(0x01c20c00 + 4, 0) {
            let x = io::read(0x01c20c00 + 4);
            io::write(0x01c20c00 + 4, x | 2);

            return true;
        }
    }

    false
}

fn irq_en(no : u32) {
    unsafe {
        let n = no >> 5;
        let a = 0x01C80000 + 0x1000 + 0x0100 + 4 * n;
        let v = io::read(a);
        io::write(a, v | (1 << (no & 0x1f)));

        io::write8(0x01C80000 + 0x1000 + 0x0800 + no, 1);
    }
}

fn gic_eoi(no : u32) {
    unsafe {
        io::write(0x01C82010, no);
    }
}

fn gic_getack() -> u32 {
    unsafe {
        io::read(0x01C8200C)
    }
}

#[naked]
extern "C" fn call_swi() {
    unsafe {
        asm!("svc #0");
    }
}

impl Main {
    pub fn new() -> Main {
        let mut ccu = CCU::get();
        let mut uart = UART::get(0);
        let mut ph = GPIO::get(GPIO_PH);
        let mut pb = GPIO::get(GPIO_PB);

        /* Mux PB 22,23 to UART0 rx/tx */

        pb.cfg(22, GPIO_UART);
        pb.cfg(23, GPIO_UART);
        pb.cfg(0, GPIO_TWI);
        pb.cfg(1, GPIO_TWI);

        ccu.clock_gate(CCU_APB1).pass(CCU_APB1_UART0);
        ccu.clock_gate(CCU_APB1).pass(CCU_APB1_TWI0);

        uart.set_mode(115200, UART_8N1);
        uart.set_echo(true);

        printf!(uart, "Initialization...\n\r");

        /* Configure PH24 (Green LED) as output */

        ph.cfg(LED_PIN, GPIO_OUT);
        ph.set_low(LED_PIN);

        Main {
            ccu: ccu,
            console: uart,
            pio_ph: ph
        }
    }

    fn print_regs(&mut self) {
        let mut r : [u32; 16] = [ 0; 16 ];
        let names : [&str; 16] = [
            "R00", "R01", "R02", "R03",
            "R04", "R05", "R06", "R07",
            "R08", "R09", "R10", "R11",
            "R12", "SP ", "LR ", "PC "
        ];

        unsafe {
            asm!("str r0,  [{x}]", x = in(reg) &mut r[0]);
            asm!("str r1,  [{x}]", x = in(reg) &mut r[1]);
            asm!("str r2,  [{x}]", x = in(reg) &mut r[2]);
            asm!("str r3,  [{x}]", x = in(reg) &mut r[3]);
            asm!("str r4,  [{x}]", x = in(reg) &mut r[4]);
            asm!("str r5,  [{x}]", x = in(reg) &mut r[5]);
            asm!("str r6,  [{x}]", x = in(reg) &mut r[6]);
            asm!("str r7,  [{x}]", x = in(reg) &mut r[7]);
            asm!("str r8,  [{x}]", x = in(reg) &mut r[8]);
            asm!("str r9,  [{x}]", x = in(reg) &mut r[9]);
            asm!("str r10, [{x}]", x = in(reg) &mut r[10]);
            asm!("str r11, [{x}]", x = in(reg) &mut r[11]);
            asm!("str r12, [{x}]", x = in(reg) &mut r[12]);
            asm!("str r13, [{x}]", x = in(reg) &mut r[13]);
            asm!("str r14, [{x}]", x = in(reg) &mut r[14]);
            asm!("str r15, [{x}]", x = in(reg) &mut r[15]);
        }

        for i in 0..16 {
            printf!(self.console, "% = %x\n\r", names[i], r[i]);
        }
    }

    pub fn run(&mut self) -> ! {
        let mut buf: [u8; 256] = [ 0; 256 ];

        printf!(self.console, "Setting up interrupts...\r\n");

        disable_irq();

        printf!(self.console, "GIC initialization...\n\r");

        unsafe {
            io::write(0x01C80000 + 0x1000 + 0, 1 << 0);
            io::write(0x01C80000 + 0x2000 + 0, 1 << 0);
            io::write(0x01C80000 + 0x2000 + 4, 0xf0);
        }
        printf!(self.console, "Enabling interrupts..\n\r");

        enable_irq();

        printf!(self.console, "Running main loop");

        loop {
            self.console.write_str("\r\n> ");
            let size = self.console.read_str(&mut buf);
            let s = unsafe { slice::from_raw_parts(&buf as *const u8, size) };
            self.on_cmd(str::from_utf8(s).unwrap());
        }
    }

    fn timer_sleep(&self, msec: u32) {
        for _ in 0..msec {
            unsafe {
                while core::ptr::read_volatile(&G_TIMER_EVENT) == 0 {}
                core::ptr::write_volatile(&mut G_TIMER_EVENT, 0);
            }
        }
    }

    fn timer_test(&mut self) {
        let sleep_msec = 2 * 1000;
        timer_on(24_000);

        printf!(self.console, "\r\nWaiting for % msec...", sleep_msec);
        self.timer_sleep(sleep_msec);
        printf!(self.console, "OK\n\r");
    }

    fn delay_test(&mut self) {
        let ticks : u32 = 24_000_000;
        printf!(self.console, "\r\nDelay for % ticks...", ticks);
        unsafe { _delay(ticks); }
    }

    fn power(&mut self) {
        let v = unsafe {
            twi_init();
            twi_read(0x34, 0x03)
        };

        printf!(self.console, "\r\nAXP209 version: %", (v & 0x0f) as u32);
    }

    fn on_cmd(&mut self, line: &str) {
        match line {
            "led on"  => self.pio_ph.set_high(LED_PIN),
            "led off" => self.pio_ph.set_low(LED_PIN),
            "swi"     => call_swi(),
            "reg"     => self.print_regs(),
            "wait"    => self.timer_test(),
            "reclock" => self.ccu.set_cpu_1500mhz(),
            "delay"   => self.delay_test(),
            "blink"   => blinker(),
            "power"   => self.power(),
            _         => self.console.write_str("\n\runknown cmd")
        }
    }
}

#[no_mangle]
pub extern "C" fn _main() -> ! {
    let mut main = Main::new();
    main.run();
}

