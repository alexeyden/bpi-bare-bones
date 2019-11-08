CROSS_COMPILE ?= $(HOME)/x-tools/arm-unknown-eabi/bin/arm-unknown-eabi-
FLASH_DEV ?= /dev/sdb

TARGET = main
SOURCES = boot.s main.c uart.c

AS := $(CROSS_COMPILE)as
OBJCOPY := $(CROSS_COMPILE)objcopy
OBJDUMP := $(CROSS_COMPILE)objdump
CC := $(CROSS_COMPILE)gcc

CFLAGS = -fpic \
         -ffreestanding \
         -std=gnu99 \
         -O2 \
         -Wall \
         -Wextra \
         -mcpu=cortex-a7 \
         -Wl,-Tlinker.ld \
         -nostartfiles

.PHONY: all tools clean flash

all: $(TARGET)-spl

$(TARGET)-spl: $(TARGET) tools
	tools/mksunxiboot $(TARGET).bin $(TARGET)-spl.bin

$(TARGET): $(TARGET).elf
	$(OBJCOPY) -O binary $(TARGET).elf $(TARGET).bin

$(TARGET).elf: $(SOURCES)
	$(CC) $(CFLAGS) -o $(TARGET).elf $^

tools:
	$(MAKE) -C $@

flash:
	[ -b $(FLASH_DEV) ] && sudo dd if=$(TARGET)-spl.bin of=$(FLASH_DEV) bs=1K seek=8 && sync

clean:
	$(MAKE) -C tools clean
	rm -f *.elf *.bin

