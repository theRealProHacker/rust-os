.section .init
.global _start
_start:
    bl cat
.Lend:
    b .Lend