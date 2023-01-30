ENTRY(_start)
SECTIONS
{
. = 0x20000000;
.init : { *(.init) }
. = ALIGN(4);
.text : { *(.text) }
}
