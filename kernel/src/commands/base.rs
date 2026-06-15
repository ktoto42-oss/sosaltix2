use alloc::string::String;
use crate::println;
use crate::fat32::CURRENT_DIR_CLUSTER;

pub fn help() {
    println!("Available commands: \n echo \n clear \n help \n fetch \n disk-status \n read-sector \n poweroff \n reboot");
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

            if let Some(fs) = get_fs() {
                match fs.append_to_file(filename, content.as_bytes()) {
                    Ok(_) => println!("Appended to '{}'", filename),
                    Err(e) => println!("Error appending to file: {}", e),
                }
            } else {
                println!("Error: FS not mounted.");
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

            if let Some(fs) = get_fs() {
                match fs.write_file(filename, content.as_bytes()) {
                    Ok(_) => println!("Written to '{}'", filename),
                    Err(e) => println!("Error writing to file: {}", e),
                }
            } else {
                println!("Error: FS not mounted.");
            }
            return;
        }
    }

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

use crate::fat32::Fat32FileSystem;
use alloc::str;

pub fn ls_cmd(args: String) {
    if let Some(fs) = get_fs() {
        let trimmed = args.trim();
        let target_dir = if trimmed.is_empty() { None } else { Some(trimmed) };
        
        if let Err(e) = fs.list_dir(target_dir) {
            println!("Error: {}", e);
        }
    } else {
        println!("Error: Failed to mount file system.");
    }
}

pub fn cat_cmd(filename: String) {
    let trimmed_name = filename.trim(); 
    if trimmed_name.is_empty() {
        println!("Usage: cat <file_path>");
        return;
    }
    
    if let Some(fs) = get_fs() {
        if let Some(data) = fs.read_file(trimmed_name) {
            match str::from_utf8(&data) {
                Ok(text) => println!("{}", text),
                Err(_) => println!("Error: File contains non-UTF8 data. Total bytes: {}", data.len()),
            }
        } else {
            println!("Error: File '{}' not found.", trimmed_name);
        }
    } else {
        println!("Error: Failed to mount FAT32 file system.");
    }
}

fn get_fs() -> Option<crate::fat32::Fat32FileSystem> {
    crate::fat32::Fat32FileSystem::init()
}

pub fn touch_cmd(filename: String) {
    let trimmed = filename.trim();
    if trimmed.is_empty() { 
        println!("Usage: touch <file_path>");
        return; 
    }
    
    if let Some(fs) = get_fs() {
        match fs.write_file(trimmed, b"New empty file.") {
            Ok(_) => println!("File '{}' created successfully", trimmed),
            Err(e) => println!("Error: {}", e),
        }
    }
}

pub fn mkdir_cmd(dirname: String) {
    let trimmed = dirname.trim();
    if trimmed.is_empty() {
        println!("Usage: mkdir <directory_path>");
        return;
    }
    if let Some(fs) = get_fs() {
        match fs.create_dir(trimmed) {
            Ok(_) => println!("Directory '{}' created successfully", trimmed),
            Err(e) => println!("Error: {}", e),
        }
    }
}

pub fn rm_cmd(filename: String) {
    let trimmed = filename.trim();
    if trimmed.is_empty() {
        println!("Usage: rm <file_path>");
        return;
    }
    if let Some(fs) = get_fs() {
        match fs.delete_file(trimmed) {
            Ok(_) => println!("'{}' removed successfully.", trimmed),
            Err(e) => println!("Error: {}", e),
        }
    }
}

pub fn cd_cmd(target: String) {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        println!("Usage: cd <directory_path>");
        return;
    }

    if let Some(fs) = get_fs() {
        match fs.resolve_path(trimmed) {
            Ok((cluster, is_dir, _)) => {
                if is_dir {
                    *CURRENT_DIR_CLUSTER.lock() = cluster;
                } else {
                    println!("Error: '{}' is a file, not a directory.", trimmed);
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}