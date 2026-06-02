use x86_64::instructions::port::Port;

pub fn reboot() -> ! {
    
    unsafe {
        let mut reset_port = Port::<u8>::new(0x64);
        reset_port.write(0xFE);
    }

    unsafe {
        let mut pci_reset = Port::<u8>::new(0xCF9);
        pci_reset.write(0x06);
    }

    loop {
        x86_64::instructions::hlt();
    }
}

pub fn poweroff() -> ! {
    unsafe {
        let mut qemu_power = Port::<u16>::new(0x604);
        qemu_power.write(0x2000);

        let mut bochs_power = Port::<u16>::new(0xB004);
        bochs_power.write(0x2000);
        
        let mut vbox_power = Port::<u16>::new(0x4004);
        vbox_power.write(0x3400);
    }   
    
    loop {
        x86_64::instructions::hlt();
    }
}