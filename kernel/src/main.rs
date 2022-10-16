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

use core::arch::global_asm;

global_asm!(include_str!("entry.S"));

#[no_mangle]
fn main() {
    printf!("              __                  \n");
    error!("__  ____   __/ /_        _ __ ___ \n");
    warn!("\\ \\/ /\\ \\ / / '_ \\ _____| '__/ __|\n");
    info!(" >  <  \\ V /| (_) |_____| |  \\__ \\\n");
    debug!("/_/\\_\\  \\_/  \\___/      |_|  |___/\n");
    panic!("May chaos take the world!");
}
