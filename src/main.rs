#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(sosaltix2::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use sosaltix2::println;
use bootloader::{BootInfo, entry_point};
extern crate alloc;
use sosaltix2::task::Task;
use sosaltix2::task::executor::Executor;
use sosaltix2::shell;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use sosaltix2::memory;
    use sosaltix2::allocator;
    use x86_64::{structures::paging::Page, VirtAddr};
    use sosaltix2::memory::BootInfoFrameAllocator;

    println!("Welcome to Sosaltix2");

    sosaltix2::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));

    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    // запуск задач
    let mut executor = Executor::new();
    executor.spawn(Task::new(shell::run_shell()));
    executor.run();

    #[cfg(test)]
    test_main();

    sosaltix2::hlt_loop();
}

// тэстики

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

