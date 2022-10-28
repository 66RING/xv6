// TODO: unused check
#![allow(unused)]

use crate::param::*;
use crate::trap::usertrapret;
use core::arch::asm;
use core::cell::{RefCell, RefMut};
use lazy_static::lazy_static;
use spin::Mutex;
use crate::riscv::*;

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

// FIXME: 临时, 待虚拟内存后删除硬编码
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

#[derive(Clone, Copy)]
pub enum ProcState {
    UNUSED,
    USED,
    SLEEPING,
    RUNNABLE,
    RUNNING,
    ZOMBIE,
}

#[derive(Clone, Copy)]
pub struct Proc {
    // TODO: TrapFrame should be some kind of reference or pointer in case of kernel stack overflow
    // TODO: wait for kalloc to implement
    // trapframe暂时不用保存
    pub trapframe: TrapFrame, // data page for trampoline.S
    pub context: Context,     // data page for trampoline.S
    pub state: ProcState,     // Process state
    pub kstack: usize,

    pub chan: usize,          // If non-zero, sleeping on chan, 时钟中断会检测各个proc是chan情况,
                              // 到点wakeup

    pub killed: i64,
    pub pid: i64,
}

impl Proc {
    pub fn zero_init() -> Self {
        Proc {
            context: Context::zero_init(),
            state: ProcState::RUNNABLE,
            kstack: 0,
            trapframe: TrapFrame::zero_init(),
            killed: 0,
            pid: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
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
    /*  16 */ pub kernel_trap: usize, // usertrap() TODO: 哪里用到了
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
        // TODO: xv6中sstatus怎么处理? 和rcore不同
        // let mut sstatus = sstatus::read();
        // sstatus.set_spp(SPP::User);
        let mut cx = Self::zero_init();
        cx.epc = entry;
        cx.set_sp(sp);
        cx
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

//impl ProcManager {
//    pub fn new() -> ProcManager {
//        // 获取总app数
//        let num_app = get_num_app();
//        // 创建tcb数组(未初始化)
//        let mut procs = [Proc {
//            context: Context::zero_init(),
//            state: ProcState::UNUSED,
//            kstack: 0,
//            trapframe: TrapFrame::zero_init(),
//            killed: 0,
//            pid: 0,
//        }; NPROC];
//        // 初始化每个任务 -> Ready & cx
//        //  **为每个任务的内核栈都伪造一个TrapFrame**
//        //  TrapContext.sp -> **用户栈**
//        //  TaskContext.sp -> 内核栈
//        //
//        //  __swtch切换任务的内核栈, 寄存器上下文
//        //  __userret切换内核栈到用户栈
//        for i in 0..num_app {
//            // TODO:
//            // procs[i].context = Context::goto_userret(init_app_cx(i));
//            // 因为xv6中trapframe不是保存在栈中, 所以直接给内核栈指针
//            procs[i].context = Context::goto_usertrapret(KERNEL_STACK[i].get_sp());
//            procs[i].state = ProcState::RUNNABLE;
//            procs[i].kstack = KERNEL_STACK[i].get_sp();
//            procs[i].pid = i as i64;

//            // TODO: 整理
//            procs[i].trapframe.epc = get_base_i(i);
//            procs[i].trapframe.sp = USER_STACK[i].get_sp();
//        }
//        // 返回全局任务管理器实例
//        ProcManager { procs, curr_id: 0 }
//    }
//}

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

/// 运行第一个用户程序
pub fn userinit() {
    // FIXME: 运行第一个程序, 程序退出后触发exit系统调用, 在运行下一个
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

/// TODO: 重新抽象
/// 将程序加载到对应的内存地址中
fn load_apps() {
    extern "C" {
        fn _num_app();
    }

    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    // TODO : review
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    // load app    // clear i-cache first
    unsafe {
        asm!("fence.i");
    }
    // load apps
    for i in 0..num_app {
        let base_i = get_base_i(i);
        // clear region
        (base_i..base_i + APP_SIZE_LIMIT)
            .for_each(|addr| unsafe { (addr as *mut u8).write_volatile(0) });
        // load app from data section to memory
        let src = unsafe {
            core::slice::from_raw_parts(app_start[i] as *const u8, app_start[i + 1] - app_start[i])
        };
        // 第i个app加载的base_i
        let dst = unsafe { core::slice::from_raw_parts_mut(base_i as *mut u8, src.len()) };
        dst.copy_from_slice(src);
    }
}

/// 初始化各个程序, 并为CPU附上初始程序
pub fn procinit() {
    // FIXME: just batch system for now
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
        // TODO:
        // procs[i].context = Context::goto_userret(init_app_cx(i));
        // 因为xv6中trapframe不是保存在栈中, 所以直接给内核栈指针
        procs[i].context = Context::goto_usertrapret(KERNEL_STACK[i].get_sp());
        procs[i].state = ProcState::RUNNABLE;
        procs[i].kstack = KERNEL_STACK[i].get_sp();
        procs[i].pid = i as i64;

        // TODO: 整理
        procs[i].trapframe.epc = get_base_i(i);
        procs[i].trapframe.sp = USER_STACK[i].get_sp();
    }

    // TODO: 为cpu附上初始程序
    let p = &mut procs[0];
    let c = mycpu();
    c.process = p as *mut Proc;
    p.state = ProcState::RUNNING;

    load_apps();
    println!("load_app done");
    // lazy_static, 第一次调用才触发初始化
    // 初始化进程的内核栈指针
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

/// TODO: 简化版: 调度下一个可运行的程序
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

