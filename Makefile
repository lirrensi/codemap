CC_PATH := C:/ProgramData/mingw64/mingw64/bin
export CC := $(CC_PATH)/gcc.exe
export PATH := $(CC_PATH);$(PATH)

.PHONY: build release test clean

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

clean:
	cargo clean
