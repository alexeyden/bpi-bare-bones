CROSS_COMPILE ?= $(HOME)/x-tools/arm-unknown-eabi/bin/arm-unknown-eabi-

TARGET = chainloader
SOURCES = boot.s main.c

AS := $(CROSS_COMPILE)as
CC := $(CROSS_COMPILE)gcc
OBJCOPY := $(CROSS_COMPILE)objcopy
OBJDUMP := $(CROSS_COMPILE)objdump

CFLAGS = -fpic \
         -ffreestanding \
         -fno-jump-tables \
         -std=gnu99 \
         -Os \
         -Wall \
         -Wextra \
         -mcpu=cortex-a7 \
         -Wl,-Tlinker.ld \
         -nostartfiles

.PHONY: all clean

all: $(TARGET)

$(TARGET): $(TARGET).elf
	$(OBJCOPY) -O binary $(TARGET).elf $(TARGET).bin

$(TARGET).elf: $(SOURCES)
	$(CC) $(CFLAGS) -o $(TARGET).elf $^

clean:
	rm -f *.elf *.bin

