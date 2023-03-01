OBJDUMP := rust-objdump --arch-name=riscv64
ELF := target/riscv64gc-unknown-none-elf/release/sharedscheduler
DISASM_TMP := target/riscv64gc-unknown-none-elf/release/sharedscheduler.asm


build:
	cargo build --release --features inner
	# @$(OBJDUMP) -S $(ELF) > $(DISASM_TMP)
	cp target/riscv64gc-unknown-none-elf/release/sharedscheduler \
        ../user/target/riscv64gc-unknown-none-elf/release/sharedscheduler
