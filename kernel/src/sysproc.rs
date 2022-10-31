use crate::memlayout::CLINT_MTIME;
use crate::proc::{myproc, sched, ProcState};
use crate::syscall::argint;
use crate::trap::TICKS;
use core::time::Duration;

/// 退出当前进程, 然后调度下一个程序
/// TODO: 完全体应该包含获取exit代码, 资源回收等
pub fn sys_exit() -> usize {
    let p = myproc();
    p.state = ProcState::ZOMBIE;
    sched();
    0
}

