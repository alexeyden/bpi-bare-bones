// To keep this in the first portion of the binary.
.section ".text.boot"
 
// Make _start global.
.global _start 
_start:
    mov r0, #0x00000001 //Activate_PIN_20_21 content for PH_CFG2
    mov r1, #0x00000000 //Disable_LED content for PH_DAT
    mov r2, #0x01000000 //Enable_LED content for PH_DAT
    ldr r3, =0x01C20908 //PH_CFG2 Address
    str r0, [r3] //This sets the Port H configuration ready for the two LEDs
    ldr r3, =0x01C2090C //PH_DAT Address
    str r2, [r3] //This writes the data to Port H, indicating to switch the Yellow and Blue LED on

    ldr r3, =0x01C2082C //PB_CFG2 Address
    mov r0, #0x22000000
    str r0, [r3] //This sets the Port H configuration ready for the two LEDs

    mov r4, #0x24000
    mov sp, r4
/*
    // We enter execution in supervisor mode. For more information on
    // processor modes see ARM Section A2.2 (Processor Modes)
    // Change to supervisor (SVC) mode anyway
    cps #0x13    
    
    // define stack pointer base
    mov r4, #0x80000000
    // set SVC stack at offset 0x2000
    add sp, r4, #0x2400
*/
    // r0, r1, r2 are arguments for main
    bl      _main

    // If main does return for some reason, just catch it and stay here.
_inf_loop:
    b       _inf_loop
