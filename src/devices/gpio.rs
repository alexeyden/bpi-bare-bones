use crate::devices::io;

const PIO_BASE: u32 = 0x01c20800;

const fn port_cfg(port: u32, n: u32) -> u32 {
    PIO_BASE + port * 0x24 + n * 4
}

const fn port_dat(port: u32) -> u32 {
    PIO_BASE + port * 0x24 + 0x10
}

const fn port_drv(port: u32, n: u32) -> u32 {
    PIO_BASE + port * 0x24 + 0x14 + n * 4
}

const fn port_pull(port: u32, n: u32) -> u32 {
    PIO_BASE + port * 0x24 + 0x1c + n * 4
}

pub const GPIO_PB: u32 = 1;
pub const GPIO_PH: u32 = 7;

pub const GPIO_IN: u32 = 0;
pub const GPIO_OUT: u32 = 1;
pub const GPIO_UART: u32 = 2;

pub struct GPIO {
    port: u32
}

impl GPIO {
    pub fn get(port: u32) -> GPIO {
        GPIO { port: port }
    }

    pub fn cfg(&mut self, pin: u32, fun: u32) {
        let n = pin >> 3;
        let r = (pin & 7) << 2;
        unsafe {
            let x = io::read(port_cfg(self.port, n));
            let x = x & !(0x0f << r);
            let x = x | (fun << r);
            io::write(port_cfg(self.port, n), x);
        }
    }

    pub fn set_high(&mut self, pin: u32) {
        unsafe {
            let x = io::read(port_dat(self.port));
            io::write(port_dat(self.port), x | (1<<pin));
        }
    }

    pub fn set_low(&mut self, pin: u32) {
        unsafe {
            let x = io::read(port_dat(self.port));
            io::write(port_dat(self.port), x & !(1<<pin));
        }
    }

    pub fn is_high(&mut self, pin: u32) -> bool {
        unsafe {
            io::read(port_dat(self.port)) & (1 << pin) > 0
        }
    }
}

