use crate::devices::io;

const TWI_BASE: u32 = 0x01C2AC00;

const TWI_SLAVE: u32 = 0;
const TWI_DATA: u32 = 4;
const TWI_CTRL: u32 = 8;
const TWI_BAUD: u32 = 12;
const TWI_STATUS: u32 = 12;
const TWI_RESET: u32 = 24;

extern "C" {
    #[no_mangle]
    fn _delay(ticks : u32);
}

unsafe fn twi_stop() {

}

pub unsafe fn twi_init() {
    io::write(TWI_BASE + TWI_RESET, 0);
    _delay(100);
    io::write(TWI_BASE + TWI_CTRL, );
}

