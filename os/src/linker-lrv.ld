OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x100200000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    stext = .;
    .text : {
        *(.text.entry)
        . = ALIGN(4K);
        strampoline = .;
        *(.text.trampoline);
        . = ALIGN(4K);
        *(.text .text.*)
    }

    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data.heap)
        *(.data.executor)
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    edata = .;
    sbss_with_stack = .;
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss.memory)
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        ebss_ma = .;
    }

    . = ALIGN(4K);
    ebss = .;
    svdso = .;
    .vdso : {
        *(.vdso .vdso.*)
    }
    . = ALIGN(4K);
    evdso = .;
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}
