use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{println, print};
use lazy_static::lazy_static;
use x86_64::instructions::port::Port;
use core::sync::atomic::{AtomicU64, Ordering};

pub const TIMER_VECTOR: u8 = 32;
pub const SPURIOUS_VECTOR: u8 = 255;

pub fn disable_pic() {
    unsafe {
        Port::<u8>::new(0x21).write(0xFF);
        Port::<u8>::new(0xA1).write(0xFF);
    }
}

#[inline]
unsafe fn rdmsr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    core::arch::asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") low,
        out("edx") high,
        options(nomem, nostack, preserves_flags),
    );
    ((high as u64) << 32) | (low as u64)
}

#[inline]
unsafe fn wrmsr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    core::arch::asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") low,
        in("edx") high,
        options(nostack, preserves_flags),
    );
}

pub fn init_apic() {

    disable_pic();

    // проверка поддержки x2apic
    let cpuid = raw_cpuid::CpuId::new();
    if !cpuid.get_feature_info().map(|f| f.has_x2apic()).unwrap_or(false) {
        panic!("x2APIC not supported");
    }

    unsafe {
        // включение x2APIC
        let mut apic_base = rdmsr(0x1B);
        apic_base |= 1 << 10;
        apic_base |= 1 << 11; 
        wrmsr(0x1B, apic_base);

        // маскировка хуйни
        wrmsr(0x832, 1 << 16);
        wrmsr(0x833, 1 << 16);
        wrmsr(0x834, 1 << 16);
        wrmsr(0x835, 1 << 16);
        wrmsr(0x836, 1 << 16); 
        wrmsr(0x837, 1 << 16);

        wrmsr(0x80F, (SPURIOUS_VECTOR as u64) | (1 << 8));

        wrmsr(0x3E2, 0x3);
        
        wrmsr(0x3E0, 0);
    }

    println!("x2APIC initialized (timer masked)");
}

pub fn start_timer(initial_count: u32) {
    unsafe {
        let timer_lvt = (TIMER_VECTOR as u64) & 0xFF;
        wrmsr(0x832, timer_lvt);
        
        wrmsr(0x3E0, initial_count as u64);
    }
    println!("APIC timer started with count: {}", initial_count);
}

pub fn send_eoi() {
    unsafe {
        wrmsr(0x80B, 0);
    }
}

pub static TIMER_COUNT: AtomicU64 = AtomicU64::new(0);

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        
        idt[TIMER_VECTOR].set_handler_fn(timer_interrupt_handler);
        idt[SPURIOUS_VECTOR].set_handler_fn(spurious_interrupt_handler);
        
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn spurious_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // просто возвращается
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    TIMER_COUNT.fetch_add(1, Ordering::Relaxed);
    print!("T");
    send_eoi();
}