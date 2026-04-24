#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(sosaltix2::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use sosaltix2::println;

use bootloader_api::{entry_point, BootInfo};
use bootloader_api::info::MemoryRegions;
use bootloader_api::info::Optional; 

extern crate alloc;
use sosaltix2::task::Task;
use sosaltix2::task::executor::Executor;
use sosaltix2::shell;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    use sosaltix2::memory;
    use sosaltix2::allocator;
    use x86_64::{structures::paging::Page, VirtAddr};
    use sosaltix2::memory::BootInfoFrameAllocator;

    println!("Welcome to Sosaltix2 (UEFI navernoe)");

    sosaltix2::init();

    let phys_mem_offset = match boot_info.physical_memory_offset {
        Optional::Some(offset) => VirtAddr::new(offset),
        Optional::None => panic!("phys_mem_offset naeb"),
    };

    let mut mapper = unsafe { memory::init(phys_mem_offset) };

    let memory_map: &'static MemoryRegions = &boot_info.memory_regions;

    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(memory_map)
    };

    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));

    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("init_heap naeb");

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


