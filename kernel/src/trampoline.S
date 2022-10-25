	#
        # code to switch between user and kernel space.
        #
        # this code is mapped at the same virtual address
        # (TRAMPOLINE) in user and kernel space so that
        # it continues to work when it switches page tables.
	#
	# kernel.ld causes this to be aligned
        # to a page boundary.
        #
	.section .text # TODO: should be trampsec, but text for handful
.globl trampoline
trampoline:
.align 4
.globl uservec
uservec:    
	# ======================================
		# 从用户态陷入内核态
		# **此时**:
		# 	sscratch指向进程内核栈指针
		# 	sp指向用户栈
		# 	a7寄存保存了系统调用号
		# 	a0~a5为系统调用参数
		#		因为是默认的函数参数传递寄存器, 那么当用户态下陷时, a0~a5就指向用户系统调用的参数

		# TODO: 参数协议, 详见usertrap
		# usertrap()为所谓trap_handler

		# a0~a5, a7等即将要压入trapframe
		# 之后通过trapframe访问(argraw())

		# 之后:
		# 	trapframe.a0保存系统调用的结果

		# 1. 先切换栈空间, 以保存trapframe到内核栈
		# 	进入用户态后sscratch就保存了内核栈指针	
		# 	为了不影响其他寄存器, 我们需要寄存器相对寻址: xx(a0)
		# 2. 寄存器现场存入trapframe

		# 唯一会收到影响的寄存器只有a0, 所以使用a0来访问内核栈trapframe
		# 	实现尽可能全的寄存器现场保存

		# FIXME: 栈空间切换bug导致, usertrap调用失败

		# sscratch此时指向内核栈, 与sp指向用户栈
		# 	FIXME: 先不考虑部分寄存器现场没能保存, 先切换
		#	FIXME: 必须保存好a0寄存器线程, 因为要通过a0获取系统调用参数
		# TODO: xv6中sscratch并不是sp寄存器的backup!!!!!而rcore是

		# 切换栈空间, 寄存器现场保存到trapframe中
        csrrw a0, sscratch, a0

        # save the user registers in TRAPFRAME
        sd ra, 40(a0)
        sd sp, 48(a0)
        sd gp, 56(a0)
        sd tp, 64(a0)
        sd t0, 72(a0)
        sd t1, 80(a0)
        sd t2, 88(a0)
        sd s0, 96(a0)
        sd s1, 104(a0)
        sd a1, 120(a0)
        sd a2, 128(a0)
        sd a3, 136(a0)
        sd a4, 144(a0)
        sd a5, 152(a0)
        sd a6, 160(a0)
        sd a7, 168(a0)
        sd s2, 176(a0)
        sd s3, 184(a0)
        sd s4, 192(a0)
        sd s5, 200(a0)
        sd s6, 208(a0)
        sd s7, 216(a0)
        sd s8, 224(a0)
        sd s9, 232(a0)
        sd s10, 240(a0)
        sd s11, 248(a0)
        sd t3, 256(a0)
        sd t4, 264(a0)
        sd t5, 272(a0)
        sd t6, 280(a0)

	# save the user a0 in p->trapframe->a0
        csrr t0, sscratch
        sd t0, 112(a0)

        # restore kernel stack pointer from p->trapframe->kernel_sp
        ld sp, 8(a0)

        # make tp hold the current hartid, from p->trapframe->kernel_hartid
        ld tp, 32(a0)

        # load the address of usertrap(), p->trapframe->kernel_trap
        ld t0, 16(a0)

        # restore kernel page table from p->trapframe->kernel_satp
        ld t1, 0(a0)
        #csrw satp, t1
        #sfence.vma zero, zero

        # a0 is no longer valid, since the kernel page
        # table does not specially map p->tf.

        # jump to usertrap(), which does not return
        #jr t0
		call usertrap
	# ======================================
	# FIXME: 以下是xv6的实现, 目前还用不上, 可以对比以下



