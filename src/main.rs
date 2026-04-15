#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(sosaltix2::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use sosaltix2::println;
use bootloader::{BootInfo, entry_point};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use sosaltix2::memory::active_level_4_table;
    use x86_64::VirtAddr;
    use sosaltix2::memory::translate_addr;

    println!("Welcome to Sosaltix2");
    sosaltix2::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let addresses = [
        0xb8000,
        0x201008,
        0x0100_0020_1a10,
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { translate_addr(virt, phys_mem_offset) };
        println!("{:?} -> {:?}", virt, phys);
    }


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