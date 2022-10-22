#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

mod memlayout;
mod param;
#[macro_use]
mod printf;
mod riscv;
mod start;
mod uart;
mod trap;
mod proc;
mod syscall;

use core::arch::global_asm;
use crate::trap::trapinit;
use crate::proc::{procinit, userinit};

global_asm!(include_str!("entry.S"));
global_asm!(include_str!("kernelvec.S"));
global_asm!(include_str!("trampoline.S"));
global_asm!(include_str!("swtch.S"));

#[no_mangle]
fn main() {
    printf!("              __                  \n");
    error!("__  ____   __/ /_        _ __ ___ \n");
    warn!("\\ \\/ /\\ \\ / / '_ \\ _____| '__/ __|\n");
    info!(" >  <  \\ V /| (_) |_____| |  \\__ \\\n");
    debug!("/_/\\_\\  \\_/  \\___/      |_|  |___/\n");
    error!("May chaos take the world!");

    procinit();      // process table
    trapinit();      // trap vectors

    userinit();      // first user process
}
