// TODO: unused check
#![allow(unused)]

use crate::param::*;
use core::arch::asm;
use lazy_static::lazy_static;
use spin::Mutex;

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
static KERNEL_STACK: [KernelStack; NPROC] = [
    KernelStack { data: [0; KERNEL_STACK_SIZE], };
    NPROC
];

static USER_STACK: [UserStack; NPROC] = [
    UserStack { data: [0; USER_STACK_SIZE], };
    NPROC
];

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, trap_cx: TrapFrame) -> usize {
        let trap_cx_ptr = (self.get_sp() - core::mem::size_of::<TrapFrame>()) as *mut TrapFrame;
        unsafe { *trap_cx_ptr = trap_cx; }
        trap_cx_ptr as usize
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}



#[derive(Clone, Copy)]
enum ProcState {
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
    // trapframe: TrapFrame, // data page for trampoline.S
    context: Context, // data page for trampoline.S
    state: ProcState, // Process state
    kstack: usize,
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

    // 创建一个从userret开始的上下文
    pub fn goto_userret(kstack_ptr: usize) -> Self {
        extern "C" { fn userret(); }
        let mut ctx = Self::zero_init();
        ctx.ra = userret as usize;
        ctx.sp = kstack_ptr;
        ctx
    }

}

#[derive(Clone, Copy, Default)]
struct TrapFrame {
    /*   0 */ kernel_satp: usize, // kernel page table
    /*   8 */ kernel_sp: usize, // top of process's kernel stack
    /*  16 */ kernel_trap: usize, // usertrap()
    /*  24 */ epc: usize, // saved user program counter
    /*  32 */ kernel_hartid: usize, // saved kernel tp
    /*  40 */ ra: usize,
    /*  48 */ sp: usize,
    /*  56 */ gp: usize,
    /*  64 */ tp: usize,
    /*  72 */ t0: usize,
    /*  80 */ t1: usize,
    /*  88 */ t2: usize,
    /*  96 */ s0: usize,
    /* 104 */ s1: usize,
    /* 112 */ a0: usize,
    /* 120 */ a1: usize,
    /* 128 */ a2: usize,
    /* 136 */ a3: usize,
    /* 144 */ a4: usize,
    /* 152 */ a5: usize,
    /* 160 */ a6: usize,
    /* 168 */ a7: usize,
    /* 176 */ s2: usize,
    /* 184 */ s3: usize,
    /* 192 */ s4: usize,
    /* 200 */ s5: usize,
    /* 208 */ s6: usize,
    /* 216 */ s7: usize,
    /* 224 */ s8: usize,
    /* 232 */ s9: usize,
    /* 240 */ s10: usize,
    /* 248 */ s11: usize,
    /* 256 */ t3: usize,
    /* 264 */ t4: usize,
    /* 272 */ t5: usize,
    /* 280 */ t6: usize,
}

impl TrapFrame {
    pub fn zero_init() -> Self {
        // default的另一种写法
        Self {
            ..Default::default()
        }
    }
    pub fn set_sp(&mut self, sp: usize) { self.sp = sp; }
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        // let mut sstatus = sstatus::read();
        // sstatus.set_spp(SPP::User);
        let mut cx = Self::zero_init();
        cx.epc = entry;
        cx.set_sp(sp);
        cx
    }
}

extern "C" {
    pub fn swtch(
        current_task_cx_ptr: *mut Context,
        next_task_cx_ptr: *const Context,
    );
}


// 应用管理器(PCB)
struct ProcManager {
    procs: [Proc; NPROC],
}

lazy_static! {
    static ref PROC: Mutex<ProcManager> = unsafe { Mutex::new(ProcManager::new()) };
}

// 在进程内核栈中创建一个trapframe, 以完成用户态切换
//      - 入口地址
//      - 用户栈指针
// return: 进程内核栈
pub fn init_app_cx(app_id: usize) -> usize {
    KERNEL_STACK[app_id].push_context(
        TrapFrame::app_init_context(get_base_i(app_id), USER_STACK[app_id].get_sp()))
}


impl ProcManager {
    pub fn new() -> ProcManager {
        // 获取总app数
        let num_app = get_num_app();
        // 创建tcb数组(未初始化)
        let mut procs = [Proc {
            context: Context::zero_init(),
            state: ProcState::UNUSED,
            kstack: 0,
        }; NPROC];
        // 初始化每个任务 -> Ready & cx
        //  **为每个任务的内核栈都伪造一个TrapFrame**
        //  TrapContext.sp -> **用户栈**
        //  TaskContext.sp -> 内核栈
        //
        //  __swtch切换任务的内核栈, 寄存器上下文
        //  __userret切换内核栈到用户栈
        for i in 0..num_app {
            procs[i].context = Context::goto_userret(init_app_cx(i));
            procs[i].state = ProcState::RUNNABLE;
        }
        // 返回全局任务管理器实例
        ProcManager { procs }
    }

    // FIXME: temporary
    pub fn run_first_proc(&self) {
        let mut procs = self.procs;
        let task0 = &mut procs[0];
        task0.state = ProcState::RUNNING;
        let next_task_cx_ptr = &task0.context as *const Context;
        // 运行第一个任务前并没有执行任何app，分配一个unused上下文
        let mut _unused = Context::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            swtch(
                &mut _unused as *mut Context,
                next_task_cx_ptr,
            );
        }
        panic!("unreachable in run_first_task!");
    }
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

// 加载第一个用户程序
pub fn userinit() {
    // FIXME: 运行第一个程序, 程序退出后触发exit系统调用, 在运行下一个
    PROC.lock().run_first_proc();
}

pub fn procinit() {
    // FIXME: just batch system for now
    println!("processs initializing");
    // lazy_static, 第一次调用才触发初始化
    // 初始化进程的内核栈指针
    let mut procs = PROC.lock().procs;
    for i in 0..procs.len() {
        procs[i].kstack = KERNEL_STACK[i].get_sp();
    }
}
