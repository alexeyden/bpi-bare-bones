#![no_std]
#![no_main]
#![feature(global_asm)]

use core::panic::PanicInfo;

const PB_DAT : u32= 0x01C2090C;

mod uart;

global_asm!(include_str!("boot.S"));

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart::uart_puts("\rPANIC!");
    loop {}
}

fn led_on() {
    unsafe {
        core::ptr::write_volatile(PB_DAT as *mut u32, 0x1000000);
    }
}

fn led_off() {
    unsafe {
        core::ptr::write_volatile(PB_DAT as *mut u32, 0);
    }
}

#[no_mangle]
pub extern "C" fn _main() -> ! {
    uart::uart_init();
    loop {
        uart::uart_puts("\rhello, world! >");
        let x = uart::uart_getc();
        uart::uart_puts(core::str::from_utf8(&[ x as u8, '\n' as u8 ]).unwrap());
        match (x as u8) as char {
            '0' => led_off(),
            '1' => led_on(),
            _ => {}
        }
    }
}

