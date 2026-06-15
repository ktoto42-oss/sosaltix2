use alloc::string::String;
use alloc::str;
use crate::println;
use crate::virtio::DISK;
use crate::vfs::{
    vfs_read_file, vfs_write_file, vfs_append_file, 
    vfs_create_dir, vfs_delete_file, vfs_list_dir, vfs_change_dir
};

pub fn help() {
    println!("Available commands: \n echo \n clear \n help \n fetch \n disk-status \n read-sector \n poweroff \n reboot \n ls \n cat \n touch \n mkdir \n rm \n cd");
}

pub fn echo(args: String) {
    if args.contains(">>") {
        let parts: alloc::vec::Vec<&str> = args.split(">>").collect();
        if parts.len() == 2 {
            let mut content = parts[0].trim();
            let filename = parts[1].trim();

            if content.starts_with('"') && content.ends_with('"') && content.len() >= 2 {
                content = &content[1..content.len() - 1];
            }

            match vfs_append_file(filename, content.as_bytes()) {
                Ok(_) => println!("Appended to '{}'", filename),
                Err(e) => println!("Error appending to file: {}", e),
            }
            return;
        }
    } 
    else if args.contains(">") {
        let parts: alloc::vec::Vec<&str> = args.split('>').collect();
        if parts.len() == 2 {
            let mut content = parts[0].trim();
            let filename = parts[1].trim();

            if content.starts_with('"') && content.ends_with('"') && content.len() >= 2 {
                content = &content[1..content.len() - 1];
            }

            match vfs_write_file(filename, content.as_bytes()) {
                Ok(_) => println!("Written to '{}'", filename),
                Err(e) => println!("Error writing to file: {}", e),
            }
            return;
        }
    }

    println!("{args}");
}

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

pub fn read_sector_cmd(sector_string: String) {
    let sector_id: u64 = match sector_string.trim().parse() {
        Ok(num) => num,
        Err(_) => 0, 
    };
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

pub fn ls_cmd(args: String) {
    let trimmed = args.trim();
    let target_dir = if trimmed.is_empty() { None } else { Some(trimmed) };
    
    if let Err(e) = vfs_list_dir(target_dir) {
        println!("Error: {}", e);
    }
}

pub fn cat_cmd(filename: String) {
    let trimmed_name = filename.trim(); 
    if trimmed_name.is_empty() {
        println!("Usage: cat <file_path>");
        return;
    }
    
    if let Some(data) = vfs_read_file(trimmed_name) {
        match str::from_utf8(&data) {
            Ok(text) => println!("{}", text),
            Err(_) => println!("Error: File contains non-UTF8 data. Total bytes: {}", data.len()),
        }
    } else {
        println!("Error: File '{}' not found or failed to read.", trimmed_name);
    }
}

pub fn touch_cmd(filename: String) {
    let trimmed = filename.trim();
    if trimmed.is_empty() { 
        println!("Usage: touch <file_path>");
        return; 
    }
    
    match vfs_write_file(trimmed, b"New empty file.") {
        Ok(_) => println!("File '{}' created successfully", trimmed),
        Err(e) => println!("Error: {}", e),
    }
}

pub fn mkdir_cmd(dirname: String) {
    let trimmed = dirname.trim();
    if trimmed.is_empty() {
        println!("Usage: mkdir <directory_path>");
        return;
    }
    match vfs_create_dir(trimmed) {
        Ok(_) => println!("Directory '{}' created successfully", trimmed),
        Err(e) => println!("Error: {}", e),
    }
}

pub fn rm_cmd(filename: String) {
    let trimmed = filename.trim();
    if trimmed.is_empty() {
        println!("Usage: rm <file_path>");
        return;
    }
    match vfs_delete_file(trimmed) {
        Ok(_) => println!("'{}' removed successfully.", trimmed),
        Err(e) => println!("Error: {}", e),
    }
}

pub fn cd_cmd(target: String) {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        println!("Usage: cd <directory_path>");
        return;
    }

    match vfs_change_dir(trimmed) {
        Ok(_) => {}, 
        Err(e) => println!("Error: {}", e),
    }
}