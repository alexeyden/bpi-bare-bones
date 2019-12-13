TARGET=bpi-boot

# gcc toolchain tools
TOOLCHAIN=$(HOME)/x-tools/arm-unknown-eabi/bin/arm-unknown-eabi-
OBJDUMP=$(TOOLCHAIN)objdump
OBJCOPY=$(TOOLCHAIN)objcopy

# sd card
FLASH_DEV ?= /dev/sdb

OUTPUT=target/armv7-none-unknown-eabi/release

.PHONY: all dump clean tools $(TARGET).bin

all: $(TARGET)-spl.bin

$(TARGET).bin:
	RUSTFLAGS="-C link-arg=-Tlinker.ld" cargo xbuild \
		--release \
		--target "armv7-none-unknown-eabi.json"
	cargo objcopy -- -O binary $(OUTPUT)/$(TARGET) $(TARGET).bin

$(TARGET)-spl.bin: tools $(TARGET).bin
	tools/mksunxiboot $(TARGET).bin $(TARGET)-spl.bin

$(TARGET).srec:
	$(OBJCOPY) -O srec --srec-forceS3 $(OUTPUT)/$(TARGET) $(TARGET).srec

chainloader: tools
	$(MAKE) -C chainload
	tools/mksunxiboot chainload/chainloader.bin chainloader-spl.bin

chainloader-flash:
	[ -b $(FLASH_DEV) ] && sudo dd if=chainloader-spl.bin of=$(FLASH_DEV) bs=1K seek=8 && sync

dump:
	$(OBJDUMP) -D -b binary -marmv7 -EL $(TARGET).bin

dump-elf:
	$(OBJDUMP) -d $(OUTPUT)/$(TARGET)

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

