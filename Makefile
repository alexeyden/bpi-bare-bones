TARGET=bpi-boot

# gcc toolchain tools
TOOLCHAIN=$(HOME)/Dev/x-tools/arm-unknown-eabi/bin/arm-unknown-eabi-
OBJDUMP=$(TOOLCHAIN)objdump
OBJCOPY=$(TOOLCHAIN)objcopy

# sd card
FLASH_DEV ?= /dev/sdc

OUTPUT=target/armv7a-none-eabi/release

.PHONY: all dump clean tools $(TARGET).bin

all: $(TARGET)-spl.bin $(TARGET).srec

$(TARGET).bin:
	RUSTFLAGS="-C link-arg=-Tlinker.ld" cargo build \
		--release \
		--target armv7a-none-eabi --verbose
	rust-objcopy -O binary $(OUTPUT)/$(TARGET) $(TARGET).bin

$(TARGET)-spl.bin: tools $(TARGET).bin
	tools/mksunxiboot $(TARGET).bin $(TARGET)-spl.bin

$(TARGET).srec: $(TARGET).bin
	objcopy -O srec --srec-forceS3 $(OUTPUT)/$(TARGET) $(TARGET).srec

chainloader: tools
	$(MAKE) -C chainload
	tools/mksunxiboot chainload/chainloader.bin chainloader-spl.bin

chainloader-flash:
	[ -b $(FLASH_DEV) ] && sudo dd if=chainloader-spl.bin of=$(FLASH_DEV) bs=1K seek=8 && sync

dump:
	rust-objdump -b binary -marmv7 $(TARGET).bin

dump-elf:
	rust-objdump -D $(OUTPUT)/$(TARGET)

tools:
	$(MAKE) -C $@

flash:
	[ -b $(FLASH_DEV) ] && sudo dd if=$(TARGET)-spl.bin of=$(FLASH_DEV) bs=1K seek=8 && sync

clean:
	$(MAKE) -C tools clean
	$(MAKE) -C chainload clean
	rm -f $(TARGET).bin
	rm -f $(TARGET)-spl.bin
	rm -f $(TARGET).srec
	rm -f chainloader-spl.bin
	cargo clean

