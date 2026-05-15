#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use kernel::{println, allocator, memory};
use bootloader_api::{entry_point, BootInfo, BootloaderConfig, config::Mapping};
use bootloader_api::info::{Optional, MemoryRegions};
use x86_64::VirtAddr;
use kernel::memory::BootInfoFrameAllocator;
use x86_64::structures::paging::Page;

const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::FixedAddress(0xffff800000000000));
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {

    // инит VGA
    let framebuffer = boot_info.framebuffer.take();
    kernel::vga_buffer::init(framebuffer);
    println!("VGA initialized");

    // инит остальной залупы
    kernel::init();

    println!("APIC timer active. System idle loop.");

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

    // запуск задач (пока выключил смысла с нерабочими прерываниями в них нет)

    /*let mut executor = Executor::new();
    executor.spawn(Task::new(shell::run_shell()));
    executor.run();
    */

    kernel::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    kernel::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info)
}