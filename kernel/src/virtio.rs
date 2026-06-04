use virtio_drivers::{BufferDirection, Hal, PhysAddr, PAGE_SIZE};
use virtio_drivers::transport::pci::{
    PciTransport,
    bus::{ConfigurationAccess, PciRoot, DeviceFunction},
};
use virtio_drivers::device::gpu::VirtIOGpu;
use core::ptr::NonNull;
use alloc::alloc::{alloc_zeroed, dealloc, Layout};
use x86_64::VirtAddr;

const PHYS_MEM_OFFSET: u64 = 0xffff_8000_0000_0000;

pub struct SosaltixHal;

unsafe impl Hal for SosaltixHal {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let layout = Layout::from_size_align(pages * PAGE_SIZE, PAGE_SIZE).unwrap();
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() {
            panic!("VirtIO HAL: Failed to allocate {} pages for DMA", pages);
        }
        
        let vaddr = VirtAddr::new(ptr as u64);
        let paddr = unsafe { crate::memory::translate_addr(vaddr, VirtAddr::new(PHYS_MEM_OFFSET)) }
            .expect("VirtIO HAL: Failed to translate DMA virtual address")
            .as_u64();
        
        (paddr as PhysAddr, NonNull::new(ptr).unwrap())
    }

    unsafe fn dma_dealloc(_paddr: PhysAddr, vaddr: NonNull<u8>, pages: usize) -> i32 {
        let layout = Layout::from_size_align(pages * PAGE_SIZE, PAGE_SIZE).unwrap();
        dealloc(vaddr.as_ptr(), layout);
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new((paddr as u64 + PHYS_MEM_OFFSET) as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let vaddr = VirtAddr::new(buffer.as_ptr() as *mut u8 as u64);
        let paddr = unsafe { crate::memory::translate_addr(vaddr, VirtAddr::new(PHYS_MEM_OFFSET)) }
            .expect("VirtIO HAL: Failed to translate shared buffer virtual address")
            .as_u64();
        paddr as PhysAddr
    }

    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {}
}

struct PortIoCam;

impl ConfigurationAccess for PortIoCam {
    fn read_word(&self, bdf: DeviceFunction, offset: u8) -> u32 {
        crate::pci::pci_config_read_u32(bdf.bus, bdf.device, bdf.function, offset)
    }

    fn write_word(&mut self, bdf: DeviceFunction, offset: u8, value: u32) {
        crate::pci::pci_config_write_u32(bdf.bus, bdf.device, bdf.function, offset, value)
    }

    unsafe fn unsafe_clone(&self) -> Self {
        PortIoCam
    }
}

pub fn test_graphics(
    pci_device: &crate::pci::PciDevice,
    mapper: &mut x86_64::structures::paging::OffsetPageTable,
    frame_allocator: &mut impl x86_64::structures::paging::FrameAllocator<x86_64::structures::paging::Size4KiB>,
) {
    crate::println!("Initializing VirtIO GPU driver via custom Port IO CAM...");

    for i in 0..6 {
        let offset = 0x10 + (i * 4);
        let bar = crate::pci::pci_config_read_u32(pci_device.bus, pci_device.slot, pci_device.func, offset);
        if bar == 0 { continue; }
        
        if bar & 1 == 0 {
            let is_64bit = (bar & 0x6) == 0x4;
            let mut phys_addr = (bar & 0xFFFF_FFF0) as u64;
            
            if is_64bit && i < 5 {
                let next_bar = crate::pci::pci_config_read_u32(pci_device.bus, pci_device.slot, pci_device.func, offset + 4);
                phys_addr |= (next_bar as u64) << 32;
            }
            
            if phys_addr != 0 {
                crate::println!("Pre-mapping PCI BAR{} at physical address {:#x}", i, phys_addr);
                map_mmio_range(phys_addr, 0x0100_0000, mapper, frame_allocator);
            }
        }
    }

    let cam = PortIoCam;
    let mut pci_root = PciRoot::new(cam);

    let device_function = DeviceFunction {
        bus: pci_device.bus,
        device: pci_device.slot,
        function: pci_device.func,
    };

    let transport = PciTransport::new::<SosaltixHal, _>(&mut pci_root, device_function)
        .expect("Failed to create VirtIO PCI transport");

    let mut gpu = VirtIOGpu::<SosaltixHal, _>::new(transport)
        .expect("Failed to initialize VirtIO GPU device");

    crate::println!("VirtIO GPU initialized successfully! Setting up framebuffer...");

    let (width, height) = gpu.resolution().unwrap();
    crate::println!("VirtIO GPU Screen Resolution: {}x{}", width, height);

    {
        let fb = gpu.setup_framebuffer()
            .expect("Failed to setup VirtIO GPU framebuffer");

        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                if idx + 3 < fb.len() {
                    fb[idx]     = x as u8;  
                    fb[idx + 1] = y as u8;   
                    fb[idx + 2] = 240;       
                    fb[idx + 3] = 255;       
                }
            }
        }
    }

    gpu.flush().expect("Failed to flush GPU framebuffer to screen");

    crate::println!("Graphics test finished rendering.");
}

use x86_64::structures::paging::{Page, PhysFrame, Mapper, Size4KiB, FrameAllocator, PageTableFlags, Translate};

pub fn map_mmio_range(
    phys_start: u64,
    size: usize,
    mapper: &mut x86_64::structures::paging::OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let start_phys = phys_start & !0xFFF;
    let page_offset = phys_start & 0xFFF;
    let total_size = size + page_offset as usize;
    
    let start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(start_phys + PHYS_MEM_OFFSET));
    let end_page = Page::<Size4KiB>::containing_address(VirtAddr::new(start_phys + PHYS_MEM_OFFSET + total_size as u64 - 1));

    let flags = PageTableFlags::PRESENT 
        | PageTableFlags::WRITABLE 
        | PageTableFlags::NO_CACHE 
        | PageTableFlags::WRITE_THROUGH;

    for page in Page::range_inclusive(start_page, end_page) {
        let offset = page.start_address().as_u64() - start_page.start_address().as_u64();
        let frame = PhysFrame::containing_address(x86_64::PhysAddr::new(start_phys + offset));
        
        if mapper.translate_addr(page.start_address()).is_none() {
            unsafe {
                mapper.map_to(page, frame, flags, frame_allocator)
                    .expect("Failed to map MMIO page for VirtIO")
                    .flush();
            }
        }
    }
}