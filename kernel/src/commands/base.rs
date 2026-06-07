use alloc::string::String;
use crate::println;

pub fn help() {
    println!("Available commands: echo, help, clear, fetch, reboot, poweroff");
}

pub fn echo(args: String) {
    println!("{args}");
}

use crate::virtio::DISK;

pub fn print_disk_status() {
    let mut disk_lock = DISK.lock();
    
    if let Some(ref mut disk) = *disk_lock {
        println!("--- Global Disk Status ---");
        println!("Status: ONLINE");
        println!("Capacity: {} sectors ({} MB)", disk.capacity(), (disk.capacity() * 512) / 1024 / 1024);
    } else {
        println!("Disk Status: OFFLINE (Not initialized yet)");
    }
}

pub fn read_sector_cmd(sector_id: u64) {
    let mut disk_lock = DISK.lock();
    
    if let Some(ref mut disk) = *disk_lock {
        let mut buf = [0u8; 512];
        
        match disk.read_blocks(sector_id as usize, &mut buf) {
            Ok(_) => {
                println!("Sector {} read successfully! First 16 bytes:", sector_id);
                println!("{:x?}", &buf[0..16]);
            }
            Err(e) => {
                println!("Error reading sector {}: {:?}", sector_id, e);
            }
        }
    } else {
        println!("Error: Disk is offline.");
    }
}