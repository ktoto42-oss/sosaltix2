use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::VirtAddr;
use crate::{println, print, gdt};
use lazy_static::lazy_static;
use x2apic::lapic::{LocalApicBuilder, xapic_base};
use x86_64::instructions::port::Port;

// векторы прерывание > 32 так приказал аллах
pub const APIC_TIMER_VECTOR: u8 = 32;
pub const APIC_ERROR_VECTOR: u8 = 33;
pub const KEYBOARD_VECTOR: u8 = 34;
pub const APIC_SPURIOUS_VECTOR: u8 = 255;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        // регистрация обработчиков
        idt[APIC_TIMER_VECTOR].set_handler_fn(apic_timer_handler);
        idt[APIC_ERROR_VECTOR].set_handler_fn(apic_error_handler);
        idt[KEYBOARD_VECTOR].set_handler_fn(keyboard_interrupt_handler);
        idt[APIC_SPURIOUS_VECTOR].set_handler_fn(apic_spurious_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.general_protection_fault.set_handler_fn(gpf_handler);
        idt
    };
}

static mut LAPIC: Option<x2apic::lapic::LocalApic> = None;

pub fn init_idt() {
    IDT.load();
}

pub fn disable_legacy_pic() {
    unsafe {
        let mut master_data_port = Port::<u8>::new(0x21);
        master_data_port.write(0xFF);

        let mut slave_data_port = Port::<u8>::new(0xA1);
        slave_data_port.write(0xFF);
    }
}

/// инит lapic
pub fn init_apic(phys_mem_offset: VirtAddr) {
    // получение физ адреса
    let apic_phys_base = unsafe { xapic_base() };
    //println!("APIC physical base: {:#x}", apic_phys_base);

    // преобразование в виртуальный
    let apic_virt_base = VirtAddr::new(phys_mem_offset.as_u64() + apic_phys_base);
    //println!("APIC virtual base: {:#x}", apic_virt_base.as_u64());

    // создание lapic
    let mut lapic = LocalApicBuilder::new()
        .timer_vector(APIC_TIMER_VECTOR as usize)
        .error_vector(APIC_ERROR_VECTOR as usize)
        .spurious_vector(APIC_SPURIOUS_VECTOR as usize)
        .set_xapic_base(apic_virt_base.as_u64())
        .build()
        .unwrap_or_else(|e| panic!("Failed to build Local APIC: {}", e));

    // включение apic
    unsafe {
        lapic.enable();
        lapic.set_timer_divide(x2apic::lapic::TimerDivide::Div16);
        lapic.set_timer_initial(0x100000); // подберите под ваш CPU
        lapic.set_timer_mode(x2apic::lapic::TimerMode::Periodic);
        LAPIC = Some(lapic);
    }
    //println!("Local APIC enabled.");
}

pub fn init_ioapic(phys_mem_offset: VirtAddr) {
    // адекса iopic
    let ioapic_phys_base = 0xFEC00000;
    let ioapic_virt_base = phys_mem_offset.as_u64() + ioapic_phys_base;

    // запись в регистры iopic
    unsafe fn write_ioapic(base: u64, reg: u32, data: u32) {
        let index_ptr = base as *mut u32;
        let data_ptr = (base + 0x10) as *mut u32;
        index_ptr.write_volatile(reg);
        data_ptr.write_volatile(data);
    }

    unsafe {
        let irq = 1; 
        let vector = KEYBOARD_VECTOR as u32;
        let low_index = 0x10 + irq * 2;
        let high_index = 0x10 + irq * 2 + 1;

        write_ioapic(ioapic_virt_base, high_index, 0);
        write_ioapic(ioapic_virt_base, low_index, vector);
    }
    //println!("I/O APIC initialized (Keyboard IRQ1 -> Vector {})", KEYBOARD_VECTOR);
}

// обработчики прерываний

use x86_64::structures::idt::PageFaultErrorCode;

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    panic!(
        "EXCEPTION: PAGE FAULT\nAccessed Address: {:?}\nError Code: {:?}\n{:#?}",
        Cr2::read(),
        error_code,
        stack_frame
    );
}

extern "x86-interrupt" fn gpf_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT\nError Code: {}\n{:#?}", error_code, stack_frame);
}

extern "x86-interrupt" fn apic_timer_handler(stack_frame: InterruptStackFrame) {
    //println!("APIC Timer interrupt")
    unsafe {
        if let Some(ref mut lapic) = LAPIC {
            lapic.end_of_interrupt();
        }
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    
    // передача сканкода в шелл
    crate::shell::add_scancode(scancode);

    unsafe {
        if let Some(ref mut lapic) = LAPIC {
            lapic.end_of_interrupt();
        }
    }
}


extern "x86-interrupt" fn apic_error_handler(stack_frame: InterruptStackFrame) {
    println!("APIC Error interrupt: {:#?}", stack_frame);
}

extern "x86-interrupt" fn apic_spurious_handler(stack_frame: InterruptStackFrame) {
    println!("APIC Spurious interrupt");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}