#	#
#        # trap.c sets stvec to point here, so
#        # traps from user space start here,
#        # in supervisor mode, but with a
#        # user page table.
#        #
#        # sscratch points to where the process's p->trapframe is
#        # mapped into user space, at TRAPFRAME.
#        #
#        
#	# swap a0 and sscratch
#        # so that a0 is TRAPFRAME
#        csrrw a0, sscratch, a0
#
#        # save the user registers in TRAPFRAME
#        sd ra, 40(a0)
#        sd sp, 48(a0)
#        sd gp, 56(a0)
#        sd tp, 64(a0)
#        sd t0, 72(a0)
#        sd t1, 80(a0)
#        sd t2, 88(a0)
#        sd s0, 96(a0)
#        sd s1, 104(a0)
#        sd a1, 120(a0)
#        sd a2, 128(a0)
#        sd a3, 136(a0)
#        sd a4, 144(a0)
#        sd a5, 152(a0)
#        sd a6, 160(a0)
#        sd a7, 168(a0)
#        sd s2, 176(a0)
#        sd s3, 184(a0)
#        sd s4, 192(a0)
#        sd s5, 200(a0)
#        sd s6, 208(a0)
#        sd s7, 216(a0)
#        sd s8, 224(a0)
#        sd s9, 232(a0)
#        sd s10, 240(a0)
#        sd s11, 248(a0)
#        sd t3, 256(a0)
#        sd t4, 264(a0)
#        sd t5, 272(a0)
#        sd t6, 280(a0)
#
#	# save the user a0 in p->trapframe->a0
#        csrr t0, sscratch
#        sd t0, 112(a0)
#
#        # restore kernel stack pointer from p->trapframe->kernel_sp
#        ld sp, 8(a0)
#
#        # make tp hold the current hartid, from p->trapframe->kernel_hartid
#        ld tp, 32(a0)
#
#        # load the address of usertrap(), p->trapframe->kernel_trap
#        ld t0, 16(a0)
#
#        # restore kernel page table from p->trapframe->kernel_satp
#        ld t1, 0(a0)
#        csrw satp, t1
#        sfence.vma zero, zero
#
#        # a0 is no longer valid, since the kernel page
#        # table does not specially map p->tf.
#
#        # jump to usertrap(), which does not return
#        jr t0

.globl userret
userret:
		# =================================
		# WARN: xv6中切换的核心不是sp, 而是**trapframe**, sscratch保存的是trapframe的指针, **且trapframe不是保存在内核栈中, 而是保存在堆上**
