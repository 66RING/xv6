use crate::kalloc::*;
use crate::memlayout::*;
use crate::proc::*;
use crate::riscv::*;
use crate::string::*;
use core::ptr::null_mut;
use lazy_static::lazy_static;

// entry数目等于PAGE_SIZE / 64bit(sizeof(usize)) = 4096 / 8
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; ENTRY_NUMS],
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PageTableEntry {
    data: usize,
}

impl PageTableEntry {
    pub const fn zero() -> PageTableEntry {
        Self { data: 0 }
    }
}

impl PageTable {
    pub const fn zero() -> Self {
        Self {
            entries: [PageTableEntry::zero(); ENTRY_NUMS],
        }
    }
}

/// TODO: boxed
/// WARN: 小心被move走
pub static mut KERNEL_PAGETABLE: PageTable = PageTable::zero();

impl PageTable {
    // TODO: 合并到lazy_static中, 初次访问自动初始化
    // KERNEL_PAGETABLE = kvmmake();
    /// 映射一系列地址空间到内核空间(内核页表)
    pub fn kvmmake(&mut self) {
        let mut kpgtbl = self;
        extern "C" {
            fn etext();
            fn trampoline();
        }

        // uart registers
        kvmmap(&mut kpgtbl, UART0, UART0, PGSIZE, PTE_R | PTE_W);

        // virtio mmio disk interface
        kvmmap(&mut kpgtbl, VIRTIO0, VIRTIO0, PGSIZE, PTE_R | PTE_W);

        // PLIC
        kvmmap(&mut kpgtbl, PLIC, PLIC, 0x400000, PTE_R | PTE_W);

        // map kernel text executable and read-only.
        kvmmap(
            &mut kpgtbl,
            KERNBASE,
            KERNBASE,
            etext as usize - KERNBASE,
            PTE_R | PTE_X,
        );

        // map kernel data and the physical RAM we'll make use of.
        kvmmap(
            &mut kpgtbl,
            etext as usize,
            etext as usize,
            PHYSTOP - etext as usize,
            PTE_R | PTE_W,
        );

        // map the trampoline for trap entry/exit to
        // the highest virtual address in the kernel.
        kvmmap(
            &mut kpgtbl,
            TRAMPOLINE,
            trampoline as usize,
            PGSIZE,
            PTE_R | PTE_X,
        );

        // map kernel stacks
        proc_mapstacks(&mut kpgtbl);
    }

    /// 翻译虚拟地址va, 利用当前页表获取到最后一级的PTE
    /// alloc == true时表示为va分配PTE, 最后返回该PTE
    pub fn walk(&mut self, va: usize, alloc: bool) -> Option<&mut PageTableEntry> {
        if va > MAXVA {
            panic!("walk");
        }

        let mut pagatable = self;
        for level in (1..3).rev() {
            let mut pte = &mut pagatable.entries[PX!(level, va)];
            if pte.data & PTE_V != 0{
                // 如果已未分配, 查找下一级
                pagatable = unsafe { &mut *(PTE2PA!(pte.data) as *mut PageTable) };
            } else {
                if !alloc {
                    // 如果不分配则返回None, walk失败
                    return None;
                } else {
                    // 如果需要分配则先分配，然后添加到页表中
                    let new_page_pa = kalloc();
                    if new_page_pa == 0 {
                        return None;
                    }
                    memset(new_page_pa, 0, PGSIZE);
                    pagatable = unsafe { &mut *(new_page_pa as *mut PageTable) };
                    pte.data = PA2PTE!(new_page_pa) | PTE_V;
                }
            }
        }
        // 最后返回最后一级索引的结果, 即物理页
        Some(&mut pagatable.entries[PX!(0, va)])
    }
}

/// 初始化内核页表
pub fn kvminit() {
    unsafe {
        KERNEL_PAGETABLE.kvmmake();
    }
}

/// 设置内核页表且启动分页
pub fn kvminithart() {
    unsafe { w_satp(MAKE_SATP!(&KERNEL_PAGETABLE)) };
    sfence_vma();
}

/// TODO: enum抽象一下权限perm
/// TODO: 检查以下perm的类型
/// 将sz大小的pa映射到页表对应的va中
pub fn kvmmap(kpgtbl: &mut PageTable, va: usize, pa: usize, sz: usize, perm: usize) {
    if mappages(kpgtbl, va, pa, sz, perm) != 0 {
        panic!("kvmmap");
    }
}

/// TODO:
/// 将[va..va+sz]映射到[pa..pa+sz]
///     walk(tbl), 填写pte
fn mappages(kpgtbl: &mut PageTable, va: usize, pa: usize, size: usize, perm: usize) -> i32 {
    if size == 0 {
        panic!("mappages: size");
    }

    let mut pa = pa;
    let mut va_start = PGROUNDDOWN!(va);
    let va_end = PGROUNDDOWN!(va + size - 1);
    loop {
        if let Some(pte) = kpgtbl.walk(va_start, true) {
            if (*pte).data & PTE_V != 0 {
                panic!("mappages: remap");
            }
            (*pte).data = PA2PTE!(pa) | perm | PTE_V;
            if va_start == va_end {
                break;
            }
            va_start += PGSIZE;
            pa += PGSIZE;
        } else {
            return -1;
        }
    }
    0
}

/// 用户态虚拟内存初始化
pub fn uvminit(pgtbl: &mut PageTable, src: usize, sz: usize) {

    unimplemented!()
}
