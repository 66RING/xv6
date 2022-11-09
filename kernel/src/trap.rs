use crate::memlayout::*;
use crate::riscv::*;
use crate::syscall::syscall;
use crate::proc::myproc;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! { pub static ref TICKS: Mutex<usize> = unsafe { Mutex::new(0) }; }

extern "C" { 
    fn userret(trapframe: usize, satp: usize) -> !; 
    fn uservec(trapframe: usize) -> !; 
    fn kernelvec(); 
    fn trampoline();
}

pub fn trapinit() {
    w_stvec(kernelvec as usize);
}

#[no_mangle]
pub fn kerneltrap() {
    unimplemented!()
}

/// 用户态陷入内核态的处理函数: 系统调用, page fault等
/// usertrap_handler
/// TODO: 补完
#[no_mangle]
pub fn usertrap() -> ! {
    if (r_sstatus() & SSTATUS_SPP) != 0 {
        panic!("usertrap: not from user mode");
    }
    // send interrupts and exceptions to kerneltrap(),
    // since we're now in the kernel.
    w_stvec(kernelvec as usize);

    // 1. 根据trap的原因(系统调用, page fault)分case处理
    //  1.1 对于系统调用, trapframe.a7用于存储系统调用号
    //      trapframe.a0 ~ trapframe.a5分别存储各个参数(见argraw())
    // w_stvec(kernelvec as usize);
    let mut p = myproc();
    // let mut trapframe = unsafe { &mut **p.trapframe.as_mut().unwrap() };
    let mut trapframe = p.trapframe_mut().unwrap();
    // save user program counter.
    trapframe.epc = r_sepc();

    if r_scause() == 8 {
        trapframe.epc += 4;

        intr_on();
        syscall();
    } else {
        printf!("usertrap(): unexpected scause {:#x} pid={}\n", r_scause(), p.pid);
        printf!("            sepc={:#x} stval={:#x}\n", r_sepc(), r_stval());
        p.killed = 1;
        unimplemented!();
    }

    usertrapret();
}


/// 返回用户态
/// 1. 设置用户态traphandler
/// 2. 调用userret返回用户态, 并将trapframe作为参数传入a0寄存器
/// TODO: 补完
#[no_mangle]
pub fn usertrapret() -> ! {
    // we're about to switch the destination of traps from
    // kerneltrap() to usertrap(), so turn off interrupts until
    // we're back in user space, where usertrap() is correct.
    intr_off();

    // send syscalls, interrupts, and exceptions to trampoline.S
    // userret等位于.global trampoline段, trampoline段映射到TRAMPOLINE中, 通过相对位置访问 
    w_stvec(TRAMPOLINE + (uservec as usize - trampoline as usize));

    let mut p = myproc();
    // let mut trapframe = unsafe { &mut **p.trapframe.as_mut().unwrap() };
    let mut trapframe = p.trapframe_mut().unwrap();
    // kstack存入trapframe
    // set up trapframe values that uservec will need when
    // the process next re-enters the kernel.
    trapframe.kernel_sp = p.kstack + PGSIZE;
    // 保存内核页表
    trapframe.kernel_satp = r_satp();
    trapframe.kernel_trap = usertrap as usize;
    trapframe.kernel_hartid = r_tp();

    // set up the registers that trampoline.S's sret will use
    // to get to user space.
    
    // set S Previous Privilege mode to User.
    let mut x = r_sstatus();
    x &= !SSTATUS_SPP; // clear SPP to 0 for user mode
    x |= SSTATUS_SPIE; // enable interrupts in user mode
    w_sstatus(x);

    // set S Exception Program Counter to the saved user pc.
    w_sepc(trapframe.epc);

    // tell trampoline.S the user page table to switch to.
    // TODO: unsafe raw pointer v
    let satp = MAKE_SATP!(p.pagetable.unwrap() as usize);

    // WARN: 不再可以直接函数调用, 需要根据映射的位置(相对trampoline)计算出函数的入口
    let func_ptr = TRAMPOLINE + (userret as usize - trampoline as usize);
    let func: extern "C" fn(usize, usize) -> ! = unsafe { core::mem::transmute(func_ptr as usize) };
    func(TRAPFRAME, satp)
}

// 用户程序会触发scause 0xc 
