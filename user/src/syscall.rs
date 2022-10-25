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


// 用户态调用系统调用
// 系统调用id和其他3个参数
fn syscall(id: usize, args: [usize; 3]) -> isize {
   let mut ret: isize;
   unsafe {
       core::arch::asm!(
           "ecall",
           inlateout("a0") args[0] => ret,
           in("a1") args[1],
           in("a2") args[2],
           in("a7") id
       );
   }
   ret
}

/// 功能：将内存中缓冲区中的数据写入文件。
/// 参数：`fd` 表示待写入文件的文件描述符；
///      `buf` 表示内存中缓冲区的起始地址；
///      `len` 表示内存中缓冲区的长度。
/// 返回值：返回成功写入的长度。
/// syscall ID：64
pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYS_write, [fd, buffer.as_ptr() as usize, buffer.len()])
}


/// 功能：退出应用程序并将返回值告知批处理系统。
/// 参数：`xstate` 表示应用程序的返回值。
/// 返回值：该系统调用不应该返回。
/// syscall ID：93
pub fn sys_exit(xstate: i32) -> isize {
    syscall(SYS_exit, [xstate as usize, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(SYS_uptime, [0, 0, 0])
}

pub fn sys_yield() -> isize {
    syscall(SYS_yield, [0, 0, 0])
}

