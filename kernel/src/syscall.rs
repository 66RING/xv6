use crate::proc::{PROC_POOL, myproc};
use crate::sysfile::sys_write;
use crate::sysproc::sys_exit;

//// System call numbers
const SYS_FORK: usize = 1;
const SYS_EXIT: usize = 2;
const SYS_WAIT: usize = 3;
const SYS_PIPE: usize = 4;
const SYS_READ: usize = 5;
const SYS_KILL: usize = 6;
const SYS_EXEC: usize = 7;
const SYS_FSTAT: usize = 8;
const SYS_CHDIR: usize = 9;
const SYS_DUP: usize = 10;
const SYS_GETPID: usize = 11;
const SYS_SBRK: usize = 12;
const SYS_SLEEP: usize = 13;
const SYS_UPTIME: usize = 14;
const SYS_OPEN: usize = 15;
const SYS_WRITE: usize = 16;
const SYS_MKNOD: usize = 17;
const SYS_UNLINK: usize = 18;
const SYS_LINK: usize = 19;
const SYS_MKDIR: usize = 20;
const SYS_CLOSE: usize = 21;
// add-on
const SYS_YIELD: usize = 22;


/// 从trapframe中读取下陷时保存的函数调用参数
///     trapframe保存在堆中, 可以通过proc结构访问到
fn argraw(n: isize) -> usize {
    let mut p = myproc();
    let tf = p.trapframe;
    match n {
        0 => {
            tf.a0
        }
        1 => {
            tf.a1
        }
        2 => {
            tf.a2
        }
        3 => {
            tf.a3
        }
        4 => {
            tf.a4
        }
        5 => {
            tf.a5
        }
        _ => {
            panic!("argraw not valid");
        }
    }
}

/// 选择第n个参数, ip作为接收者, 出错返回-1
pub fn argint(n: isize, ip: &mut isize) -> isize {
  *ip = argraw(n) as isize;
  return 0;
}

/// Retrieve an argument as a pointer.
/// Doesn't check for legality, since
/// copyin/copyout will do that.
/// 选择第n个参数, ip作为接收者, 出错返回-1
pub fn argaddr(n: isize, ip: &mut usize) -> isize {
  *ip = argraw(n);
  return 0;
}


pub fn syscall() {
    let mut p = myproc();
    // 获取系统调用号
    let num  = p.trapframe.a7;
    // TODO: 简化版系统调用, 需要更多检查
    match num {
        SYS_WRITE => p.trapframe.a0 = sys_write() as usize,
        SYS_EXIT => p.trapframe.a0 = sys_exit() as usize,
        _ =>  panic!("invalid syscall {}", num),
    }
}

