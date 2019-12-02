.section ".text.boot"

.global _start

_start:
	/* setup stack at the top of SRAM B 64K module */

	ldr r0, =0x2fffc
	mov sp, r0

	bl _main

_inf_loop:
	b _inf_loop
