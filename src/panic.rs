use core::panic::PanicInfo;
use crate::drivers::uart;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart::uart_puts("\rPANIC!");
    loop {}
}

