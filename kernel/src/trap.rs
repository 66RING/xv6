use crate::riscv::w_stvec;

extern "C" { fn kernelvec(); }


pub fn trapinit() {
    w_stvec(kernelvec as usize);
}

#[no_mangle]
pub fn kerneltrap() {
}
