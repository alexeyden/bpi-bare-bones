use core::panic::PanicInfo;
use crate::devices::uart::*;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut uart = UART::get(0);
    uart.write_str("\r\n!!! PANIC: ");
    uart.write_str(info.payload().downcast_ref::<&str>().unwrap());
    uart.write_str("\r\n");
    loop {}
}

