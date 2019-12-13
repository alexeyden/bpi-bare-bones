.section ".text.boot"

.global _start

_start:

/* mux uart0 rx/tx pins to uart function */

ldr r3, =0x01C2082C
mov r0, #0x22000000
str r0, [r3]

/* setup stack */

mov r4, #0x24000
mov sp, r4

/* zero out bss */

ldr r0, =__bss_start
ldr r1, =__bss_end
1:
cmp r0, r1
ble 2f
ldr r2, =0
str r2, [r0], #4
b 1b

2:
bl main

_inf_loop:
b       _inf_loop
