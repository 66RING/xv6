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
    syscall(SYS_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}


/// 功能：退出应用程序并将返回值告知批处理系统。
/// 参数：`xstate` 表示应用程序的返回值。
/// 返回值：该系统调用不应该返回。
/// syscall ID：93
pub fn sys_exit(xstate: i32) -> isize {
    syscall(SYS_EXIT, [xstate as usize, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(SYS_UPTIME, [0, 0, 0])
}

pub fn sys_yield() -> isize {
    syscall(SYS_YIELD, [0, 0, 0])
}

