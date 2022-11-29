use crate::proc::{mycpu, CPU};
use crate::riscv::{intr_get, intr_off, intr_on, r_tp};
use core::cell::{UnsafeCell, Cell};
use core::ptr::NonNull;
use core::sync::atomic::{fence, AtomicBool, Ordering};
use core::ops::{Drop, Deref, DerefMut};

// TODO: which cell should I use? Cell, RefCell, UnsafeCell
// Cell, RefCell都没有办法在{不获取所有权/非可变引用}的情况下获取到&mut的
// 而很多情况我们只能拿到不可变引用且拿不到所有权: 如静态变量
// 而UnsafeCell可以通过get()获取到裸指针然后转换成&mut
pub struct Mutex<T> {
    pub locked: AtomicBool,

    pub name: &'static str,
    pub hartid: Cell<isize>,
    pub inner: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub const fn new(data: T, name: &'static str) -> Self {
        Self {
            locked: AtomicBool::new(false),
            // name没有实际作用, 可以用于调试
            name,
            hartid: Cell::new(0),
            inner: UnsafeCell::new(data),
        }
    }

    // tips:
    // xv6的实现不用为char* name分配堆空间, 为什么不会被释放?
    //  因为传入的的name是静态数据, 在代码段的, e.g. 直接传入的"kmem", 直接是&str
    // pub fn initlock(&mut self, name: &'static str) {
    //     self.name = name;
    //     self.locked = AtomicBool::new(false);
    //     self.hartid = 0;
    // }

    /// 在当前cpu上加锁
    pub fn acquire(&self) {
        push_off(); // 关中断, 防止二次加锁
        if self.holding() {
            panic!("acquire");
        }

        // TODO: 学习内存order
        // 执行上锁操作, spin!
        // compare_and_swap已废弃, 使用compare_exchange
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {}

        // Tell the C compiler and the processor to not move loads or stores
        // past this point, to ensure that the critical section's memory
        // references happen strictly after the lock is acquired.
        // On RISC-V, this emits a fence instruction.
        // 防止缓存预取, 防止spectre bug
        fence(Ordering::SeqCst);

        // Record info about lock acquisition for holding() and debugging.
        self.hartid.set(r_tp() as isize);
    }

    pub fn release(&self) {
        // 持有锁才可以释放
        assert_eq!(self.holding(), true);

        self.hartid.set(-1);

        // Tell the C compiler and the CPU to not move loads or stores
        // past this point, to ensure that all the stores in the critical
        // section are visible to other CPUs before the lock is released,
        // and that loads in the critical section occur strictly before
        // the lock is released.
        // On RISC-V, this emits a fence instruction.
        // 防止spectre
        fence(Ordering::SeqCst);

        // 释放锁
        self.locked.store(false, Ordering::Release);

        // 开启中断
        pop_off();
    }

    pub fn holding(&self) -> bool {
        self.locked.load(Ordering::Relaxed) && self.hartid.get() == r_tp() as isize
    }

    // 不能针对&mut, 因为有些数据的没有&mut的, 比如一些静态数据
    pub fn lock(&self) -> MutexGuard<T> {
        self.acquire();
        MutexGuard {
            lock: self,
            inner: unsafe { &mut *self.inner.get() },
        }
    }
}

// Sync trait指示数据被锁保护: 可以安全的在线程间传递, 谁申请锁成功了谁就有所有权
unsafe impl<T: Send> Sync for Mutex<T> {}

pub struct MutexGuard<'a, T> {
    lock: &'a Mutex<T>,
    inner: &'a mut T,
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}
impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.release();
    }
}

/// push_off/pop_off are like intr_off()/intr_on() except that they are matched:
/// it takes two pop_off()s to undo two push_off()s.  Also, if interrupts
/// are initially off, then push_off, pop_off leaves them off.
/// 我们可能要同时申请不同类型的锁,
/// push_off/pop_off保证在所有的锁都释放时才恢复(因为页可能关这中断拿锁)中断
pub fn push_off() {
    // 记录老的中断开启状态
    let old = intr_get();

    // 关中断
    intr_off();
    let mut cpu = mycpu();
    // 记录原始中断信息
    if cpu.noff == 0 {
        cpu.intena = old;
    }
    cpu.noff += 1;
}

pub fn pop_off() {
    let cpu = mycpu();
    // 开中断和关中断应该是成对出现的
    if intr_get() {
        panic!("pop_off - interruptible");
    }
    // push_off 和 pop_off成对出现, 不可能出现noff<1的情况
    if cpu.noff < 1 {
        panic!("pop_off");
    }
    cpu.noff -= 1;
    if cpu.noff == 0 && cpu.intena {
        intr_on();
    }
}
