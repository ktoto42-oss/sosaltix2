use x86_64::instructions::port::Port;

pub const VIRTIO_VENDOR_ID: u16 = 0x1AF4;
pub const VIRTIO_GPU_DEVICE_ID: u16 = 0x1050;

#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
    pub vendor_id: u16,
    pub device_id: u16,
}

fn pci_config_read_u32(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address = ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC)
        | 0x8000_0000;

    let mut config_address = Port::<u32>::new(0xCF8);
    let mut config_data = Port::<u32>::new(0xCFC);

    unsafe {
        config_address.write(address);
        config_data.read()
    }
}

fn get_vendor_id(bus: u8, slot: u8, func: u8) -> u16 {
    (pci_config_read_u32(bus, slot, func, 0x00) & 0xFFFF) as u16
}

fn get_device_id(bus: u8, slot: u8, func: u8) -> u16 {
    ((pci_config_read_u32(bus, slot, func, 0x00) >> 16) & 0xFFFF) as u16
}

fn get_class_subclass(bus: u8, slot: u8, func: u8) -> (u8, u8) {
    let reg = pci_config_read_u32(bus, slot, func, 0x08);
    (((reg >> 24) & 0xFF) as u8, ((reg >> 16) & 0xFF) as u8)
}

fn get_header_type(bus: u8, slot: u8, func: u8) -> u8 {
    ((pci_config_read_u32(bus, slot, func, 0x0C) >> 16) & 0xFF) as u8
}

pub fn find_virtio_gpu() -> Option<PciDevice> {
    for bus in 0..=255 {
        for slot in 0..32 {
            let vendor_id = get_vendor_id(bus, slot, 0);
            if vendor_id == 0xFFFF {
                continue;
            }

            let header_type = get_header_type(bus, slot, 0);
            let num_functions = if (header_type & 0x80) != 0 { 8 } else { 1 };

            for func in 0..num_functions {
                let v_id = get_vendor_id(bus, slot, func);
                if v_id == VIRTIO_VENDOR_ID {
                    let d_id = get_device_id(bus, slot, func);
                    let (class, _) = get_class_subclass(bus, slot, func);

                    if d_id == VIRTIO_GPU_DEVICE_ID || (d_id >= 0x1000 && d_id <= 0x103F && class == 0x03) {
                        return Some(PciDevice {
                            bus,
                            slot,
                            func,
                            vendor_id: v_id,
                            device_id: d_id,
                        });
                    }
                }
            }
        }
    }
    None
}

pub fn scan_bus() {
    crate::println!("Scanning PCI bus...");
    for bus in 0..=255 {
        for slot in 0..32 {
            let vendor_id = get_vendor_id(bus, slot, 0);
            if vendor_id == 0xFFFF { continue; }
            
            let header_type = get_header_type(bus, slot, 0);
            let num_functions = if (header_type & 0x80) != 0 { 8 } else { 1 };
            
            for func in 0..num_functions {
                let v_id = get_vendor_id(bus, slot, func);
                if v_id == 0xFFFF { continue; }
                let d_id = get_device_id(bus, slot, func);
                let (class, subclass) = get_class_subclass(bus, slot, func);
                crate::println!(
                    "PCI [{:02x}:{:02x}.{}]: Vendor={:#06x} Device={:#06x} Class={:02x}",
                    bus, slot, func, v_id, d_id, class
                );
            }
        }
    }
}