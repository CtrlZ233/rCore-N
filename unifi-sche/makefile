OBJDUMP := rust-objdump --arch-name=riscv64
ELF := target/riscv64gc-unknown-none-elf/release/unifi-sche
DISASM_TMP := target/riscv64gc-unknown-none-elf/release/unifi-sche.asm


build:
	cargo build --release
	@$(OBJDUMP) -S $(ELF) > $(DISASM_TMP)
	cp target/riscv64gc-unknown-none-elf/release/unifi-sche \
        ../user/target/riscv64gc-unknown-none-elf/release/unifi-sche