#		# FIXME: batch system only
#		# 1. 经过swtch后寄存器现场得到了切换
#		# 	sp指向进程内核栈
#		# 	pc指向userret
#		# 2. 进程内核栈中有我们制造的trapframe, 用于返回用户态
#		# 	trapframe.sp指向用户栈
#		# 	trapframe.pc指向用户程序的入口地址
#		# 3. userret中完成剩余切换
#		#	**sp最后切换**, 因为我们需要trapframe的内容
#		# 		TODO: 先利用sscratch寄存器保存当前内核栈
#		# 	从trapframe中读取现场
#		#	sp与sscratch交换
#		#	
#
#
#		# 现在sp指向保存在内核中的trapframe
#		# trapframe.sp保存了用户栈指针
#		# 主要就是利用sscratch"永久"保存内核栈, 然后切换epc到entry, sp到用户栈
#		# 栈空间最后切换: sscratch中先保存user_sp, 最后和当前sp(kernel_sp)切换就完成了
#        ld t0, 48(sp)
#        csrw sscratch, t0
#		# NOW: sscratch -> user_sp, ready to swtich
#		ld t1, 24(sp)
#		# TODO: sepc是什么呢
#        csrw sepc, t1
#
#        # restore all but sp from TRAPFRAME
#        ld ra, 40(sp)
#        ld gp, 56(sp)
#        ld tp, 64(sp)
#        ld t0, 72(sp)
#        ld t1, 80(sp)
#        ld t2, 88(sp)
#        ld s0, 96(sp)
#        ld s1, 104(sp)
#        ld a0, 112(sp)
#        ld a1, 120(sp)
#        ld a2, 128(sp)
#        ld a3, 136(sp)
#        ld a4, 144(sp)
#        ld a5, 152(sp)
#        ld a6, 160(sp)
#        ld a7, 168(sp)
#        ld s2, 176(sp)
#        ld s3, 184(sp)
#        ld s4, 192(sp)
#        ld s5, 200(sp)
#        ld s6, 208(sp)
#        ld s7, 216(sp)
#        ld s8, 224(sp)
#        ld s9, 232(sp)
#        ld s10, 240(sp)
#        ld s11, 248(sp)
#        ld t3, 256(sp)
#        ld t4, 264(sp)
#        ld t5, 272(sp)
#        ld t6, 280(sp)
#
#		# 弹出内核栈中的trapframe
#		addi sp, sp, 34*8
#	# restore user sp, and save TRAPFRAME in sscratch
#        csrrw sp, sscratch, sp
#        
#        # return to user mode and user pc.
#        # usertrapret() set up sstatus and sepc.
#        sret

		# TODO: 注释
        # userret(TRAPFRAME)
        # switch from kernel to user.
        # usertrapret() calls here.
        # a0: TRAPFRAME, in user page table.
        # a1: user page table, for satp.

        ## switch to the user page table.
        #csrw satp, a1
        #sfence.vma zero, zero

        # put the saved user a0 in sscratch, so we
        # can swap it with our a0 (TRAPFRAME) in the last step.

		# TODO: 开始绕了
		# 此时
		# 	trapframe.a0指向用户态a0
		# 	a0指向trapframe
        ld t0, 112(a0)
        csrw sscratch, t0

        # restore all but a0 from TRAPFRAME
		# TODO: 内核栈存到哪了? kstack
        ld ra, 40(a0)
        ld sp, 48(a0)
        ld gp, 56(a0)
        ld tp, 64(a0)
        ld t0, 72(a0)
        ld t1, 80(a0)
        ld t2, 88(a0)
        ld s0, 96(a0)
        ld s1, 104(a0)
        ld a1, 120(a0)
        ld a2, 128(a0)
        ld a3, 136(a0)
        ld a4, 144(a0)
        ld a5, 152(a0)
        ld a6, 160(a0)
        ld a7, 168(a0)
        ld s2, 176(a0)
        ld s3, 184(a0)
        ld s4, 192(a0)
        ld s5, 200(a0)
        ld s6, 208(a0)
        ld s7, 216(a0)
        ld s8, 224(a0)
        ld s9, 232(a0)
        ld s10, 240(a0)
        ld s11, 248(a0)
        ld t3, 256(a0)
        ld t4, 264(a0)
        ld t5, 272(a0)
        ld t6, 280(a0)

	# restore user a0, and save TRAPFRAME in sscratch
		# trapframe保存到sscratch中, 用户态a0恢复
        csrrw a0, sscratch, a0
		# 此时trapframe保存到了sscratch
        
        # return to user mode and user pc.
        # usertrapret() set up sstatus and sepc.
        sret

#		# =================================


#		# TODO: 以下是xv6现成的userret, 暂时不用, 注释一下, 可以对比看一下差别
#        # userret(TRAPFRAME, pagetable)
#        # switch from kernel to user.
#        # usertrapret() calls here.
#        # a0: TRAPFRAME, in user page table.
#        # a1: user page table, for satp.
#
#        # switch to the user page table.
#        csrw satp, a1
#        sfence.vma zero, zero
#
#        # put the saved user a0 in sscratch, so we
#        # can swap it with our a0 (TRAPFRAME) in the last step.
#        ld t0, 112(a0)
#        csrw sscratch, t0
#
#        # restore all but a0 from TRAPFRAME
#        ld ra, 40(a0)
#        ld sp, 48(a0)
#        ld gp, 56(a0)
#        ld tp, 64(a0)
#        ld t0, 72(a0)
#        ld t1, 80(a0)
#        ld t2, 88(a0)
#        ld s0, 96(a0)
#        ld s1, 104(a0)
#        ld a1, 120(a0)
#        ld a2, 128(a0)
#        ld a3, 136(a0)
#        ld a4, 144(a0)
#        ld a5, 152(a0)
#        ld a6, 160(a0)
#        ld a7, 168(a0)
#        ld s2, 176(a0)
#        ld s3, 184(a0)
#        ld s4, 192(a0)
#        ld s5, 200(a0)
#        ld s6, 208(a0)
#        ld s7, 216(a0)
#        ld s8, 224(a0)
#        ld s9, 232(a0)
#        ld s10, 240(a0)
#        ld s11, 248(a0)
#        ld t3, 256(a0)
#        ld t4, 264(a0)
#        ld t5, 272(a0)
#        ld t6, 280(a0)
#
#	# restore user a0, and save TRAPFRAME in sscratch
#        csrrw a0, sscratch, a0
#        
#        # return to user mode and user pc.
#        # usertrapret() set up sstatus and sepc.
#        sret
#

