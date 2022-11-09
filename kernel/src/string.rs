
/// fill each byte
pub fn memset(dst: usize, c: u8, n: usize) {
    // 使用指针制作一个rust slice
    let slice = unsafe { core::slice::from_raw_parts_mut(dst as *mut u8, n) };
    for i in slice {
        *i = c;
    }
}

/// 内存拷贝
/// 先自己实现体会一下
/// 返回移动后的目标地址
pub fn memmove(dst: usize, src: usize, n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    let src = unsafe {
        core::slice::from_raw_parts(src as *const u8, n)
    };
    let dst2 = unsafe { core::slice::from_raw_parts_mut(dst as *mut u8, src.len()) };
    dst2.copy_from_slice(src);
    dst
}
