use crate::devices::io;

const TWI_BASE: u32 = 0x01C2AC00;

const TWI_SLAVE0:  u32 = 0x00;
const TWI_SLAVE1:  u32 = 0x04;
const TWI_DATA:    u32 = 0x08;
const TWI_CTRL:    u32 = 0x0c;
const TWI_STATUS:  u32 = 0x10;
const TWI_BAUD:    u32 = 0x14;
const TWI_RESET:   u32 = 0x18;

const CTRL_BUS_EN : u32 = 0x00000040;
const CTRL_START  : u32 = 0x00000020;
const CTRL_STOP   : u32 = 0x00000010;
const CTRL_IFLAG  : u32 = 0x00000008;
const CTRL_ACK    : u32 = 0x00000004;

const STATUS_IDLE       : u32 = 0xf8;
const STATUS_ADDR_W_ACK : u32 = 0x18;
const STATUS_DATA_W_ACK : u32 = 0x28;
const STATUS_ADDR_R_ACK : u32 = 0x40;
const STATUS_ADDR_R_NAK : u32 = 0x48;
const STATUS_DATA_R_ACK : u32 = 0x50;
const STATUS_DATA_R_NAK : u32 = 0x58;
const STATUS_REP_START  : u32 = 0x10;
const STATUS_START      : u32 = 0x08;

extern "C" {
    fn _delay(ticks : u32);
}

unsafe fn twi_wait(expect : u32) {
    for _ in 0..10000 {
        if io::read(TWI_BASE + TWI_CTRL) & CTRL_IFLAG  > 0 {
            if io::read(TWI_BASE + TWI_STATUS) == expect {
                break;
            }
        }

        _delay(100);
    }
}

unsafe fn twi_start(expect : u32) {
    io::write(TWI_BASE + TWI_CTRL, CTRL_BUS_EN | CTRL_START);
    twi_wait(expect);
}

unsafe fn twi_stop() {
    io::write(TWI_BASE + TWI_CTRL, CTRL_BUS_EN | CTRL_STOP);
    for _ in 0..10000 {
        if io::read(TWI_BASE + TWI_STATUS) == STATUS_IDLE {
            break;
        }
        _delay(10);
    }
}

unsafe fn twi_send(expect : u32, data : u8) {
    io::write(TWI_BASE + TWI_DATA, data as u32);
    io::write(TWI_BASE + TWI_CTRL, CTRL_BUS_EN);

    twi_wait(expect);
}

unsafe fn twi_recv(ack : bool) -> u8 {
    io::write(TWI_BASE + TWI_CTRL, CTRL_BUS_EN |
              if ack { CTRL_ACK } else { 0 });

    twi_wait(if ack { STATUS_DATA_R_ACK } else { STATUS_DATA_R_NAK });

    return io::read(TWI_BASE + TWI_DATA) as u8;
}

unsafe fn twi_begin(expect : u32, addr : u8) {
    twi_start(expect);

    let st = if addr & 1 > 0 { STATUS_ADDR_R_ACK } else { STATUS_ADDR_W_ACK };
    twi_send(st, addr);
}

pub unsafe fn twi_init() {
    io::write(TWI_BASE + TWI_RESET, 1);
    _delay(1000);
    io::write(TWI_BASE + TWI_RESET, 0);
    _delay(1000);

    io::write(TWI_BASE + TWI_BAUD, 0x44);
    _delay(1000);
    io::write(TWI_BASE + TWI_SLAVE0, 0x7f);
    _delay(1000);
    io::write(TWI_BASE + TWI_SLAVE1, 0);

    twi_stop();
}

pub unsafe fn twi_read(chip : u8, reg : u8) -> u8 {
    twi_begin(STATUS_START, chip << 1);
    twi_send(STATUS_DATA_W_ACK, reg);

    twi_begin(STATUS_REP_START, (chip << 1) | 1);
    let b = twi_recv(false);
    twi_stop();

    return b;
}

pub unsafe fn twi_write(chip : u8, reg : u8, data : u8) {
    twi_begin(STATUS_START, chip << 1);
    twi_send(STATUS_DATA_W_ACK, reg);
    twi_send(STATUS_DATA_W_ACK, data);
    twi_stop();
}

