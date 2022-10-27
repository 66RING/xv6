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

- sstatus
	* TODO:
- epc
	* 进入异常状态时刻的pc指针
- mret/sret
	* 从异常状态返回原来状态(mode), 并根据对应epc设置pc指针
- ret
	* 普通函数返回会返回到ra寄存器记录的地址, 返回值保存到a0寄存器

### trap与上下文切换机制

学trap感觉非常绕, 浅记一下。个人认为[rcore](https://rcore-os.cn/rCore-Tutorial-Book-v3/)的实现会比较好理解一点, 不过xv6的也值得学习一下, 所有这里把两个的实现都整理了一下。(**未开虚拟内存版**)

拉片分析: 进程第一次启动 -> 进程第一次返回用户态 -> 进程陷入内核态 -> 进程再返回用户态

> rcore对应代码[ch3](https://github.com/rcore-os/rCore-Tutorial-v3/tree/ch3), 未开启分页版, 开启分页后大同小异

- 名词解释
	* epc
		+ 进入异常状态时刻的pc指针
	+ mret/sret
		+ 从异常状态返回原来状态(mode), 并根据对应epc设置pc指针
	* trapframe/TrapContext
		+ 下陷时刻的寄存器现场 + **一些切换所需的记录**
		+ 比如下陷时刻`sp`应该是用户栈, 所以`trapframe.sp`保存的就是用户栈

- 进程第一次启动(初始化)
	* 创建进程的上下文context
	* 然后调用`swtch`函数完成上下文切换
	* **swtch**是个函数调用, 函数返回时会根据ra寄存器返回对应地址
	* 为了实现一个完整的闭环, 第一次启动要和第n次下陷的操作是一模一样的, 所以我们要 **"伪造"TrapFrame** 和 **伪造Context(上下文)**
		+ 伪造trapframe:
			+ trapframe可以理解成下陷时刻用户态的寄存器现场, 我们根据这个现场进入到用户态, 所以我们伪造的trapframe应该
				+ `trapframe.sp`指向 **用户栈**
				+ `trapframe.epc`指向 **用户程序第一行**
			+ rcore的trapframe是保存在进程内核栈上, 而xv6的trapframe是保存在堆上通过`proc->trapframe`访问
		+ 伪造context:
			+ swtch就是对context的切换, 但是并没有做状态空间切换的操作
			+ 事实上因为有内核态和用户态, context上下文是有着内核态上下文和用户态上下文两部分的, 这里我们的**context结构记录的是内核态上下文**, 会一一对应一个trapframe, 哪里保存了**用户态上下文**
			+ 所以我们伪造的context应该
				+ `context.ra`指向用户态返回函数: xv6中是`usertrapret`, rcore中是`__restore`
				+ `context.sp`指向进程的**内核栈**
	* 总结
		+ `trapframe.sp`指向用户栈
		+ `trapframe.epc`指向用户主程序
		+ `context.sp`指向内核栈
		+ context可以理解为进程的内核态上下文
		+ trapframe可以理解为进程的内核态上下文
- 进程第一次返回用户态
	* `swtch`结束后, 函数返回到ra寄存器指示的地址, xv6中是`usertrapret`, rcore中是`__restore`
	* 此时sp指向**内核栈**, 我们要找到trapframe来还原用户态现场
		+ rcore的实现:
			+ 设置`stvec`为用户态下陷的入口地址`__alltraps`, `trap::init()`
			+ 因为我们伪造trapframe是在内核栈栈顶, 可以使用使用sp + offset访问
			+ 切换需要用到一个 **"辅助寄存器" sscratch**永久保存
			+ 第一步先让`sscratch`指向`trapframe.sp`即用户栈, 而当前sp是内核栈, 最后一交换栈空间就进入了用户栈, 而sscratch将保存内核栈
			+ 然后恢复各个寄存器的内容, 弹出内核栈trapframe: `addi sp, sp, 34*8`
			+ **设置sepc**为`trapframe.epc`即用户主程序
			+ 最后`sp`和`sscratch`交换完成栈空间切换: `csrrw sp, sscratch, sp`, `sret`后pc就成了sepc中的内容
		+ xv6的实现:
			+ 设置`stvec`为用户态下陷的入口地址`uservec`, `w_stvec(uservec);`
			+ 设置`sepc`为用户态主程序
			+ 设置trapframe用于还原用户态寄存器现场, 主要是设置`trapframe.kernel_sp`, 然后设置`trapframe.kernel_trap`为trap处理函数 **为下次下陷做准备**
			+ 将trapframe通过**a0寄存器**传递给`userret`完成用户态切换
			+ 与rcore类似, 需要使用`sscratch`寄存器做一个中介永久保留
			+ `userret`第一步让`sscratch`保存`trapframe.a0`即用户态a0的值, 然后此时的a0是trapframe
			+ 然后恢复各个寄存器
			+ 最后`a0`和`sscratch`交换把剩下的一个a0寄存器给恢复了, 之后`a0`为用户态a0值, `sscratch`为trapframe
	* 总结
		+ rcore的切换是围绕sp(sp是内核栈顶trapframe的引用), sscratch完成的
		+ xv6的切换是围绕a0(a0是堆中的trapframe的引用), sscratch完成的
		+ 此时:
			+ rcore的sscratch指向内核栈栈顶
			+ xv6的sscratch指向堆中的trapframe
- 进程陷入内核态
	* 进入用户态前都设置了`stvec`, rcore为`__alltraps`, xv6为`uservec`
	* 上一次返回用户态结束后
		+ rcore的sscratch指向内核栈栈顶
		+ xv6的sscratch指向堆中的trapframe
	* rcore的实现
		+ 将下陷时刻的寄存器现场保存到内核栈顶的trapframe
		+ 所以先从`sscratch`中恢复内核栈并开辟栈空间`csrrw sp, sscratch, sp`, `addi sp, sp, -34*8`
		+ 然后保存各个寄存器现场
		+ 最后将内核栈顶的trapframe传给`trap_handler`处理下陷
	* xv6的实现
		+ 先`csrrw a0, sscratch, a0`获取到trapframe, 用户态a0保存到sscratch
		+ 然后使用`a0`访问trapframe保存一系列寄存器现场
		+ 因为我们在上一阶段中将trap处理函数保存到了`trapframe.kernel_trap`
		+ 所以将`sscratch`记录的a0页保存到trapframe中, 最后读出`trapframe.kernel_trap`到t0, `jr t0`跳转到trap处理函数
	* 总结
		+ 找到trapframe(在sscrach寄存器中), 然后存入下陷时刻的寄存器现场到其中
		+ rcore和xv6最后都在trap处理函数中拿到了`trapframe`用于分析处理(读取参数等)
			+ 只不过rcore通过sp定位trapframe, 然后用a0做参数传递
			+ xv6则是通过结构体定位trapframe, 因为trapframe是在堆上
			+ 最后都能根据trapframe保存了下陷时刻的用户态现场得到下陷信息
- 进程再返回用户态
	* trap处理函数执行完成后: rcore的`trap_handler`, xv6的`usertrap`
	* xv6简单再次调用`usertrapret`就能返回了
	* **rcore**比较巧妙
		+ rcore汇编中`__alltrap`之后紧接着就是`__restore`
		+ 所以在`__alltrap`调用完`trap_handler`后就能进入`__restore`返回用户态
		+ 而`__restore`对初始状态的断言是**a0指向内核栈顶的trapframe**
			+ 所以在rcore中`trap_handler`结束要返回trapframe(rcore中叫TrapContext), 那么根据函数调用协议返回值就会自动保存到a0
			+ 之后下一条指令就是`__restore`形成完美闭环
	* 总结
		+ 要形成闭环就要满足trap return初始状态的断言
			+ rcore中断言(`__restore`)a0指向内核栈中的trapframe, 所以`trap_handler`要返回trapframe(TrapContext), 函数调用协议会自动保存到a0中
			+ xv6中断言(`usertrapret`)断言了a0指向trapframe, `trapframe.kernel_trap`指向trap处理函数
				+ 不过xv6先用`usertrapret`过渡把初始状态处理好后再调用`userret`返回
- 总结
	* context相当于进程的**内核态**上下文现场, trapframe相当于进程的 **用户态**上下文现场



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
- 想清楚什么时候需要lock, 什么时候不用

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
