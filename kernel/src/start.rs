use crate::main;
use crate::riscv::*;
use crate::param::*;
use core::arch::asm;
use crate::memlayout::*;

// 设置内核stack
// 也可在.S中设置，.space 4096*8
// TODO: remove hard code
//  我们目前需要比较大是boot stack
#[no_mangle]
pub static STACK0: [u8; 4096*100] = [0; 4096*100];

// TODO: 存储访问timer的地址
// a scratch area per CPU for machine-mode timer interrupts.
static mut TIMER_SCRATCH: [[usize; 5]; NCPU] = [[0; 5]; NCPU];

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
    w_sie(r_sie() | SSIE | STIE | SEIE);

    // TODO: 必须
    // configure Physical Memory Protection to give supervisor mode
    // access to all of physical memory.
    w_pmpaddr0(0x3fffffffffffff);
    w_pmpcfg0(0xf);

    // ask for clock interrupts.
    //timerinit();
    
    // keep each CPU's hartid in its tp register, for cpuid().
    let id = r_mhartid();
    w_tp(id);
    
    // switch to supervisor mode and jump to main().
    asm!("mret");

    loop {}
}


// TODO: implement
fn timerinit() {
    // each CPU has a separate source of timer interrupts.
    let id = r_mhartid();
    
    // ask the CLINT for a timer interrupt.
    let interval = 1000000; // cycles; about 1/10th second in qemu.
    let CLINT_MTIMECMP  = |id| CLINT + 0x4000 + 8*(id) ;
    let value = unsafe { core::ptr::read_volatile(CLINT_MTIME as *const usize) + interval};
    unsafe { core::ptr::write_volatile(CLINT_MTIMECMP(id) as *mut usize, value)};

    
    // prepare information in scratch[] for timervec.
    // scratch[0..2] : space for timervec to save registers.
    // scratch[3] : address of CLINT MTIMECMP register.
    // scratch[4] : desired interval (in cycles) between timer interrupts.
    unsafe {
        TIMER_SCRATCH[id][3] = CLINT_MTIMECMP(id);
        TIMER_SCRATCH[id][4] = interval;
        let scratch = TIMER_SCRATCH[id][0] as *mut usize;
        w_mscratch(scratch as usize);
    }
    
    extern "C" { fn timervec(); }
    // set the machine-mode trap handler.
    w_mtvec(timervec as usize);
    
    // enable machine-mode interrupts.
    w_mstatus(r_mstatus() | MSTATUS_MIE);
    
    // enable machine-mode timer interrupts.
    w_mie(r_mie() | MIE_MTIE);
}

