# xv6-rs

学习xv6, 用rust重新实现xv6


## ch1 打印机

所谓打印机就是只会`print()`的机器

- 硬件初始化, start.rs
	1. 设置工作模式(特权级): M, S, U
		- 机器启动时模式为M
		- 设置为S, 在mret后即进入S模式, 即常说的内核态
		- 设置mret后pc指针位置, 即我们的main, 内核主程序
	2. 配置S模式的物理内存保护(`w_pmpaddr0`), 以便我们的内核能访问所以内存
	3. 其他: 中断, 计时器, 分页等 TODO review
	* TODO: 理清硬件细节
- 打印, uart.rs, printf.rs
	* uart.rs负责与硬件打交道, 简单的说就是向特定内存/寄存器读写值
		+ `volatile_register`
			+ TODO: https://docs.rs/volatile-register/latest/volatile_register/
			+ TODO: xv6版uart处理
	* printf.rs使用uart.rs铺好的路使用rust中的`print!()`功能
- 汇编指令封装, riscv.rs
	* 内敛汇编对寄存器访问, 与主程序交互
- 链接与入口
	* 我们可以为rust编译器指定链接脚本`kernel.ld`
		+ 编辑`.cargo/config`文件, 指定目标平台和使用的链接脚本
	* 链接脚本中开头使用`ENTRY(_entry);`和`OUTPUT_ARCH("riscv");`标明入口符号是`_entry`, 目标平台是`riscv`
	* 汇编入口代码, entry.S
		1. 第一步就是初始化栈空间(sp寄存器)
		2. 然后跳转到`start`函数进行初始化


## ch2 多道程序/系统调用

- TODO
	* xv6 trap参数协议

TODO: **重点是上下文切换的逻辑**

- Q: xv6
	* usertrapret
	* usertrap
	* usertrap
	* kerneltrap
	* kernelvec
	* forkret -> usertrapret -> userret

1. 先`swtch`切换proc
	- 切换寄存器上下文
	- 切换**trapframe**, 通过切换内核栈指针隐式进行, 
2. TODO: `swtch`结束, 并没有完全进入用户态, 还有一部分内核态残余
	- pc到了`userret`
	- **sp仍指向进程的内核栈**, 通过trapframe进入用户态

- **trap**
	* trapinit();      // trap vectors
	* trapinithart();  // install kernel trap vector
	* kernelvec.S
		+ TODO: 分析
	* struct trapframe // data page for trampoline.S
- 系统调用syscall
	* TODO:
	* exit
		+ 接收用户程序退出, 运行下一个
	* print
- 分配内核栈
- 中断处理
	* 时钟中断
- 临时内核栈和用户栈
	* 因为还没实现虚拟内存和内存映射

- TODO
	* 检查swtch, userret和rcore的兼容性

- 静态分配内核栈和用户栈, 待虚拟内存和内存映射实现


### 机器状态

TODO:

- sstatus
- epc
	* sepc
	* mepc
- ret
	* sret
	* mret

### trap与上下文切换

TODO:

> 流程allocproc() -> exec()

1. 先在内核态`swtch`切换进程寄存器线程`Context`
	- `allocproc()`程序初始化时
		* Context.sp指向**进程内核栈**
		* Context.ra指向`usertrapret`(`forkret`), `swtch`函数返回后pc的地址
2. `exec()`制造trapframe
	- trapframe.sp指向用户栈
	- trapframe.epc指向用户主程序
3. trapframe用于用户态和内核态的切换

TODO: usertrapret具体流程: a0, trapframe和sscratch的操作

- a0

### 程序加载

TODO:


### 用户程序

- 每个程序单独链接脚本
	* TODO对比ch2找区别, 且为什么: 没有区别, ch2的makefile没有写好
	* 并且剔除了metadata


### xv6化

- 没有所谓`proc_manager`, 反而抽象成`cpu`和`proc_pool`
	* `proc_pool`的所有的
	* `cpu`管理当前的

### Debug

1. 还不不能照抄xv6的trap实现
	* userret
	* TODO: kernelvec
2. boot栈空间大小问题
	- 我们的TrapFrame, Context等都不是指针, 启动栈会不太够
3. TODO: 没有补完usertrap等导致scause 5
	- 下陷后没有切换机器状态, 没有切换epc
4. TODO: 没有将程序拷贝到内存导致scause 2, 没有程序/汗
	- build.rs细节
	- build.rs脚本问题, 用户程序被编译成了`xxx.bin`, 而`build.rs`加载的目标没有`.bin`后缀
5. trapframe.a7等没能保存成功
	- `userret(trapframe)`, trapframe传递有问题
6. **rust所有权: 默认move**
	- TODO: 怎么抽象
7. **rust怎么返回struct又保留原来所有权**
	- 像xv6那样的myproc已经不行了
	- 在swtch这种不会返回rust代码的程序中应小心临时变量
	- **使用RefCell**
8. 因为我们os常有`->!`, 所以所有权常常被移动了, 这样一来`p.trapframe`就不是原来的了
9. 处理系统调用时epc记得加4, 即跳过`ecall`
