use crate::memlayout::*;
use crate::riscv::*;
use core::ptr;
use spin::Mutex;
use crate::string::*;

// first address after kernel. defined by kernel.ld.
extern "C" { fn _END(); }

struct Run {
    next: *mut Run,
}

struct Kmem {
    freelist: *mut Run,
}

static mut KMEM: Mutex<Kmem> = Mutex::new(Kmem{ freelist: ptr::null_mut() });
static mut PAGE_COUNT: Mutex<isize> = Mutex::new(0);

pub fn kinit() {
    freerange(_END as usize, PHYSTOP);
    println!("kinit {:#x} .. {:#x}", _END as usize, PHYSTOP);
}

fn freerange(pa_start: usize, pa_end: usize) {
    // 释放物理地址
    // 堆空间在链接脚本中有指出, end
    let mut p: usize = PGROUNDUP!(pa_start);
    while p + PGSIZE <= pa_end {
        // printf!("\rfreeing {:#x}/{:#x}", p, pa_end);
        kfree(p);
        p += PGSIZE;
    }
}

/// 释放页帧/物理地址, 插入freelist头
/// 毕竟以页为单位, 该物理地址应该是页对齐的
pub fn kfree(pa:usize) {
    // 如果没有页对齐, 说明不是一个合法页帧
    // 如果pa < end || pa > PHYSTOP说明不再规定的堆空间中
    if pa % PGSIZE != 0 || pa < _END as usize || pa > PHYSTOP {
        panic!("kfree");
    }

    // 清空页信息
    memset(pa, 1, PGSIZE);

    unsafe {
        let mut cnt = PAGE_COUNT.lock();
        *cnt += 1;
        let r = pa as *mut Run;
        // 创建freelsit节点r
        let mut kmem = KMEM.lock();
        (*r).next = kmem.freelist;
        kmem.freelist = r;
    }
}

/// 从freelist中(链表头)申请一页
pub fn kalloc() -> usize {
    unsafe {
        let mut kmem = KMEM.lock();
        let r = kmem.freelist;
        if r.is_null() {
            panic!("out of memory");
            // return 0;
        }
        let mut cnt = PAGE_COUNT.lock();
        *cnt -= 1;

        kmem.freelist = (*r).next;
        // 填充垃圾数据
        memset(r as usize, 5, PGSIZE);
        r as usize
    }
}

#[allow(unused)]
pub fn allocator_test() {
    let a = kalloc();
    assert_eq!(a%PGSIZE, 0);
    let b = kalloc();
    kfree(b);
    let c = kalloc();
    assert_eq!(c, b);
    kfree(c);
    kfree(a);
    unsafe { 
        assert_eq!(KMEM.lock().freelist as usize, PHYSTOP - PGSIZE);
        let cnt = *PAGE_COUNT.lock();
        assert_eq!(((PHYSTOP - _END as usize)/PGSIZE) as isize, cnt);
        println!("page count: {}", cnt);
    }
}
