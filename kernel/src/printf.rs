use core::fmt::{self, Write};
use core::panic::PanicInfo;

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let m_uart = crate::uart::read();
        for c in s.chars() {
            m_uart.putc(c);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

//printf!宏
#[macro_export]
macro_rules! printf {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::printf::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::printf::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}


// blue
#[macro_export]
macro_rules! info {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::printf::print(format_args!(concat!("\x1b[34m", $fmt, "\x1b[0m") $(, $($arg)+)?));
    }
}

// yellow
#[macro_export]
macro_rules! warn {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::printf::print(format_args!(concat!("\x1b[93m", $fmt, "\x1b[0m") $(, $($arg)+)?));
    }
}

// red
#[macro_export]
macro_rules! error {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::printf::print(format_args!(concat!("\x1b[31m", $fmt, "\x1b[0m") $(, $($arg)+)?));
    }
}

// green
#[macro_export]
macro_rules! debug {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::printf::print(format_args!(concat!("\x1b[32m", $fmt, "\x1b[0m") $(, $($arg)+)?));
    }
}


// panic depending on error!()
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "Panicked at {}:{} {}\n",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        error!("Panicked: {}\n", info.message().unwrap());
    }
    loop {}
}


