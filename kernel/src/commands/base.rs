use alloc::string::String;
use crate::println;

pub fn help() {
    println!("Available commands: echo, help, clear, fetch, reboot, poweroff");
}

pub fn echo(args: String) {
    println!("{args}");
}