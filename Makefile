# Project info
CRATE_NAME := barista
PROG_NAME := Barista
PROG_DESC := A launcher for Rhythm Heaven Megamix mods
PROG_AUTHOR := patataofcourse, RHModding
PROG_ICON := $(DEVKITPRO)/libctru/default_icon.png

3DSXTOOL := $(DEVKITPRO)/tools/bin/3dsxtool
SMDHTOOL := $(DEVKITPRO)/tools/bin/smdhtool

# Prepend devkitarm bin to PATH, in case there is another arm-none-eabi-gcc installed
export PATH := $(DEVKITARM)/bin:$(PATH)

export CC_3ds := $(DEVKITARM)/bin/arm-none-eabi-gcc
export TARGET_CFLAGS := -specs=3dsx.specs -mfloat-abi=hard -march=armv6k -mtune=mpcore \
						-mfpu=vfp -mtp=soft

export XARGO_RUST_SRC=../3ds-rust-env/rust-3ds-fork/library

.PHONY: all clean $(CRATE_NAME) dist doc test send target/3ds/release/$(CRATE_NAME).elf

all: $(CRATE_NAME) 

target/3ds/release/$(CRATE_NAME).elf:
	RUST_TARGET_PATH=$(shell pwd) xargo build --release

target/3ds/release/$(CRATE_NAME).smdh:
	$(SMDHTOOL) --create "${PROG_NAME}" "${PROG_DESC}" "${PROG_AUTHOR}" "${PROG_ICON}" target/3ds/release/$(CRATE_NAME).smdh

target/3ds/release/$(CRATE_NAME).3dsx: target/3ds/release/$(CRATE_NAME).elf target/3ds/release/$(CRATE_NAME).smdh
	$(3DSXTOOL) target/3ds/release/$(CRATE_NAME).elf target/3ds/release/$(CRATE_NAME).3dsx --smdh=target/3ds/release/$(CRATE_NAME).smdh

$(CRATE_NAME): target/3ds/release/$(CRATE_NAME).3dsx

dist: $(CRATE_NAME)
	mkdir -p dist/$(CRATE_NAME)
	cp target/3ds/release/$(CRATE_NAME).elf dist/$(CRATE_NAME)
	cp target/3ds/release/$(CRATE_NAME).3dsx dist/$(CRATE_NAME)
	cp $(PROG_ICON) dist/$(CRATE_NAME)/$(CRATE_NAME).png

doc:
	RUST_TARGET_PATH=$(shell pwd) xargo doc

test: $(CRATE_NAME)
	citra target/3ds/release/$(CRATE_NAME).elf

send: $(CRATE_NAME)
	3dslink target/3ds/release/$(CRATE_NAME).3dsx

clean:
	rm -rf target
	rm -rf dist

cleanenv: clean
	rm -rf ~/.xargo
