pub const NCPU: usize = 1;
pub const STACK_SIZE: usize = 4096;

/// Maximum number of processes
pub const NPROC: usize = 8;


// FIXME: temporary for batch system
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;


