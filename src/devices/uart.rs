const REG_BASE : u32 = 0x01c28000;
const REG_SIZE : u32 = 0x400;

const RBR : u32 = 0x00;
const THR : u32 = 0x00;
const DLL : u32 = 0x00;
const IER : u32 = 0x04;
const LCR : u32 = 0x0c;
const LSR : u32 = 0x14;

const LCR_DLAB : u32 = 7;
const LSR_THRE : u32 = 5;
const LSR_DR   : u32 = 0;

const NO_PARITY : u32 = 0 << 3;
const STOP_1    : u32 = 0 << 2;
const DATA_8    : u32 = 3 << 0;

pub const UART_8N1 : u32 = ( NO_PARITY | STOP_1 | DATA_8 );

use crate::devices::io;

pub struct UART {
    base: u32,
    local_echo: bool
}

pub enum FormatArg<'a> {
    I32(i32),
    U32(u32),
    Str(&'a str)
}

impl From<u32> for FormatArg<'_> {
    fn from(item: u32) -> Self {
        FormatArg::U32(item)
    }
}

impl From<i32> for FormatArg<'_> {
    fn from(item: i32) -> Self {
        FormatArg::I32(item)
    }
}

impl<'a> From<&'a str> for FormatArg<'a> {
    fn from(item: &'a str) -> Self {
        FormatArg::Str(item)
    }
}

#[macro_export]
macro_rules! printf {
    ($u:expr, $msg:expr) => {{
        $u.printf($msg, &[]);
    }};
    ($u:expr, $msg:expr, $($val:expr),*) => {{
        $u.printf($msg, &[ $( FormatArg::from($val), )* ]);
    }};
}

impl UART {
    pub fn get(index: u32) -> UART {
        UART {
            base: REG_BASE + index * REG_SIZE,
            local_echo: false
        }
    }

    pub fn set_mode(&mut self, baud: u32, mode: u32) {
        const SYS_FREQ: u32 = 24;

        unsafe {
            io::set_bit(self.base + LCR, LCR_DLAB, true);
            io::write(self.base + IER, 0);
            io::write(self.base + DLL, SYS_FREQ * 1000 * 1000 / 16 / baud);
            io::write(self.base + LCR, mode);
        }
    }

    pub fn set_echo(&mut self, en: bool) {
        self.local_echo = en;
    }

    pub fn put_char(&mut self, ch: u8) {
        unsafe {
            while !io::get_bit(self.base + LSR, LSR_THRE) {}
            io::write(self.base + THR, ch as u32);
        }
    }

    pub fn get_char(&mut self) -> u8 {
        let ch = unsafe {
            while !io::get_bit(self.base + LSR, LSR_DR) {}
            io::read(self.base + RBR) as u8
        };

        if self.local_echo {
            self.put_char(ch)
        }

        ch
    }

    pub fn read_str(&mut self, buf: &mut [u8]) -> usize {
        let mut len = 0_usize;

        loop {
            if len == buf.len() {
                break;
            }

            let ch = self.get_char() as char;

            if ch == '\n' || ch == '\r' {
                break;
            }

            buf[len] = ch as u8;

            len += 1;
        }

        len
    }

    pub fn write_str(&mut self, s: &str) {
        for ch in s.bytes() {
            self.put_char(ch);
        }
    }

    pub fn write_u32(&mut self, val: u32) {
        let mut n = 1000000000;

        while n > 0 {
            let x = ((val / n) % 10) as u8;

            if x > 0 || n == 1 {
                self.put_char('0' as u8 + x);
            }

            n = n / 10;
        }
    }

    pub fn write_i32(&mut self, val: i32) {
        if val < 0 {
            self.put_char('-' as u8);
        }
        self.write_u32(val.abs() as u32);
    }

    pub fn printf(&mut self, fmt: &str, args: &[FormatArg]) {
        let mut chars = fmt.chars().peekable();
        let mut args = args.iter();

        while let Some(c) = chars.next() {
            match c {
                '%' => {
                    if let Some('%') = chars.peek() {
                        self.put_char('%' as u8);
                        chars.next();
                        continue;
                    }

                    match args.next() {
                        Some(FormatArg::I32(x)) => self.write_i32(*x),
                        Some(FormatArg::U32(x)) => self.write_u32(*x),
                        Some(FormatArg::Str(x)) => self.write_str(x),
                        None => {}
                    }
                },
                _ => {
                    self.put_char(c as u8);
                }
            }
        }
    }
}

