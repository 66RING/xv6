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


