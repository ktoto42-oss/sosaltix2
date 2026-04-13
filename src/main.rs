#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(sosaltix2::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use sosaltix2::println;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Welcome to Sosaltix2{}", "!");

    sosaltix2::init();
    
    #[cfg(test)]
    test_main();

    println!("It did not crash!");

    sosaltix2::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    sosaltix2::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    sosaltix2::test_panic_handler(info)
}