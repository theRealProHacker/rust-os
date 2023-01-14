@ This file contains the start function. Here we init the stackpointers in all modes before Rust takes over. 
.global _start
.section .init
_start:
    @ https:@community.arm.com/arm-community-blogs/b/architectures-and-processors-blog/posts/how-to-load-constants-in-assembly-for-arm-architecture
    @ v1 is the moving stack pointer, v2 the individual stacksizes
    @ v3 the moving cpsr, v4 the backup
    mov v1, #0x24000000
    mov v2, #0x10000 @ 64kBv
    @ sys & usr
    mrs v4, cpsr
    mov sp, v1
    sub v1, v2
    @ software interrupt
    bic v3, v4, #0x1F
    orr v3, #0x13
    msr cpsr, v3
    mov sp, v1
    sub v1, v2
    @ undefined
    bic v3, #0x1F
    orr v3, #0x1B
    msr CPSR, v3
    mov sp, v1
    sub v1, v2
    @ abort
    bic v3, #0x1F
    orr v3, #0x17
    msr CPSR, v3
    mov sp, v1
    sub v1, v2
    @ irq
    bic v3, #0x1F
    orr v3, #0x12
    msr CPSR, v3
    mov sp, v1
    sub v1, v2
    @ back to backup
    msr CPSR, v4
    b rust_start

.global _start
_src1_handler:
    push {r0-r15}
    mov r0, sp
    mrs r1, spsr
    b src1_handler