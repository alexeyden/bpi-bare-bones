.section ".text.boot"

.global _start

_start:
ldr pc, _reset_h
ldr pc, _undefined_instruction_vector_h
ldr pc, _software_interrupt_vector_h
ldr pc, _prefetch_abort_vector_h
ldr pc, _data_abort_vector_h
ldr pc, _unused_handler_h
ldr pc, _interrupt_vector_h
ldr pc, _fast_interrupt_vector_h

_reset_h:                           .word   _reset
_undefined_instruction_vector_h:    .word   software_interrupt_vector
_software_interrupt_vector_h:       .word   software_interrupt_vector
_prefetch_abort_vector_h:           .word   software_interrupt_vector
_data_abort_vector_h:               .word   software_interrupt_vector
_unused_handler_h:                  .word   software_interrupt_vector
_interrupt_vector_h:                .word   interrupt_vector
_fast_interrupt_vector_h:           .word   software_interrupt_vector

_putc:
	push {r0,r1,r3, lr}
	ldr r1, =0x01c28000
	str r0, [r1]

	ldr r3, =0x0000ffff
	_putc_lp:
	sub r3, r3, #1
	cmp r3, #0
	blne _putc_lp

	pop {r0,r1,r3, pc}

_delay:
	push {lr}
	_lp:
	sub r0, r0, #1
	cmp r0, #0
	blne _lp
	pop {pc}

_reset:
	/* setup supervisor mode stack at the top of SRAM B 64K module */
	ldr r0, =0x2fff0
	mov sp, r0

	/* IRQ stack at the top of SRAM A3 13K */
	cps #0x12
	ldr r0, =0xb3f0
	mov sp, r0

	cps #0x13

	/* disable high exception vectors (by setting SCTRL.V to 0) */
	ldr r1, =0x139A98
	mcr p15, 0, r1, c1, c0, 0

	/*
	 * disable interrupts (FIQ and IRQ), also set the cpu to SVC32 mode,
	 * except if in HYP mode already
	 */
	mrs	r0, cpsr
	and	r1, r0, #0x1f		@ mask mode bits
	teq	r1, #0x1a		@ test for HYP mode
	bicne	r0, r0, #0x1f		@ clear all mode bits
	orrne	r0, r0, #0x13		@ set SVC mode
	orr	r0, r0, #0xc0		@ disable FIQ and IRQ
	msr	cpsr,r0

	/*
	 * Invalidate L1 I/D
	 */
	mov	r0, #0			@ set up for MCR
	mcr	p15, 0, r0, c8, c7, 0	@ invalidate TLBs
	mcr	p15, 0, r0, c7, c5, 0	@ invalidate icache
	mcr	p15, 0, r0, c7, c5, 6	@ invalidate BP array
	mcr     p15, 0, r0, c7, c10, 4	@ DSB
	mcr     p15, 0, r0, c7, c5, 4	@ ISB

	/*
	 * disable MMU stuff and caches
	 */
	mrc	p15, 0, r0, c1, c0, 0
	bic	r0, r0, #0x00002000	@ clear bits 13 (--V-)
	bic	r0, r0, #0x00000007	@ clear bits 2:0 (-CAM)
	orr	r0, r0, #0x00000002	@ set bit 1 (--A-) Align
	orr	r0, r0, #0x00000800	@ set bit 11 (Z---) BTB
	bic	r0, r0, #0x00001000	@ clear bit 12 (I) I-cache
	mcr	p15, 0, r0, c1, c0, 0

	bl _main

_inf_loop:
	ldr r0, =33
	bl _putc
	b _inf_loop

interrupt_vector:
	stmfd sp!,{r0-r12, lr}

	/* align stack to 8 bytes */
	mov r4, sp
	bic sp, sp, #7

	/* call into the high level handler */
	bl handle_irq
	mov sp, r4

	ldmfd sp!,{r0-r12, pc}^

software_interrupt_vector:
	stmfd sp!,{r0-r12, lr}

	/* align stack to 8 bytes */
	mov r4, sp
	bic sp, sp, #7

	/* call high level handler */
	bl handle_swi
	mov sp, r4

	ldmfd sp!,{r0-r12, pc}^

