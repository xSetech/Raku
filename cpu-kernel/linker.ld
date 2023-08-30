/*  Linker script for the CPU kernel

    Notes:

    - Kernel code and data are expected to fit within a fixed amount of memory.
      This is an arbitrary choice enforced at link time; adjust LENGTH in the
      MEMORY block below to change the amount. Remaining memory space is
      expected to be allocated to the heap, stack, and framebuffer at runtime.

    - Word alignment (4 bytes) is a minimum requirement for least surprising
      data transfers from the ROM via scalar instructions. Double alignment (8
      bytes) is required for least surprising ROM DMA transfers (see the wiki
      pages for the PI and memory map).

*/

MEMORY
{
    /* 256KB */
    KERN : ORIGIN = 0xA0000000, LENGTH = 0x00040000
}

SECTIONS
{
    .text : AT(0xA0000000)
    {
        KEEP(*(.text.__start));
        *(.text*);
        . = ALIGN(8);
    } > KERN
    .data : ALIGN(8) {
        *(.data*);
        . = ALIGN(8);
    } > KERN
    .rodata : ALIGN(8) {
        *(.rodata*);
        . = ALIGN(8);
    } > KERN
}
