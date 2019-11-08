#include "uart.h"
#include "macro.h"

void led_on(void) {
	mmio_write32(0x01C2090C, 0x1000000);
}

void led_off(void) {
	mmio_write32(0x01C2090C, 0);
}

void delay() {
	volatile unsigned i;
	for(i = 0x02ffff; i > 0; --i);
}

void main(unsigned int r0, unsigned int r1, unsigned int r2)
{
	(void) r0;
	(void) r1;
	(void) r2;

	uart_init();
	uart_puts("uart_init done\n");

	uart_puts("echo console test\n");
	while(1) {
		char c = uart_getc();
		if(c == '1') {
			led_on();
			uart_puts("led on\n");
		} else if(c == '0') {
			led_off();
			uart_puts("led off\n");
		} else {
			uart_puts("not 0/1\n");
		}
	}
}

