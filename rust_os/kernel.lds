ENTRY(_start)
SECTIONS
{
. = 0x20000000;
.init : { *(.init) }
. = ALIGN(4);
.text : { *(.text) }
. = 0x22000000;
.kernel : { *(.kernel) }
. = ALIGN(4);
.usermem : { *(.usermem) }
. = 0x24000000 - 5*64*1024;
.kernelstacks : { *(.kernelstacks) }
}
