use crate::memlayout::CLINT_MTIME;
use crate::proc::{myproc, sched, ProcState};
use crate::syscall::argint;
use crate::trap::TICKS;
use core::time::Duration;

/// TODO: 简化: 退出当前进程, 然后调度下一个程序
/// 完全体应该包括资源回收等
pub fn sys_exit() -> usize {
    // TODO: 获取exit code
    let p = myproc();
    p.state = ProcState::ZOMBIE;
    sched();
    0
}

/// get current time from MMIO
/// TODO: review
pub fn sys_gettime() -> Duration {
    let mtime = CLINT_MTIME as *const u64;
    Duration::from_nanos(unsafe {mtime.read_volatile()} * 100)
}
