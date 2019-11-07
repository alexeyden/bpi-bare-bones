
.syntax unified

.text
.global start // make entry visible for linker

start:
    mov r0, 0x00000001 //Activate_PIN_20_21 content for PH_CFG2
    mov r1, 0x00000000 //Disable_LED content for PH_DAT
    mov r2, 0xffffffff //Enable_LED content for PH_DAT
    ldr r3, =0x01C20908 //PH_CFG2 Address
    str r0, [r3] //This sets the Port H configuration ready for the two LEDs
    ldr r3, =0x01C2090C //PH_DAT Address
    str r2, [r3] //This writes the data to Port H, indicating to switch the Yellow and Blue LED on
    ldr r6, =0x00004FFF //this is a "constant" for the upper value of the delay counter
    mov r5, 0x0 //this will be the switch to indicate if we have to switch the LEDs on or off
    mov r4, 0x0 //this will be the delay counter
endless:
    add r4, 0x1 //increase the counter by one
    cmp r4, r6 //compare it to the upper bound constant
    beq switch_led //if they are equal go to "switch_led"
    b endless
switch_led:
    mov r4, 0x0 //reset the counter
    cmp r5, 0x1 //check if we want to switch the LEDs on
    beq switch_on_led
switch_off_led:
    mov r5, 0x1 //set the switch so that the next time the LEDs will be switched on
    str r1, [r3] //write the "disable LEDs" data to the data register of Port H
    b endless
switch_on_led:
    mov r5, 0x0 //set the switch so that the next time the LEDs will be switched off
    str r2, [r3] //write the "enable LEDs" data to the data register of Port H
    b endless
