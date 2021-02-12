#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(naked_functions)]
#![feature(const_fn)]

mod panic;
mod devices;

//use core::slice;
//use core::str;

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

extern "C" {
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


        if G_TICKS % 500 == 0 {
            let mut ph = GPIO::get(GPIO_PH);

            if ph.is_high(LED_PIN) {
                ph.set_low(LED_PIN);
            } else {
                ph.set_high(LED_PIN);
            }
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
unsafe extern "C" fn call_swi() {
    asm!("svc #0", options(noreturn));
}

unsafe fn fixup() {
}

struct LoaderStateData {
    addr: u32,
    count: u32,
    csum: u32
}

enum LoaderState {
    Start,
    Type,
    S0Ignore,
    S3Addr,
    S3Data,
    S7Addr,
    Failure
}

fn load_srec(uart: &mut UART, base: *mut u8) -> (LoaderState, LoaderStateData) {
    let mut data = LoaderStateData { addr: 0, count: 0, csum: 0 };
    let mut state = LoaderState::Start;

    uart.set_echo(false);

    fn read4(serial : &mut UART) -> u8 {
        let c = serial.get_char();

        if (b'0'..=b'9').contains(&c) {
            c - b'0'
        } else {
            10 + c - b'A'
        }
    }

    fn read8(serial : &mut UART) -> u8 {
        (read4(serial) << 4) | read4(serial)
    }

    fn read32(serial : &mut UART) -> u32 {
        (read8(serial) as u32) << 24 |
        (read8(serial) as u32) << 16 |
        (read8(serial) as u32) <<  8 |
         read8(serial) as u32
    }

    loop {
        match state {
            LoaderState::Start => {
                let ch = uart.get_char();
                if ch != b'S' {
                    printf!(uart, "error: expected start %\r\n", ch as char);
                    state = LoaderState::Failure;
                    continue;
                }

                state = LoaderState::Type;
            },
            LoaderState::Type => {
                let r = uart.get_char();

                if !(b'0'..=b'9').contains(&r) {
                    uart.write_str("error: unexpected record type\r\n");
                    state = LoaderState::Failure;
                    continue;
                }

                data.count = read8(uart) as u32;
                data.csum = data.count;

                state = match r {
                    b'0' => LoaderState::S0Ignore,
                    b'3' => LoaderState::S3Addr,
                    b'7' => LoaderState::S7Addr,
                    _    => LoaderState::Failure
                };
            },
            LoaderState::S0Ignore => {
                while data.count > 0 {
                    uart.get_char();
                    uart.get_char();

                    data.count -= 1;
                }

                uart.get_char();
                uart.get_char();

                state = LoaderState::Start;
            },
            LoaderState::S3Addr => {
                data.count -= 5;
                data.addr = read32(uart);

                data.csum += (data.addr >> 24) & 0xff;
                data.csum += (data.addr >> 16) & 0xff;
                data.csum += (data.addr >>  8) & 0xff;
                data.csum += data.addr & 0xff;

                state = LoaderState::S3Data;
            },
            LoaderState::S3Data => {
                let v = read8(uart);

                data.count -= 1;
                data.csum += v as u32;
                data.addr += 1;

                unsafe {
                    *(base.offset(data.addr as isize)) = v;
                }

                if data.count > 0 {
                    continue;
                }

                let cs0 = read8(uart);
                let cs1 = !(data.csum & 0xff) as u8;

                uart.put_char(if cs0 == cs1 { b'.'} else { b'!' });

                uart.get_char();
                uart.get_char();

                state = LoaderState::Start;
            },
            LoaderState::S7Addr => {
                read32(uart);
                read8(uart);
                uart.get_char();
                uart.get_char();
                break;
            },
            LoaderState::Failure => {
                break;
            }
        }
    }

    uart.set_echo(true);

    (state, data)
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
            ccu,
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
        //let mut buf: [u8; 256] = [ 0; 256 ];

        printf!(self.console, "Setting up interrupts...\r\n");

        disable_irq();

        /*
        printf!(self.console, "GIC initialization...\n\r");

        unsafe {
            io::write(0x01C80000 + 0x1000 + 0, 1 << 0);
            io::write(0x01C80000 + 0x2000 + 0, 1 << 0);
            io::write(0x01C80000 + 0x2000 + 4, 0xf0);
        }
        printf!(self.console, "Enabling interrupts..\n\r");
        */

        //enable_irq();

        self.power();
        self.ccu.set_cpu_1500mhz();
        unsafe { self.ccu.dram_init(); }

        const KERNEL_BASE  : u32 = 0x4200_0000;
        const DTB_BASE     : u32 = 0x4800_0000;
        const INITRAM_BASE : u32 = 0x4820_0000;

        printf!(self.console, "\r\nLoading kernel at 0x%x", KERNEL_BASE);
        load_srec(&mut self.console, KERNEL_BASE as *mut u8);
        printf!(self.console, "OK");

        printf!(self.console, "\r\nLoading dtb at 0x%x", DTB_BASE);
        load_srec(&mut self.console, DTB_BASE as *mut u8);
        printf!(self.console, "OK");

        printf!(self.console, "\r\nLoading initramfs at 0x%x", INITRAM_BASE);
        load_srec(&mut self.console, INITRAM_BASE as *mut u8);
        printf!(self.console, "OK");

        unsafe {
            asm!(
                "mov r0, #0",
                "mov r1, #0",
                "ldr r2, ={x}",
                "ldr r3, ={y}",
                "b r3",
                x = const DTB_BASE,
                y = const KERNEL_BASE);
        }

        loop {
            unsafe { asm!("wfi"); }
        }

        /*
        loop {
            self.console.write_str("\r\n> ");
            let size = self.console.read_str(&mut buf);
            let s = unsafe { slice::from_raw_parts(&buf as *const u8, size) };
            self.on_cmd(str::from_utf8(s).unwrap());
        }
        */
    }

    fn msleep(&self, msec: u32) {
        unsafe {
            let ticks = core::ptr::read_volatile(&G_TICKS);
            while core::ptr::read_volatile(&G_TICKS) < ticks + msec {}
        }
    }

    fn timer_test(&mut self) {
        let sleep_msec = 2 * 1000;
        timer_on(24_000);

        printf!(self.console, "\r\nWaiting for % msec...", sleep_msec);
        self.msleep(sleep_msec);
        printf!(self.console, "OK\n\r");
    }

    fn power(&mut self) {
        let axp209 = 0x34;

        unsafe {
            twi_init();
        };

        unsafe {
            /* Enable DCDC2 and set the voltage to 1.4V */

            let dcdc2 : u32 = (1400 - 700)/25;
            twi_write(axp209, 0x23, dcdc2 as u8);

            let v = twi_read(axp209, 0x12);
            twi_write(axp209, 0x12, v | (1 << 4));

            /* Enable DCDC3 and set the voltage to 1.25V */

            let dcdc3 : u32= (1250 - 700)/25;
            twi_write(axp209, 0x27, dcdc3 as u8);

            let v = twi_read(axp209, 0x12);
            twi_write(axp209, 0x12, v | (1 << 1));

            /* Enable LDO2 and set the voltage to 3.0V */

            let ldo2 : u32= (3000 - 1800)/100;
            let v = twi_read(axp209, 0x28);
            twi_write(axp209, 0x28, ((v & 0x0f) | (ldo2 as u8 & 0xf0)) as u8);

            let v = twi_read(axp209, 0x12);
            twi_write(axp209, 0x12, v | (1 << 2));

            /* Turn off LDO3 and LDO4 */

            let v = twi_read(axp209, 0x12) & !(1<<6 | 1<<3);
            twi_write(axp209, 0x12, v);
        }
    }

    fn on_cmd(&mut self, line: &str) {
        match line {
            "led on"  => self.pio_ph.set_high(LED_PIN),
            "led off" => self.pio_ph.set_low(LED_PIN),
            "swi"     => unsafe { call_swi() },
            "reg"     => self.print_regs(),
            "wait"    => self.timer_test(),
            "reclock" => self.ccu.set_cpu_1500mhz(),
            "blink"   => blinker(),
            "power"   => self.power(),
            "dram"    => unsafe { self.ccu.dram_init() },
            _         => self.console.write_str("\n\runknown cmd")
        }
    }
}

#[no_mangle]
pub extern "C" fn _main() -> ! {
    let mut main = Main::new();
    main.run();
}

