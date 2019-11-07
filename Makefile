CROSS_COMPILE ?= $(HOME)/x-tools/arm-unknown-eabi/bin/arm-unknown-eabi-

AS=$(CROSS_COMPILE)as
OBJCOPY=$(CROSS_COMPILE)objcopy

.PHONY: all tools clean

all: blinky-spl

blinky.elf: blinky.s
	$(AS) -o blinky.elf blinky.s

blinky: blinky.elf
	$(OBJCOPY) -O binary blinky.elf blinky.bin

tools:
	$(MAKE) -C $@

blinky-spl: blinky tools
	tools/mksunxiboot blinky.bin blinky-spl.bin

clean:
	$(MAKE) -C tools clean
	rm *.elf
	rm *.bin

