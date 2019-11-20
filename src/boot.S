.section ".text.boot"

.set PB_CFG2, 0x01C2082C
.set PH_CFG3, 0x01C20908
.set PH_DAT,  0x01C2090C

.set PB_SELECT_UART0, 0x22000000
.set PH24_SELECT_OUTPUT, 0x00000001
.set PH24_ON, 0x01000000

.global _start

_start:
	/* configure PH24 as output */

	mov r0, #PH24_SELECT_OUTPUT
	ldr r3, =PH_CFG3
	str r0, [r3]

	/* drive PH24 high */

	mov r0, #PH24_ON
	ldr r3, =PH_DAT
	str r0, [r3]

	/* configure PB22-23 as UART0 rx/tx */

	ldr r3, =PB_CFG2
	mov r0, #PB_SELECT_UART0
	str r0, [r3]

	/* setup stack at 0x2_4000 on SRAM B 64K module */

	mov r4, #0x24000
	mov sp, r4

	bl _main

_inf_loop:
	b _inf_loop
