use crate::main;
use crate::riscv::*;
use crate::param::*;
use core::arch::asm;
use crate::memlayout::*;

// 设置内核stack
// 也可在.S中设置，.space 4096*8
#[no_mangle]
static STACK0: [u8; STACK_SIZE*2 * NCPU] = [0; STACK_SIZE*2 * NCPU];

// a scratch area per CPU for machine-mode timer interrupts.
static mut TIMER_SCRATCH: [[usize; NCPU]; 5] = [[0]; 5];

// entry.S会跳转到这里
// no_mangle会避免编译器改名
#[no_mangle]
pub unsafe fn start() {
    // set M Previous Privilege mode to Supervisor, for mret.
    set_mpp(MPP::Supervisor);

    // set M Exception Program Counter to main, for mret.
    // requires gcc -mcmodel=medany
    w_mepc(main as usize);
    
    // disable paging for now.
    w_satp(0);

    // delegate all interrupts and exceptions to supervisor mode.
    w_medeleg(0xffff);
    w_mideleg(0xffff);
    intr_on();

    // 必须
    // configure Physical Memory Protection to give supervisor mode
    // access to all of physical memory.
    w_pmpaddr0(0x3fffffffffffff);
    w_pmpcfg0(0xf);

    // ask for clock interrupts.
    timerinit();
    
    // keep each CPU's hartid in its tp register, for cpuid().
    let id = r_mhartid();
    w_tp(id);
    
    // switch to supervisor mode and jump to main().
    asm!("mret");

    loop {}
}


// TODO: implment
fn timerinit() {
    // each CPU has a separate source of timer interrupts.
    // let id = mhartid::r_mhartid();
    
    // // ask the CLINT for a timer interrupt.
    // let interval = 1000000; // cycles; about 1/10th second in qemu.
    // let offset = memlayout::CLINT + 0x4000 + 8*(id);
    // let value = unsafe { core::ptr::read_volatile(memlayout::CLINT_MTIME as *const _) + interval};
    // unsafe { core::ptr::write_volatile(offset as *mut u64, value)};

    
    // // prepare information in scratch[] for timervec.
    // // scratch[0..2] : space for timervec to save registers.
    // // scratch[3] : address of CLINT MTIMECMP register.
    // // scratch[4] : desired interval (in cycles) between timer interrupts.

    // // uint64 *scratch = &timer_scratch[id][0];
    // // scratch[3] = CLINT_MTIMECMP(id);
    // // scratch[4] = interval;
    // // w_mscratch((uint64)scratch);
    // let scratch = unsafe {TIMER_SCRATCH[id][0] as *mut usize};

    
    // set the machine-mode trap handler.
    // w_mtvec((uint64)timervec);
    
    // enable machine-mode interrupts.
    w_mstatus(r_mstatus() | MSTATUS_MIE);
    
    // enable machine-mode timer interrupts.
    w_mie(r_mie() | MIE_MTIE);
}

