ifeq ($(strip $(DEVKITPRO)),)
	$(error "Please set DEVKITPRO in your environment. export DEVKITPRO=<path to>devkitPro)
endif

DEBUG 		?= 1
FEATURES 	?=

NM 			:= $(DEVKITARM)/bin/arm-none-eabi-nm
3DSXTOOL 	:= $(DEVKITPRO)/tools/bin/3dsxtool
SMDHTOOL 	:= $(DEVKITPRO)/tools/bin/smdhtool

ifeq ($(DEBUG), 1)
PROFILE 	:= debug
CARGOFLAGS 	:=
else
PROFILE 	:= release
CARGOFLAGS  := --release
endif
ifneq ($(FEATURES),)
CARGOFLAGS	+= --features=$(FEATURES)
endif

TARGET		:= target/3ds/$(PROFILE)
DIST		:= dist/barista_$(PROFILE)
ROMFS 		:= romfs

CRATE_NAME 	:= barista
PROG_NAME 	:= Barista
PROG_DESC 	:= A launcher for Rhythm Heaven Megamix mods
PROG_AUTHOR := patataofcourse, RHModding
PROG_ICON 	:= icon.png

# Prepend devkitarm bin to PATH, in case there is another arm-none-eabi-gcc installed
export PATH := $(DEVKITARM)/bin:$(PATH)

export XARGO_RUST_SRC	= ../3ds-rust-env/rust-3ds-fork/library
export RUST_TARGET_PATH	= $(shell pwd)

.PHONY: all clean dist plgldr check doc fmt test

all: dist

### Main executable ###

dist: $(TARGET)/$(CRATE_NAME).3dsx $(TARGET)/$(CRATE_NAME).elf $(TARGET)/$(CRATE_NAME).smdh
	@mkdir -p $(DIST)
	@cp $(TARGET)/$(CRATE_NAME).elf $(DIST)
	@cp $(TARGET)/$(CRATE_NAME).lst $(DIST)
	@cp $(TARGET)/$(CRATE_NAME).3dsx $(DIST)
	@cp $(PROG_ICON) $(DIST)/$(notdir $(PROG_ICON))

%.elf: plgldr$(basename $@)
	@xargo build $(CARGOFLAGS)
	@$(NM) -Cn $@ > $(basename $@).lst

%.smdh:
	@$(SMDHTOOL) --create "${PROG_NAME}" "${PROG_DESC}" "${PROG_AUTHOR}" "${PROG_ICON}" $(basename $@).smdh

%.3dsx: %.elf %.smdh
	@$(3DSXTOOL) $(basename $@).elf $(basename $@).3dsx --smdh=$(basename $@).smdh --romfs=$(ROMFS)


### Clean

clean:
	@echo "clean ..."
	@rm -rf target
	@rm -rf dist
	@cd plgldr && make clean --no-print-directory

cleanenv: clean
	@rm -rf ~/.xargo


### C libraries ###

plgldr:
	@make --no-print-directory -C plgldr


### Useful Cargo stuff ###

doc:
	@xargo doc --open

fmt:
	@xargo fmt

test: debug
	@if which citra 2> /dev/null > /dev/null;\
		then citra $(TARGET)/$(CRATE_NAME).elf;\
		else flatpak run org.citra_emu.citra $(TARGET)/$(CRATE_NAME).elf;\
	fi

check:
	@xargo check --features=$(FEATURES)