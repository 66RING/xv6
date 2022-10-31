.section .text.entry
.global _entry
# boot stack size
.equ STACK_SIZE, 4096*100
_entry:
	# set up a stack for C.
	# stack0 is declared in start.rs,
	# with a 4096-byte stack per CPU.
	# sp = stack0 + (hartid * 4096)
	la sp, STACK0
	li a0, STACK_SIZE
	csrr a1, mhartid
	addi a1, a1, 1
	mul a0, a0, a1
	add sp, sp, a0
	# jump to start() in start.rs
	call start
