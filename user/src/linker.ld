OUTPUT_ARCH(riscv)
ENTRY(_start)

BASE_ADDRESS = 0x1000;

SECTIONS
{
    . = BASE_ADDRESS;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    . = ALIGN(4K);
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    . = ALIGN(4K);
    .data : {
        *(.data.heap)
        *(.data.executor)
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    .bss : {
        *(.bss.interface)
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }
    .vdso : {
        *(.vdso .vdso.*)
    }
    
    /DISCARD/ : {
        *(.eh_frame)
        *(.debug*)
    }
}