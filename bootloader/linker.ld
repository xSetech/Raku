SECTIONS
{

    /* @ RSP DMEM in KSEG1 */
    .text 0xA4000040 : AT(0xA4000040)
    {
        KEEP(*(.text.__start))
        *(.text*)
    }

}
