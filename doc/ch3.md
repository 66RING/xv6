## ch3 地址空间

### 页帧分配器

> 使用release模式编译

- `alloc`
- `kfree`

### 虚拟内存分页

- 内核态地址空间映射
- trampoline机制
    * trampoline映射到物理内存最高处(某一协定的指定地址), 该映射位置在用户态和内核态是相同的
    * 之所以需要trampoline是因为需要同时切换以下三个空间: 页表, 栈, pc。用户态和内核态映射相同位置的策略就可以完成pc和页表的同时切换

开启虚存后的, 注意以下几点

- 内核栈映射
    * `proc_mapstacks` -> `KSTACK!()`
- ra, 入口地址
    * 用户态程序入口地址
        + 直接拷贝到va 0
        + `trapframe.epc`记录
        + 内核切用户态
    * 上下文切换后ra返回地址
        + trapframe的相对trampoline的映射位置
        + `context.ra`记录
        + 仍在内核态
- 用户栈怎么计算
    * 用户程序代码段之后的`2xPGSIZE`的空间作为用户栈
    * 用户栈之后就是用户线性增长的heap空间
    * [the xv6 book: Figure 2.3: Layout of a process’s virtual address space](https://pdos.csail.mit.edu/6.S081/2020/xv6/book-riscv-rev1.pdf)


### Spinlock

- 上锁
    * `push_off`关闭中断
    * 预防重复上锁导致的死锁: 上锁前检测是否持有锁, 应该是不持有锁的
    * 使用原子CAS指令完成上锁标记, 如果失败则spin: `while`
        + `AtomicBool:compare_exchange()`
    * spin后添加"内存屏障", 防止cpu预取导致spectre漏洞
    * 最后记录cpuid(hartid), 用于判断当前cpu是否持有锁
- 解锁
    * cpuid置无效
    * "内存屏障"防spectre
    * 原子操作将锁标记清除
    * `pop_off`恢复中断
- `push_off`/`pop_off`
    * push/pop用于记录锁调用栈的深度, 因为上锁过程需要关闭中断, 并且期间我们可能会申请多种类型的锁"多次关中断", 但是在释放过程中我们希望是最后一个锁释放后才恢复中断, 否则将会导致持有锁的过程中中断发送
    * 实现就是调用深度`noff ++--`, 当`noff`为0时才开关中断
- `holding`判断是否持有锁
    * 即判断当前cpu是否是持有锁的cpu, 并且判断锁标记是否上锁`locked`
- rust式锁
    * cpp和rust都有锁(MutexGuard)自动释放功能, 实现方法就是离开作用域析构时自动解锁
    * rust中我们使用一个wrapper结构体`MutexGuard`
        + 申请锁时自动加锁并返回该结构体
        + 为该结构体实现deref, 自动对内部数据解引用, 因此与普通引用用法相同
        + 为该结构体实现drop, 在其生命周期结束时自动释放锁


### 栈空间

- 内核栈
    * 内核栈事先(`kvmmake`)会根据进程的idx(`KSTACK!()`)固定映射在内核空间
    * 注意栈的增长方向, `KSTACK!()`算出的是栈空间所处的物理地址, 而由于栈是从高到低增长的, 所以`KSTACK!()`的结果应该是内核栈的结束位置
        + 计算得到的内核栈页帧地址保存到`proc.kstack`中
        + 初始化内核栈时应该将**栈指针sp赋值为kstack+PGSIZE, 指向栈开始**
        + 内核栈由`context.sp`指示
        + 用户栈右`trapframe.sp`指示
- 用户栈
    * 用户栈空间则是紧接着代码段后的一页??
    * 用户栈右`trapframe.sp`指示
- `trapframe.sp` vs `trapframe.kernel_sp`
    * `trapframe.sp`用户栈指针
    * `trapframe.kernel_sp`内核栈指针
    * 内核栈指针需要额外保存是因为每个进程的内核栈空间不一样, 为了下次切换会内核态时能够找到


### exec

- 简易实现(从内存加载)
    1. `proc_pagetable`创建进程页表`uvmcreate`, **映射trampoline空间, 映射trapframe**
    2. 读取程序: 这里用从内存读模拟从ELF读, 真实情况ELF文件会解析出一些列信息包括, entry等
    3. `uvmalloc(table, oldsz, newsz)`为程序申请物理页, **并标记可读可写可执行可用户态访问**
    4. `loadseg()`数据拷贝, `walkaddr()`翻译到物理页然后拷贝(从内存/从ELF文件)
    5. 初始化`Proc`结构体信息
        - 分配用户栈 
        - 设置pageable, sz, trapframe等成员


### tips

- rust中函数指针的构建方法
    * `core::mem::transmute`

```rust
    let func_ptr = TRAMPOLINE + (userret as usize - trampoline as usize);
    let func: extern "C" fn(usize, usize) -> ! = unsafe { core::mem::transmute(func as usize) };
    func(trapframe, satp)
```


### 开发日志

- walk panic: `etext`
    * PTE的标志位没有做好, 导致页表根本没能分级, 两个原因
        1. **RUST所有权**, 取pte时把所有权从`entries[]`里拿走了, 之后都判断不到`PTE_V`
        2. 手抖, 创建PTE时应该标志位更新`PTE_V`, 但是写成了`PA2PTE!(new_page_pa | PTE_V);`, 应该是`PA2PTE!(new_page_pa) | PTE_V;`

- 开启虚存后地址的转换, 不再能直接函数调用了
    * 内核态可以直接函数调用
    * 用户态需要转换一下地址

- memory can not access
    * 内核页表映射 ✅
    * 问题出在`userret`第一行`csrw satp, a1`
        + 为什么先切换页表?? 也可以后切换
        + **rust内存模型理解问题导致satp赋值页表时出错**, 重构项目, 用`&mut`替代`\*mut`: `NonNull<T>`, 内部非指针, 创建时指针`NonNull<T>::new(*mut T)`
        ```
        pagetable: Option<*mut PageTable>
        println!("{:#x}", p.pagetable.unwrap() as usize); ✅
        println!("{:#x}", &p.pagetable.unwrap() as *const _ as usize); ❌
        println!("{:#x}", p.pagetable.as_ref().unwrap() as *const _ as usize); ❌
        ```
    * 页表切换后访存存在问题
- trapframe 无法访问
    * `112(a0)`
    * `proc_pagetable`中做的trapframe映射
    * 同样是页表切换后的寻址问题, 需要根据trampoline计算得到trapframe相对位置
- sret切换会用户态前`epc`问题
    1. 代码段手抖bug, 少了可执行`PTE_X`, 写成了`PTE_W | PTE_W`
    2. 但是还是会跳转到`0x3ffffff000`
    3. 似乎是因为`sp`设置错了: 用户栈sp = newsz-PGSIZE应为sp=newsz指向栈底(高地址)
        a. usertrapret -> userret(trapframe, satp)参数的传递发生错误??也不是因为页表切换了
        b. 但是trapframe确实是访问错了, 因为sp设置错了
- **无论是内核态还是用户态, 都隔了层页表, 不能把内核态虚拟地址当成是物理地址**
    * trapframe映射异常问题是因为我直接把内核态`&p.trapframe`当成物理地址了, 但其实这是内核态虚拟地址
    * 虽然这里内核态地址空间是直接映射(处理trampoline), 即`va = pa`
    * 直接的物理地址只有页帧分配器知道 ✅
    * 否则就要`walk`转换一下
    * 那么同样`trapframe`的问题，程序加载的时候, 拿到的src也是内核态的虚拟地址
- 用户栈设置错了:sp = newsz - PGSIZE, 而栈从高向低, 应为sp = newsz
- scause: 0xc Load page fault
    * **链接问题**
    * sepc = 0x804002ee
    * **链接时, 我们原本的链接到0x80400000。因为存在部分跳转地址的硬编码从而导致sepc = 0x804002ee**
    * 不想需要修改xv6的线性地址策略?? p.sz, 那就修改用户程序的链接策略, 直接链接到`0x0000`
- **启动页表后系统调用需要修改**
    * bug表现: usertrap处理系统调用后会触发kerneltrap
    * 用户写入的地址不再直接可用, 需要先翻译, 如write
- `sched`导致的`kerneltrap`
    * 因为我们的链接脚本虽然把base改成0了，但仍然step, 所以要把step该成0
    * 重写用户态链接机制, 不再需要链接脚本了
- `b *0x660`



