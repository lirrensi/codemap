ifeq ($(OS),Windows_NT)
CC_PATH := C:/ProgramData/mingw64/mingw64/bin
export CC := $(CC_PATH)/gcc.exe
export CXX := $(CC_PATH)/g++.exe
export PATH := $(CC_PATH);$(PATH)
else
export CARGO_TARGET_DIR ?= /tmp/codemap-target
endif

.PHONY: build release test clean

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

clean:
	cargo clean
