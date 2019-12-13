#include <string.h>

#define AW_CCM_BASE    0x01c20000
#define SUNXI_UART0_BASE  0x01C28000

#define APB2_GATE    (AW_CCM_BASE + 0x06C)
#define APB2_GATE_UART_SHIFT  (16)

#define UART0_RBR (SUNXI_UART0_BASE + 0x0)    /* receive buffer register */
#define UART0_THR (SUNXI_UART0_BASE + 0x0)    /* transmit holding register */
#define UART0_DLL (SUNXI_UART0_BASE + 0x0)    /* divisor latch low register */

#define UART0_IER (SUNXI_UART0_BASE + 0x4)    /* interrupt enable reigster */

#define UART0_LCR (SUNXI_UART0_BASE + 0xc)    /* line control register */
#define UART0_LSR (SUNXI_UART0_BASE + 0x14)   /* line status register */

#define BAUD_115200    (0xd) /* 24 * 1000 * 1000 / 16 / 115200 = 13 */
#define NO_PARITY      (0)
#define ONE_STOP_BIT   (0)
#define DAT_LEN_8_BITS (3)
#define LC_8_N_1       (NO_PARITY << 3 | ONE_STOP_BIT << 2 | DAT_LEN_8_BITS)

#define mmio_read32(addr)         (*((volatile unsigned long  *)(addr)))
#define mmio_write32(addr, v)     (*((volatile unsigned long  *)(addr)) = (unsigned long)(v))
#define mmio_write32or(addr, v)   (*((volatile unsigned long  *)(addr)) |= (unsigned long)(v))

#define EGON_HEADER_SIZE 96
#define RELOC_ADDR 0x8000

static void uart_init(void) {
	mmio_write32or(APB2_GATE, 1 << (APB2_GATE_UART_SHIFT));

	mmio_write32(UART0_LCR, 0x80);
	mmio_write32(UART0_IER, 0);
	mmio_write32(UART0_DLL, BAUD_115200);
	mmio_write32(UART0_LCR, LC_8_N_1);
}

static void putc(char c) {
	while ((mmio_read32(UART0_LSR) & 0x20) == 0) continue;
	mmio_write32(UART0_THR, c);
}

static void puts(const char *s) {
	while (*s) {
		if (*s == '\n')
			putc('\r');
		putc(*s++);
	}
}

static unsigned getc() {
	while(!(mmio_read32(UART0_LSR) & 0x01));
	return mmio_read32(UART0_RBR);
}

static unsigned hex(char c) {
	if(c >= '0' && c <= '9')
		return c - '0';
	
	return 10 + c - 'A';
}

char hexchr(unsigned c) {
	if(c >= 10) return (c - 10) + 'a';
	return c + '0';
}

static void phex32(unsigned long v) {
	putc(hexchr((v >> 28) & 0xf));
	putc(hexchr((v >> 24) & 0xf));
	putc(hexchr((v >> 20) & 0xf));
	putc(hexchr((v >> 16) & 0xf));
	putc(hexchr((v >> 12) & 0xf));
	putc(hexchr((v >>  8) & 0xf));
	putc(hexchr((v >>  4) & 0xf));
	putc(hexchr((v >>  0) & 0xf));
}

static unsigned long read32() {
	unsigned long v;
	v  = ( hex(getc()) << 28 );
	v |= ( hex(getc()) << 24 );
	v |= ( hex(getc()) << 20 );
	v |= ( hex(getc()) << 16 );
	v |= ( hex(getc()) << 12 );
	v |= ( hex(getc()) <<  8 );
	v |= ( hex(getc()) <<  4 );
	v |= ( hex(getc()) <<  0 );
	return v;
}

void loader(void) {
	puts("Waiting for image in SREC32 format...\n");

	unsigned rec_count = 0;
	unsigned char *rec_addr = 0;
	unsigned long rec_csum = 0;

rec_start:
	{
		if(getc() != 'S') {
			puts("error: unexpected expected record start symbol\n");
			goto rec_start;
		}

		goto rec_type;
	}
rec_type:
	{
		int r = getc() - '0';

		if(r < 0 || r > 9) {
			puts("error: unexpected record type\n");
			goto rec_start;
		}

		rec_count  = hex(getc()) << 4;
		rec_count |= hex(getc());

		rec_csum = rec_count;

		if(r == 3)
			goto rec_s3_addr;
		else if(r == 0)
			goto rec_s0_ignore;
		else if(r == 7)
			goto rec_s7_addr;

		puts("error: unsupported record type\n");

		goto rec_start;
	}
rec_s0_ignore:
	{
		while(rec_count--) {
			getc(); getc();
		}

		getc();
		getc();

		goto rec_start;
	}
rec_s3_addr:
	{
		rec_count -= 5;

		rec_addr = (void *) read32();

		rec_csum += ((unsigned long) rec_addr >> 24) & 0xff;
		rec_csum += ((unsigned long) rec_addr >> 16) & 0xff;
		rec_csum += ((unsigned long) rec_addr >> 8) & 0xff;
		rec_csum += ((unsigned long) rec_addr >> 0) & 0xff;

		goto rec_s3_data;
	}
rec_s3_data:
	{
		*rec_addr = hex(getc()) << 4;
		*rec_addr |= hex(getc());
		rec_csum += *rec_addr;
		rec_addr++;

		if(--rec_count)
			goto rec_s3_data;

		unsigned char cs = hex(getc()) << 4;
		cs |= hex(getc());
		unsigned char rec_cs = ~(rec_csum & 0xff);

		if(rec_cs != cs)
			putc('!');
		else
			putc('.');

		getc();
		getc();

		goto rec_start;
	}
rec_s7_addr:
	{
		rec_addr = (void *) read32();

		/* ignore csum and line feed */
		getc(); getc();
		getc(); getc();

		puts("\n");
		puts("Jumping to code at 0x");
		phex32((unsigned long) rec_addr);
		puts("\n");
		puts("\n");
		
		goto trampoline;
	}
trampoline:
	asm("bx %0" :: "r" (rec_addr));
}

extern int __start[];
extern int __end[];

void main(void)
{
	uart_init();

	puts("UART chain loader is running\n");

	const size_t reloc_from = (size_t) &__start + EGON_HEADER_SIZE;
	const size_t reloc_to   = RELOC_ADDR;
	const size_t reloc_size = (size_t) &__end - reloc_from;
	const size_t loader_start = (size_t) &loader - (size_t) &__start;
	void (*reloc_loader)(void) = (void *) (reloc_to + loader_start);

	memcpy((void *) reloc_to, (void*) reloc_from, reloc_size);
	reloc_loader();
}

