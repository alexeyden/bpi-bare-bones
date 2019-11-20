TARGET=bpi-boot

# gcc toolchain tools
TOOLCHAIN=$(HOME)/x-tools/arm-unknown-eabi/bin/arm-unknown-eabi-
OBJDUMP=$(TOOLCHAIN)objdump

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

dump:
	$(OBJDUMP) -D -b binary -marmv7 -EL $(TARGET).bin

tools:
	$(MAKE) -C $@

flash:
	[ -b $(FLASH_DEV) ] && sudo dd if=$(TARGET)-spl.bin of=$(FLASH_DEV) bs=1K seek=8 && sync

clean:
	$(MAKE) -C tools clean
	rm -f $(TARGET).bin
	rm -f $(TARGET)-spl.bin
	cargo clean

