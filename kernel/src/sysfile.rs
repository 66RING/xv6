use crate::syscall::{argint, argaddr};

// TODO: 临时设置
const STDOUT: isize = 1;

/// 获取文件描述符
/// FIXME: 简化, 仅是int, 其他都是0
/// 选择第n个参数, pfd作为接收者接收fd, pf作为接收这接收file, 出错返回-1
pub fn argfd(n: isize, pfd: &mut isize, pf: isize) -> isize {
    if argint(n, pfd) < 0 {
        return -1;
    }
    0
}

/// 分别获取用户态的fd, buf, len参数
/// 返回写入的长度, 小于0出错
pub fn sys_write() -> isize {
    let mut fd = 0;
    let mut buf: usize = 0;
    let mut len: isize = 0;
    if argint(2, &mut len) < 0 || argint(0, &mut fd) < 0 {
        return -1;
    }
    if argaddr(1, &mut buf) < 0 {
        return -1;
    }
    match fd {
        STDOUT => {
            // 根据裸指针制作一个slice方便处理
            let slice = unsafe {core::slice::from_raw_parts(buf as *const u8, len as usize)};
            // 将传入的缓冲区的开始地址转换成&str
            let str = core::str::from_utf8(slice).unwrap();
            printf!("{}", str);
            len as isize
        }
        _ => {
            panic!("unsupported fd {}", fd);
        }
    }
}

