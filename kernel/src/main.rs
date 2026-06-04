#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use kernel::{println, allocator, memory, shell};
use bootloader_api::{entry_point, BootInfo, BootloaderConfig, config::Mapping};
use bootloader_api::info::{Optional, MemoryRegions};
use x86_64::VirtAddr;
use kernel::memory::BootInfoFrameAllocator;
use x86_64::instructions::interrupts;
use kernel::task::executor::Executor;
use kernel::task::Task;


const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::FixedAddress(0xffff800000000000));
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let framebuffer = boot_info.framebuffer.take();
    let phys_mem_offset = match boot_info.physical_memory_offset {
        Optional::Some(offset) => VirtAddr::new(offset),
        Optional::None => panic!("phys_mem_offset naeb"),
    };

    kernel::init(phys_mem_offset);

    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let memory_map: &'static MemoryRegions = &boot_info.memory_regions;
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed");
    kernel::terminal::init(framebuffer);
    
    interrupts::enable();

    println!("Welcome to Sosaltix2");

    if let Some(gpu) = kernel::pci::find_virtio_gpu() {
        println!(
            "Found VirtIO GPU at [{:02x}:{:02x}.{}] (Device ID: {:#06x})",
            gpu.bus, gpu.slot, gpu.func, gpu.device_id
        );

        kernel::virtio::test_graphics(&gpu, &mut mapper, &mut frame_allocator);
        
    } else {
        println!("VirtIO GPU not found!");
    }

    let mut executor = Executor::new();
    executor.spawn(Task::new(shell::run_shell()));
    executor.run();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    kernel::hlt_loop();
}