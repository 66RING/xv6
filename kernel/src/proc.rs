// TODO: unused check
#![allow(unused)]

use crate::param::*;
use crate::string::*;
use crate::trap::usertrapret;
use crate::vm::PageTable;
use core::arch::asm;
use core::cell::{RefCell, RefMut};
use lazy_static::lazy_static;
use spin::Mutex;
use crate::riscv::*;
use crate::memlayout::*;
use crate::kalloc::*;
use crate::vm::*;

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

// FIXME: 临时, 硬编码进程内核栈空间
static KERNEL_STACK: [KernelStack; NPROC] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; NPROC];

static USER_STACK: [UserStack; NPROC] = [UserStack {
    data: [0; USER_STACK_SIZE],
}; NPROC];

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    /// 将TrapFrame压入内核栈
    /// @return: trapframe压栈后内核栈指针
    pub fn push_context(&self, trap_cx: TrapFrame) -> usize {
        let trap_cx_ptr = (self.get_sp() - core::mem::size_of::<TrapFrame>()) as *mut TrapFrame;
        unsafe {
            *trap_cx_ptr = trap_cx;
        }
        trap_cx_ptr as usize
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcState {
    UNUSED,
    USED,
    SLEEPING,
    RUNNABLE,
    RUNNING,
    ZOMBIE,
}

impl Default for ProcState {
    fn default() -> Self {
        ProcState::UNUSED
    }
}

#[derive(Clone, Copy, Default)]
pub struct Proc {
    // TODO: TODO: TrapFrame指针, kalloc分配
    // trapframe暂时不用保存
    pub trapframe: Option<*mut TrapFrame>, // data page for trampoline.S
    pub context: Context,     // data page for trampoline.S
    pub state: ProcState,     // Process state
    pub kstack: usize,        // TODO: kernel stack page number
    pub pagetable: Option<*mut PageTable>,

    pub killed: i64,
    pub pid: i64,
    pub sz: usize,
}

// TODO: 线程安全
unsafe impl Send for Proc {}

impl Proc {
    pub fn zero_init() -> Self {
        Proc {
            context: Context::zero_init(),
            state: ProcState::UNUSED,
            kstack: 0,
            trapframe: None,
            killed: 0,
            ..Default::default()
        }
    }

    pub fn trapframe(&self) -> Option<&'static TrapFrame> {
        unsafe { Some(&*self.trapframe.unwrap()) }
    }
    pub fn trapframe_mut(&mut self) -> Option<&'static mut TrapFrame> {
        unsafe { Some(&mut *self.trapframe.unwrap()) }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Context {
    ra: usize,
    sp: usize,

    // callee-saved
    s0: usize,
    s1: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
}

impl Context {
    pub fn zero_init() -> Self {
        Default::default()
    }

    /// 创建一个从usertrapret开始的上下文
    pub fn goto_usertrapret(kstack_ptr: usize) -> Self {
        let mut ctx = Self::zero_init();
        // swtch返回后根据ra寄存器进入usertrapret
        ctx.ra = usertrapret as usize;
        // swtch切换到对应进程的栈空间
        ctx.sp = kstack_ptr;
        ctx
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct TrapFrame {
    /*   0 */ pub kernel_satp: usize, // kernel page table
    /*   8 */ pub kernel_sp: usize, // top of process's kernel stack
    /*  16 */ pub kernel_trap: usize, // usertrap(),
                                      //uservec根据这里记录的内容跳转handler, 即usertrap
    /*  24 */ pub epc: usize, // saved user program counter
    /*  32 */ pub kernel_hartid: usize, // saved kernel tp
    /*  40 */ pub ra: usize,
    /*  48 */ pub sp: usize,
    /*  56 */ pub gp: usize,
    /*  64 */ pub tp: usize,
    /*  72 */ pub t0: usize,
    /*  80 */ pub t1: usize,
    /*  88 */ pub t2: usize,
    /*  96 */ pub s0: usize,
    /* 104 */ pub s1: usize,
    /* 112 */ pub a0: usize,
    /* 120 */ pub a1: usize,
    /* 128 */ pub a2: usize,
    /* 136 */ pub a3: usize,
    /* 144 */ pub a4: usize,
    /* 152 */ pub a5: usize,
    /* 160 */ pub a6: usize,
    /* 168 */ pub a7: usize,
    /* 176 */ pub s2: usize,
    /* 184 */ pub s3: usize,
    /* 192 */ pub s4: usize,
    /* 200 */ pub s5: usize,
    /* 208 */ pub s6: usize,
    /* 216 */ pub s7: usize,
    /* 224 */ pub s8: usize,
    /* 232 */ pub s9: usize,
    /* 240 */ pub s10: usize,
    /* 248 */ pub s11: usize,
    /* 256 */ pub t3: usize,
    /* 264 */ pub t4: usize,
    /* 272 */ pub t5: usize,
    /* 280 */ pub t6: usize,
}

impl TrapFrame {
    pub fn zero_init() -> Self {
        // default的另一种写法
        Self {
            ..Default::default()
        }
    }
    pub fn set_sp(&mut self, sp: usize) {
        self.sp = sp;
    }
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut tf = Self::zero_init();
        tf.epc = entry;
        tf.set_sp(sp);
        tf
    }
}

extern "C" {
    pub fn swtch(current_task_cx_ptr: *mut Context, next_task_cx_ptr: *const Context);
}

static mut CPUS: [CPU; NCPU] = [CPU::zero(); NCPU];

pub struct CPU {
    process: *mut Proc,
}

impl CPU {
    pub const fn zero() -> Self {
        Self {
            process: core::ptr::null_mut(),
        }
    }
}

// 所有进程
lazy_static! {
    pub static ref PROC_POOL: Mutex<[Proc; NPROC]> =
        unsafe { Mutex::new([Proc::zero_init(); NPROC]) };
}

/// 在进程内核栈中创建一个trapframe, 以完成用户态切换
///      trapframe中记录了: 入口地址, 用户栈指针
/// return: 进程内核栈
pub fn init_app_cx(app_id: usize) -> usize {
    KERNEL_STACK[app_id].push_context(TrapFrame::app_init_context(
        get_base_i(app_id),
        USER_STACK[app_id].get_sp(),
    ))
}

// 我们用户程序的编译脚本中的协议规定了加载地址将0x20000(APP_SIZE_LIMIT)
// 从而简单计算出加载地址
fn get_base_i(app_id: usize) -> usize {
    APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

/// 加载运行第一个用户程序
/// 制作进程用户态地址空间
pub fn userinit() {

    extern "C" { fn _num_app(); }

    // 获取app数
    let num_app = get_num_app();
    // 协议规定第一的usize是数量, 后续是各个app的起始地址
    let num_app_ptr = _num_app as usize as *const usize;
    // 将app起始地址创建成数组
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    // load app    // clear i-cache first
    unsafe {
        asm!("fence.i");
    }

    // TODO: 这里一次性加载所有, 应该仅加载init程序, 待exec实现
    // TODO: 和myload_app大量重复
    for i in 0..num_app {
        unsafe {
            // 获取PCB中一个可用进程位
            let mut p = allocproc();
            let p = &mut *p;
            let size = app_start[i+1] - app_start[i];
            assert!(p.pagetable.is_some());
            let pgtbl = &mut *(*p).pagetable.unwrap();
            // uvminit(pgtbl, app_start[i], size);
            p.sz = size;
            // TODO: 丑
            p.trapframe_mut().unwrap().epc = 0; // 代码从va 0开始

            // 边界申请两页作为用户栈, 第一页做guard, 第二页才是用户栈
            // Allocate two pages at the next page boundary.
            // Use the second as the user stack.
            let sz = PGROUNDUP!(size);
            // 申请用户栈
            let newsz = uvmalloc(pgtbl, sz, sz + 2*PGSIZE);
            if newsz == 0 {
                unimplemented!();
            }
            uvmclear(pgtbl, newsz-2*PGSIZE);
            // ERROR: BUG!!!!!!!!!!!!!!!!!!1
            // let sp = newsz - PGSIZE;
            let sp = newsz;

            // 设置用户栈, "代码段"之后
            p.trapframe_mut().unwrap().sp = sp;
            p.state = ProcState::RUNNABLE;
        }
    }



    // allocate one user page and copy init's instructions
    // and data into it.
    // uvminit(p.pagetable, initcode, sizeof(initcode));
    // p->sz = PGSIZE;
    // 
    // // prepare for the very first "return" from kernel to user.
    // p->trapframe->epc = 0;      // user program counter
    // p->trapframe->sp = PGSIZE;  // user stack pointer
    // 
    // safestrcpy(p->name, "initcode", sizeof(p->name));
    // p->cwd = namei("/");
    // 
    // p->state = RUNNABLE;




    let task0 = myproc();
    let next_task_cx_ptr = &task0.context as *const Context;
    // 运行第一个任务前并没有执行任何app，分配一个unused上下文
    let mut _unused = Context::zero_init();
    // before this, we should drop local variables that must be dropped manually
    unsafe {
        swtch(&mut _unused as *mut Context, next_task_cx_ptr);
    }
    panic!("unreachable in run_first_task!");
}

/// 返回一个UNUSED的proc
///  如果找到则初始化它的状态
///  初始化pid, state, trapframe, context
fn allocproc() -> *mut Proc {
    let mut p: &mut Proc;
    let mut procs = PROC_POOL.lock();
    for i in 0..NPROC {
        if procs[i].state == ProcState::UNUSED {
            p = &mut procs[i];

            // 初始化进程状态
            p.state = ProcState::USED;
            // 分配pid
            p.pid = i as i64;
            // 分配trapframe
            // TODO: 暂时不用分配
            // p.trapframe = Some(*kalloc() as *const TrapFrame);
            // 分配pagetable
            // p.pagetable = proc_pagetable(p);
            // 初始化context
            // TODO: 不在这
            // memset(&p.context as *const _ as usize, 0, core::mem::size_of::<Context>());

            // TODO: review
            // p.context.ra = get_base_i(i);
            // 内核栈设置
            p.context.sp = p.kstack + PGSIZE;

            return p;
        }
    }

    // 如果没找到可用的proc slot, 直接诶panic
    panic!("run out of proc");
}

/// TODO: 重新抽象
/// 将程序加载到对应的内存地址中
fn load_apps() {
    extern "C" {
        fn _num_app();
    }

    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    // load app    // clear i-cache first
    unsafe {
        asm!("fence.i");
    }
    let mut procs = PROC_POOL.lock();
    // load apps
    for i in 0..num_app {
        // let base_i = get_base_i(i);
        // // clear region
        // (base_i..base_i + APP_SIZE_LIMIT)
        //     .for_each(|addr| unsafe { (addr as *mut u8).write_volatile(0) });
        // // load app from data section to memory
        // let src = unsafe {
        //     core::slice::from_raw_parts(app_start[i] as *const u8, app_start[i + 1] - app_start[i])
        // };
        // // 第i个app加载的base_i
        // let dst = unsafe { core::slice::from_raw_parts_mut(base_i as *mut u8, src.len()) };
        // dst.copy_from_slice(src);

        myload_app(&mut procs[i], app_start[i], app_start[i+1] - app_start[i]);
    }
}

/// 初始化各个程序, 并为CPU附上初始程序 TODO: recomment
pub fn procinit() {
    let mut procs = PROC_POOL.lock();
    let num_app = get_num_app();
    // 初始化每个任务 -> Ready & cx
    //  **为每个任务的内核栈都伪造一个TrapFrame**
    //  TrapContext.sp -> **用户栈**
    //  TaskContext.sp -> 内核栈
    //
    //  __swtch切换任务的内核栈, 寄存器上下文
    //  __userret切换内核栈到用户栈
    for i in 0..num_app {
        // 因为xv6中trapframe不是保存在栈中, 所以直接给内核栈指针
        // TODO: dulplicated
        // procs[i].context = Context::goto_usertrapret(PGSIZE + KSTACK!(&procs[i] as *const _ as usize - &*procs as *const _ as usize));
        procs[i].context.ra = usertrapret as usize;
        procs[i].state = ProcState::UNUSED;
        // TODO: 使用PCB中的索引计算出进程内核栈 **页帧**, kstack + PGSIZE才得到内核栈起始地址
        procs[i].kstack = KSTACK!(&procs[i] as *const _ as usize - &*procs as *const _ as usize);
        procs[i].pid = i as i64;
        // TODO: 暂时在这, 应该是在allocproc的
        let tf = kalloc();
        memset(tf, 0, PGSIZE);
        procs[i].trapframe = Some(unsafe {tf as *mut TrapFrame});

        // 初始化程序第一次启动时trapframe
        // 程序将拷贝到va 0处
        // 后续初始化时再申请栈空间
            // procs[i].trapframe.epc = get_base_i(i);
            // procs[i].trapframe.sp = USER_STACK[i].get_sp();
    }

    // 添加到当前CPU上方便后续运行和访问
    let p = &mut procs[0];
    let c = mycpu();
    c.process = p as *mut Proc;
    p.state = ProcState::UNUSED;
    // TODO: 这里手动drop, 有没有更好的设计
    drop(procs);

    load_apps();
    println!("load_app done");
}

fn myload_app(p: &mut Proc, src: usize, sz: usize) {
    // 创建进程页表: 映射trampoline和trapframe
    if let Some(pagetable) = proc_pagetable(p) {
        // 申请用户态地址空间, 标记为W | R | U
        p.pagetable = Some(pagetable);
        let pagetable = unsafe {&mut *pagetable};
        let newsz = uvmalloc(pagetable, 0, sz);

        // TODO: loadseg
        // 读取程序信息, 从0填充va
        for a in (0..sz).step_by(PGSIZE) {
            if let Some(pa) = pagetable.walkaddr(a) {

                // 去读数据载入pa
                // 不足一页时不载一页
                let n = if sz - a < PGSIZE {
                    sz - a
                } else {
                    PGSIZE
                };
                memmove(pa, src + a, n);
            } else {
                panic!("va not exist");
            }
        }
    } else {
        panic!("fail to load");
    }

    // unimplemented!()
}

/// 通过CPU di寄存器获取当前cpu
pub fn mycpu() -> &'static mut CPU {
    unsafe { &mut CPUS[r_tp()] }
}

/// 获取当前cpu进程
pub fn myproc() -> &'static mut Proc {
    let c = mycpu();
    unsafe { &mut(*c.process) }
}

/// TODO: 简化版: 简单调度下一个可运行的程序
/// 应该的切换到scheduler, 这里直接调度下一个
pub fn sched() {
    let p = myproc();
    let app_num = get_num_app();
    let mut procs = PROC_POOL.lock();
    let next_id = (p.pid + 1) as usize;
    if next_id >= app_num {
        panic!("run out of process");
    }
    let next_proc = &mut procs[next_id];
    let mut c = mycpu();
    c.process = next_proc as *mut Proc;
    p.state = ProcState::RUNNING;

    let old_ctx = &mut p.context as *mut Context;
    let next_ctx = &next_proc.context as *const Context;
    drop(procs);

    unsafe {
        swtch(old_ctx, next_ctx);
    }
    unreachable!();
}

/// 为进程映射内核栈空间
/// 根据在PROC_POOL中的索引计算出各个进程栈空间的虚拟地址
/// 然后各申请一页作为内核栈
pub fn proc_mapstacks(kpgtbl :&mut PageTable) {
    let num_app = get_num_app();
    let mut procs = PROC_POOL.lock();
    for i in 0..num_app {
        // 通过index计算出进程的内核栈空间, 做个栈空间随机化
        // TODO: 重新设计计算方法
        let va = KSTACK!(&procs[i] as *const _ as usize - &*procs as *const _ as usize);
        // 为内核栈分配一页空间
        let new_page_pa = kalloc();
        memset(new_page_pa, 0, PGSIZE);
        unsafe {
            // TODO: kill this unsafe
            kvmmap(kpgtbl, va, new_page_pa, PGSIZE, PTE_R | PTE_W);
        }
    }
}

/// 创建进程页表
///     映射trampoline
///     通过p.trapframe映射trapframe
/// FIXME:: ruxt怎么传递指针呢
/// TODO: 传引用
pub fn proc_pagetable(p: &mut Proc) -> Option<*mut PageTable> {
    let mut pgtbl = uvmcreate();
    extern "C" {
        fn trampoline();
    }

    // TODO:
    // 映射trampoline跳板空间
    // map the trampoline code (for system call return)
    // at the highest user virtual address.
    // only the supervisor uses it, on the way
    // to/from user space, so not PTE_U.
    // TODO: 看注释，为何不需要PTE_U
    if mappages(pgtbl, TRAMPOLINE, trampoline as usize, PGSIZE, PTE_X | PTE_R) < 0 {
        uvmfree(pgtbl, 0);
        return None;
    }

    // 映射trapframe, trapframe映射到trampoline正下方
    // 因为上下文切换需要访问trapframe, 而当页表切换后不再可以直接&取址了
    // 需要通过trampoline计算出相对位置
    if mappages(pgtbl, TRAPFRAME, p.trapframe.unwrap() as *const _ as usize, PGSIZE, PTE_R | PTE_W) < 0 {
        uvmunmap(pgtbl, TRAMPOLINE, 1, false);
        uvmfree(pgtbl, 0);
        return None;
    }

    Some(pgtbl)
}
