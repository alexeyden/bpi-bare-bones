use core::panic::PanicInfo;
use crate::devices::uart::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut uart = UART::get(0);
    uart.write_str("\rpanic on the streets of london");
    loop {}
}

