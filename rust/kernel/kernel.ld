ENTRY(_start)

SECTIONS
{
    /* The kernel will live at 1MB in the physical address space */
    /* This also works as the virtual address in our simple identity mapping */
    . = 1M;

    /* Ensure the multiboot header is at the very beginning */
    .multiboot BLOCK(4K) : ALIGN(4K)
    {
        *(.multiboot)
    }

    /* The rest of the kernel */
    .boot BLOCK(4K) : ALIGN(4K)
    {
        *(.multiboot.text)
    }

    .text BLOCK(4K) : ALIGN(4K)
    {
        *(.text)
    }

    .rodata BLOCK(4K) : ALIGN(4K)
    {
        *(.rodata)
    }

    .data BLOCK(4K) : ALIGN(4K)
    {
        *(.data)
    }

    .bss BLOCK(4K) : ALIGN(4K)
    {
        *(COMMON)
        *(.bss)
    }

    /* Define kernel bounds for use in code */
    kernel_start = 1M;
    kernel_end = .;
}
