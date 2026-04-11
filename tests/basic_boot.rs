#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(sosaltix2::test_runner)]

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    sosaltix2::test_panic_handler(info)
}


use core::panic::PanicInfo;

#[unsafe(no_mangle)] 
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}