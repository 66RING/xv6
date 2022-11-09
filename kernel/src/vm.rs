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

    /// 仅用于查表
    /// va查表, 返回物理地址或返回0出错
    pub fn walkaddr(&mut self, va: usize) -> Option<usize> {
        if va > MAXVA {
            panic!("bad va");
        }
        if let Some(pte) = self.walk(va, false) {
            // 地址未分配
            if pte.data & PTE_V == 0 || pte.data & PTE_U == 0 {
                panic!("[debug] walkaddr");
                return None;
            }
            Some(PTE2PA!(pte.data))
        } else {
            None
        }
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
    unsafe { w_satp(MAKE_SATP!(&KERNEL_PAGETABLE as *const _ as usize)) };
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
pub fn mappages(pagetable: &mut PageTable, va: usize, pa: usize, size: usize, perm: usize) -> i32 {
    if size == 0 {
        panic!("mappages: size");
    }

    let mut pa = pa;
    let mut va_start = PGROUNDDOWN!(va);
    let va_end = PGROUNDDOWN!(va + size - 1);
    loop {
        if let Some(pte) = pagetable.walk(va_start, true) {
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

/// TODO: 仅加载initcode, 但这里还只是简单实现
/// 用户态虚拟内存初始化
/// 从src拷贝(因为src不可执行)到分配的物理页中, 然后从0开始创建虚拟内存
///     即, 代码拷贝到虚拟地址0处
pub fn uvminit(pgtbl: &mut PageTable, src: usize, sz: usize) {
    // 申请物理页
    let mem = kalloc();
    memset(mem, 0, PGSIZE);
    // 将物理页添加到页表完成映射
    // 从va 0开始映射, 标记为用户态可访问
    mappages(pgtbl, 0, mem, PGSIZE, PTE_R | PTE_W | PTE_X | PTE_U);
    // 将数据拷贝入物理页中
    memmove(mem, src, sz);
}

/// 扩展/收缩进程的地址空间: 从oldsz到newsz, 需要页对齐
/// 返回新地址空间或出错返回0
pub fn uvmalloc(pgtbl: &mut PageTable, oldsz: usize, newsz: usize) -> usize {
    if newsz < oldsz {
        return oldsz;
    }
    // 申请物理页, 覆盖oldsz .. newsz的虚拟地址空间
    let va_start = PGROUNDUP!(oldsz);
    for a in (va_start..newsz).step_by(PGSIZE) {
        let mem = kalloc();
        if mem == 0 {
            uvmdealloc(pgtbl, oldsz, newsz);
            return 0;
        }
        memset(mem, 0, PGSIZE);

        if mappages(pgtbl, a, mem, PGSIZE, PTE_R | PTE_W | PTE_X | PTE_U) < 0 {
            kfree(mem);
            uvmdealloc(pgtbl, oldsz, newsz);
            return 0;
        }
    }

    newsz
}

pub fn uvmdealloc(pgtbl: &mut PageTable, oldsz: usize, newsz: usize) -> usize {
    unimplemented!()
}

/// 将地址空间标记为用户态不可用的
/// exec中创建stack guard page
pub fn uvmclear(pgtbl: &mut PageTable, va: usize) {
    if let Some(pte) = pgtbl.walk(va, false) {
        pte.data &= !PTE_U;
    } else {
        panic!("uvmclear");
    }
}

/// 从0释放用户态地址空间
/// 将0..sz的地址空间取消映射
/// TODO:
pub fn uvmfree(pgtbl: &mut PageTable, sz: usize) {
    unimplemented!()
}

/// TODO:
pub fn uvmunmap(pgtbl: &mut PageTable, va: usize, npages: usize, do_free: bool) {
    unimplemented!()
}

/// 申请一物理页作为一级页表
pub fn uvmcreate() -> &'static mut PageTable {
    let p = kalloc();
    memset(p, 0, PGSIZE);
    unsafe {&mut *(p as *mut PageTable)}
}
