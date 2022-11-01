.section .init
.global _start
_start:
    bl yellow_on
.Lend:
    b .Lend