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
use devices::io;

const LED_PIN: u32 = 24;

global_asm!(include_str!("boot.S"));

struct Main {
    ccu: CCU,
    console: UART,
    pio_ph: GPIO
}

#[no_mangle]
extern "C" fn handle_swi() {
    let mut uart = UART::get(0);
    uart.write_str("\r\nSWI");
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

        ccu.clock_gate(CCU_APB1).pass(CCU_APB1_UART0);

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

    fn on_cmd(&mut self, line: &str) {
        match line {
            "led on"  => self.pio_ph.set_high(LED_PIN),
            "led off" => self.pio_ph.set_low(LED_PIN),
            "swi"     => call_swi(),
            _         => self.console.write_str("\n\runknown cmd")
        }
    }
}

#[no_mangle]
pub extern "C" fn _main() -> ! {
    let mut main = Main::new();
    main.run();
}

