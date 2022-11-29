#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
mod memlayout;
mod param;
#[macro_use]
mod printf;
#[macro_use]
mod riscv;
mod start;
mod uart;
mod trap;
mod proc;
mod syscall;
mod sysfile;
mod sysproc;
mod kalloc;
mod string;
mod vm;
mod exec;
mod spinlock;

use core::arch::global_asm;
use crate::kalloc::{kinit, allocator_test};
use crate::trap::trapinit;
use crate::proc::{procinit, userinit};
use crate::vm::{kvminit, kvminithart};

global_asm!(include_str!("entry.S"));
global_asm!(include_str!("kernelvec.S"));
global_asm!(include_str!("trampoline.S"));
global_asm!(include_str!("swtch.S"));
global_asm!(include_str!("link_app.S"));


#[no_mangle]
fn main() {
    printf!("              __                  \n");
    error!("__  ____   __/ /_        _ __ ___ \n");
    warn!("\\ \\/ /\\ \\ / / '_ \\ _____| '__/ __|\n");
    info!(" >  <  \\ V /| (_) |_____| |  \\__ \\\n");
    debug!("/_/\\_\\  \\_/  \\___/      |_|  |___/\n");
    error!("May chaos take the world!\n");
    dprintf!("[DEBUG ON]");

    kinit();
    allocator_test();
    kvminit();      // create kernel page table
    kvminithart();   // turn on paging
    procinit();      // process table
    // trapinit();      // trap vectors

    userinit();      // first user process
}
