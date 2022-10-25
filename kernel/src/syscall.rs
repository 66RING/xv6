use crate::proc::{PROC, myproc};
use crate::sysfile::sys_write;

//// System call numbers
const SYS_fork: usize = 1;
const SYS_exit: usize = 2;
const SYS_wait: usize = 3;
const SYS_pipe: usize = 4;
const SYS_read: usize = 5;
const SYS_kill: usize = 6;
const SYS_exec: usize = 7;
const SYS_fstat: usize = 8;
const SYS_chdir: usize = 9;
const SYS_dup: usize = 10;
const SYS_getpid: usize = 11;
const SYS_sbrk: usize = 12;
const SYS_sleep: usize = 13;
const SYS_uptime: usize = 14;
const SYS_open: usize = 15;
const SYS_write: usize = 16;
const SYS_mknod: usize = 17;
const SYS_unlink: usize = 18;
const SYS_link: usize = 19;
const SYS_mkdir: usize = 20;
const SYS_close: usize = 21;
// add-on
const SYS_yield: usize = 22;


// 用户态syscall协议: **通过a7寄存器传递系统调用号**
// sub entry {
//     my $name = shift;
//     print ".global $name\n";
//     print "${name}:\n";
//     print " li a7, SYS_${name}\n";
//     print " ecall\n";
//     print " ret\n";
// }


/// 从trapframe中读取下陷时保存的函数调用参数
/// TODO: 怎么访问到trapframe呢??
///     保存在proc结构的trapframe中
fn argraw(n: isize) -> usize {
    let p = myproc();
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

/// @param: n, 第几个参数
/// @param: ip, integer pointr?, 待赋值的参数
/// @return: 错误代码, -1出错
pub fn argint(n: isize, ip: &mut isize) -> isize {
  *ip = argraw(n) as isize;
  return 0;
}

/// Retrieve an argument as a pointer.
/// Doesn't check for legality, since
/// copyin/copyout will do that.
/// @param: n第几个参数
/// @param: ip地址保存者
/// @return: 错误代码, 小于0出错
pub fn argaddr(n: isize, ip: &mut usize) -> isize {
  *ip = argraw(n);
  return 0;
}


pub fn syscall() {
    let mut p = myproc();
    // 获取系统调用号
    let num  = p.trapframe.a7;
    // TODO: 简化, 应添加更多检测
    if num == SYS_write {
        p.trapframe.a0 = sys_write() as usize;
    } else {
        error!("unimplemented syscall {}\n", num);
        unimplemented!();
    }
}

