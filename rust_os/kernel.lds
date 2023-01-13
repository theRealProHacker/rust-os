ENTRY(_start)
SECTIONS
{
. = 0x20000000;
.init : { *(.init) }
.text : { *(.text) }
. = 0x22000000;
.kernel : { *(.kernel) }
.userstacks : { *(.usermem) }
. = 0x24000000 - 5*64*1024;
.kernelstacks : { *(.kernelstacks) }
}
