// TODO learn bit_field crate
use bit_field::BitField;
use core::arch::asm;

pub const PGSIZE: usize = 4096;
pub const MAXVA: usize = 1 << (9 + 9 + 9 + 12 - 1);
pub const PGSHIFT: usize = 12; // bits of offset within a page
pub const PXMASK: usize = 0x1FF; // 9 bits
pub const SATP_SV39: usize = 8 << 60;


#[macro_export]
macro_rules! PGROUNDUP {
    ( $x:expr ) => {
        (($x)+$crate::riscv::PGSIZE-1) & !($crate::riscv::PGSIZE-1)
    }
}

#[macro_export]
macro_rules! PGROUNDDOWN {
    ( $x:expr ) => {
        $x & !($crate::riscv::PGSIZE-1)
    }
}

// level 2 1 0 -> 30 21 12
#[macro_export]
macro_rules! PXSHIFT {
    ( $level:expr) => {
        $crate::riscv::PGSHIFT + (9 * $level)
    }
}

// extract the three 9-bit page table indices from a virtual address.
#[macro_export]
macro_rules! PX {
    ( $level:expr, $va:expr) => {
        ($va as usize) >> PXSHIFT!($level) & $crate::riscv::PXMASK
    }
}

#[macro_export]
macro_rules! PTE2PA {
    ( $pte:expr ) => {
        (($pte >> 10) << 12)
    }
}

#[macro_export]
macro_rules! PA2PTE {
    ( $pa:expr ) => {
        (($pa >> 12) << 10)
    }
}


/// Machine Previous Privilege Mode
pub enum MPP {
    User = 0,
    Supervisor = 1,
    Machine = 3,
}
pub const MSTATUS_MPP_MASK: usize = 3 << 11; // previous mode.
pub const MSTATUS_MPP_M: usize = 3 << 11;
pub const MSTATUS_MPP_S: usize = 1 << 11;
pub const MSTATUS_MPP_U: usize = 0 << 11;
pub const MSTATUS_MIE: usize = 1 << 3; // machine-mode interrupt enable.

/// set MPP field
#[inline(always)]
pub fn set_mpp(mpp: MPP) {
    unsafe {
        let mut mstatus = r_mstatus();
        // mstatus.set_bits(11..13, mpp as usize);
        mstatus &= !MSTATUS_MPP_MASK;
        mstatus |= MSTATUS_MPP_S;

        w_mstatus(mstatus);
    }
}

#[inline(always)]
pub fn r_mstatus() -> usize {
    let mut x: usize;
    unsafe {
        asm!(
            "csrr {}, mstatus",
            out(reg) x,
        );
    }
    x
}

#[inline(always)]
pub fn w_mstatus(x: usize) {
    unsafe {
        asm!(
            "csrw mstatus, {0}",
            in(reg) x,
        );
    }
}

#[inline(always)]
pub fn w_satp(x: usize) {
    unsafe {
        asm!(
            "csrw satp, {0}",
            in(reg) x,
        );
    }
}

const SSIE: usize = 1 << 1; // software
const STIE: usize = 1 << 5; // timer
const SEIE: usize = 1 << 9; // external

#[inline(always)]
pub fn r_sie() -> usize {
    let mut x: usize;
    unsafe {
        asm!(
            "csrr {0}, sie",
            out(reg) x,
        );
    }
    x
}

// static inline void
#[inline(always)]
pub fn w_sie(x: usize) {
    unsafe {
        asm!(
            "csrw sie, {0}",
            in(reg) x,
        );
    }
}

/// enable all software interrupts
/// still need to set SIE bit in sstatus
pub unsafe fn intr_on() {
    let mut sie = r_sie();
    sie |= SSIE | STIE | SEIE;
    w_sie(sie);
}

#[inline(always)]
pub fn w_pmpaddr0(x: usize) {
    unsafe {
        asm!(
            "csrw pmpaddr0, {0}",
            in(reg) x,
        );
    }
}

#[inline(always)]
pub fn w_pmpcfg0(x: usize) {
    unsafe {
        asm!(
            "csrw pmpcfg0, {0}",
            in(reg) x,
        );
    }
}

// Machine-mode Interrupt Enable
pub const MIE_MEIE: usize = 1 << 11; // external
pub const MIE_MTIE: usize = 1 << 7; // timer
pub const MIE_MSIE: usize = 1 << 3; // software

#[inline(always)]
pub fn w_mie(x: usize) {
    unsafe {
        asm!(
            "csrw mie, {0}",
            in(reg) x,
        );
    }
}

#[inline(always)]
pub fn r_mie() -> usize {
    let x: usize;
    unsafe {
        asm!(
            "csrr {0}, mie",
            out(reg) x,
        );
    }
    x
}

#[inline(always)]
pub fn w_mepc(x: usize) {
    unsafe {
        asm!(
            "csrw mepc, {0}",
            in(reg) x,
        );
    }
}

#[inline(always)]
pub fn r_medeleg() -> usize {
    let mut x: usize;
    unsafe {
        asm!(
            "csrr {0}, medeleg",
            out(reg) x,
        );
    }
    return x;
}

#[inline(always)]
pub fn w_medeleg(x: usize) {
    unsafe {
        asm!(
            "csrw medeleg, {0}",
            in(reg) x,
        );
    }
}

#[inline(always)]
pub fn w_mideleg(x: usize) {
    unsafe {
        asm!(
            "csrw mideleg, {0}",
            in(reg) x,
        );
    }
}

#[inline(always)]
pub fn r_mhartid() -> usize {
    let mut x: usize;
    unsafe {
        asm!(
            "csrr {0}, mhartid",
            out(reg) x
        );
    }
    x
}

#[inline(always)]
pub fn w_tp(x: usize) {
    unsafe {
        asm!(
            "mv tp, {0}",
            in(reg) x
        );
    }
}

#[inline(always)]
pub fn vma() {
    unsafe {
        // the zero, zero means flush all TLB entries.
        asm!("sfence.vma zero, zero");
    }
}