#![no_std]
#![no_main]
#![feature(global_asm)]
#![allow(dead_code)]
#![feature(const_fn)]

mod panic;
mod devices;

use core::slice;
use core::str;

use devices::uart::*;
use devices::ccu::*;
use devices::gpio::*;

const LED_PIN: u32 = 24;

global_asm!(include_str!("boot.S"));

struct Main {
    ccu: CCU,
    console: UART,
    pio_ph: GPIO
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
            "led on" => self.pio_ph.set_high(LED_PIN),
            "led off" => self.pio_ph.set_low(LED_PIN),
            _ => self.console.write_str("\n\runknown cmd")
        }
    }
}

#[no_mangle]
pub extern "C" fn _main() -> ! {
    let mut main = Main::new();
    main.run();
}

