all:
	cargo build --release --target thumbv7em-none-eabihf
	@arm-none-eabi-objcopy -O binary target/thumbv7em-none-eabihf/release/demo target/thumbv7em-none-eabihf/release/demo.bin
	@exa -Bl target/thumbv7em-none-eabihf/release/demo.bin
	@arm-none-eabi-nm -SlC target/thumbv7em-none-eabihf/release/demo | sort > target/thumbv7em-none-eabihf/release/demo.map
	@arm-none-eabi-objdump -Cdw target/thumbv7em-none-eabihf/release/demo > target/thumbv7em-none-eabihf/release/demo.asm
	@arm-none-eabi-objdump -s -j .rodata target/thumbv7em-none-eabihf/release/demo >> target/thumbv7em-none-eabihf/release/demo.asm

run:
	cargo run --release
