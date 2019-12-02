use core::ptr::write_volatile;
use core::ptr::read_volatile;

pub unsafe fn write(reg: u32, value: u32) {
    write_volatile(reg as *mut u32, value)
}

pub unsafe fn read(reg: u32) -> u32 {
    read_volatile(reg as *mut u32)
}

pub unsafe fn set_bit(reg: u32, i: u32, v: bool) {
    let val = read(reg);
    match v {
        true => write(reg, val | (1 << i)),
        false => write(reg, val & !(1 << i))
    }
}

pub unsafe fn get_bit(reg: u32, i: u32) -> bool {
    (read(reg) & (1 << i)) != 0
}

