
/// fill each byte
pub fn memset(dst: usize, c: u8, n: usize) {
    // 使用指针制作一个rust slice
    let slice = unsafe { core::slice::from_raw_parts_mut(dst as *mut u8, n) };
    for i in slice {
        *i = c;
    }
}

