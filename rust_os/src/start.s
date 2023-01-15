@ The start function. Here we init the stackpointers in all modes before Rust takes over. 
.global _start
.section .init
_start:
    @ https://community.arm.com/arm-community-blogs/b/architectures-and-processors-blog/posts/how-to-load-constants-in-assembly-for-arm-architecture
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

@ Note: mainly copied from beispiel
.global _src1_handler
_src1_handler:
    @ first go into sys mode
    @ push {r0}
    @ mrs r0, cpsr
    @ bic r0, r0, #0x1F
    @ orr r0, #0x13
    @ msr cpsr, r0
    @ pop {r0}
    @ now push everything onto the stack and pass the stack pointer to scr1_handler
    sub	lr, #4
 	stmfd sp!, {lr}
 
 	/*
 	 * Aufgrund des S-Bits ist kein Writeback möglich, also Platz auf Stack
 	 * manuell reservieren.
 	 */
 	sub	sp, #(15*4)
 	stmia sp, {r0-r14}^
    
  	mov	r0, sp
 	bl	src1_handler
 
 	/*
 	 * Zuvor gesicherte Register wieder herstellen (R0-R12, R13-R14
 	 * User-Modus). Laut Doku sollte in der Instruktion nach LDM^ auf
 	 * keines der umgeschalteten Register zugegriffen werden.
 	 */
 	ldmia	sp, {r0-r14}^
 	nop
 	add	sp, sp, #(15*4)
 
 	/* Rücksprung durch Laden des PC mit S-Bit */ 
 	ldmfd	sp!, {pc}^