use crate::syscall::{argint, argaddr};

// TODO: 临时设置
const STDOUT: isize = 1;

/// 获取文件描述符
/// FIXME: 简化, 仅是int, 其他都是0
/// @param: n
/// @param: int pfd , TODO:
/// @param: file pf, TODO:
/// @return: 错误代码, 小于0出错
pub fn argfd(n: isize, pfd: &mut isize, pf: isize) -> isize {
    if argint(n, pfd) < 0 {
        return -1;
    }
    0
}

/// @param: fd, buf, len
/// @return: 写入的长度, 小于0出错
pub fn sys_write() -> isize {
    // TODO: rust 怎么不初始化
    let mut fd = 0;
    // TODO: usize as raw pointer
    let mut buf: usize = 9;
    let mut len: usize = 0;
    if argint(0, &mut fd) < 0 || argint(2, &mut (len as isize)) < 0 {
        return -1;
    }
    if argaddr(1, &mut buf) < 0 {
        return -1;
    }
    match fd {
        STDOUT => {
            let slice = unsafe {core::slice::from_raw_parts(buf as *const u8, len)};
            // 将传入的缓冲区的开始地址转换成&str
            let str = core::str::from_utf8(slice).unwrap();
            println!("{}", str);
            len as isize
        }
        _ => {
            unimplemented!()
        }
    }
}

