pub unsafe fn get_xapic_base() -> u64 {
    let low: u32;
    let high: u32;
    core::arch::asm!(
        "rdmsr",
        in("ecx") 0x1B,
        out("eax") low,
        out("edx") high,
        options(nomem, nostack, preserves_flags)
    );
    let base = ((high as u64) << 32) | (low as u64);
    base & 0xFFFFF000
}

pub struct LocalApic {
    base_addr: u64,
}

impl LocalApic {
    pub unsafe fn new(virt_base: u64) -> Self {
        LocalApic { base_addr: virt_base }
    }

    unsafe fn write(&self, offset: u32, val: u32) {
        let ptr = (self.base_addr + offset as u64) as *mut u32;
        ptr.write_volatile(val);
    }

    unsafe fn read(&self, offset: u32) -> u32 {
        let ptr = (self.base_addr + offset as u64) as *const u32;
        ptr.read_volatile()
    }

    pub unsafe fn enable(&self, spurious_vector: u8) {
        let svr = (1 << 8) | (spurious_vector as u32);
        self.write(0x0F0, svr);
    }

    pub unsafe fn setup_timer(&self, timer_vector: u8, initial_count: u32, error_vector: u8) {
        self.write(0x370, error_vector as u32);

        self.write(0x3E0, 0x03);

        let timer_lvt = (1 << 17) | (timer_vector as u32);
        self.write(0x320, timer_lvt);

        self.write(0x380, initial_count);
    }

    pub unsafe fn eoi(&self) {
        self.write(0x0B0, 0);
    }
}