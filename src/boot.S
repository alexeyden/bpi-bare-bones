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
_undefined_instruction_vector_h:    .word   _inf_loop
_software_interrupt_vector_h:       .word   software_interrupt_vector
_prefetch_abort_vector_h:           .word   _inf_loop
_data_abort_vector_h:               .word   _inf_loop
_unused_handler_h:                  .word   _reset
_interrupt_vector_h:                .word   interrupt_vector
_fast_interrupt_vector_h:           .word   _reset

/*
_putc:
	push {r0,r1,r3, lr}
	ldr r1, =0x01c28000
	str r0, [r1]

	ldr r3, =0x0000ffff
	_lp:
	sub r3, r3, #1
	cmp r3, #0
	blne _lp

	pop {r0,r1,r3, pc}
*/

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

	bl _main

_inf_loop:
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

