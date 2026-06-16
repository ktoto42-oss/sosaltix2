use alloc::string::String;
use alloc::str;
use crate::println;
use crate::virtio::DISK;
use crate::vfs::{
    vfs_open, vfs_close, vfs_read, vfs_write, 
    vfs_create_dir, vfs_delete_file, vfs_list_dir, vfs_change_dir, OpenMode
};

const commands: &str = "clear, help, fetch, reboot, poweroff, cd, ls, cat, echo, touch, mkdir, rm";

pub fn help() {
    println!("Available commands: {commands}");
}

pub fn echo(args: String) {
    let mut mode = OpenMode::Write;
    let mut delimiter = ">";

    if args.contains(">>") {
        mode = OpenMode::Append;
        delimiter = ">>";
    } else if !args.contains('>') {
        println!("{args}");
        return;
    }

    let parts: alloc::vec::Vec<&str> = args.split(delimiter).collect();
    if parts.len() == 2 {
        let mut content = parts[0].trim();
        let filename = parts[1].trim();

        if content.starts_with('"') && content.ends_with('"') && content.len() >= 2 {
            content = &content[1..content.len() - 1];
        }

        match vfs_open(filename, mode) {
            Ok(fd) => {
                match vfs_write(fd, content.as_bytes()) {
                    Ok(_) => {
                        if mode == OpenMode::Append {
                            println!("Appended to '{}' via fd {}", filename, fd);
                        } else {
                            println!("Written to '{}' via fd {}", filename, fd);
                        }
                    }
                    Err(e) => println!("Write error: {}", e),
                }
                let _ = vfs_close(fd);
            }
            Err(e) => println!("Error opening file: {}", e),
        }
    }
}

pub fn cat_cmd(filename: String) {
    let trimmed_name = filename.trim(); 
    if trimmed_name.is_empty() {
        println!("Usage: cat <file_path>");
        return;
    }

    match vfs_open(trimmed_name, OpenMode::Read) {
        Ok(fd) => {
            let mut file_data = alloc::vec::Vec::new();
            let mut buf = [0u8; 256];

            loop {
                match vfs_read(fd, &mut buf) {
                    Ok(0) => break, 
                    Ok(n) => file_data.extend_from_slice(&buf[..n]),
                    Err(e) => {
                        println!("Read error: {}", e);
                        let _ = vfs_close(fd);
                        return;
                    }
                }
            }
            let _ = vfs_close(fd);

            match str::from_utf8(&file_data) {
                Ok(text) => println!("{}", text),
                Err(_) => println!("Error: File contains non-UTF8 data. Total bytes: {}", file_data.len()),
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}

pub fn touch_cmd(filename: String) {
    let trimmed = filename.trim();
    if trimmed.is_empty() { 
        println!("Usage: touch <file_path>");
        return; 
    }

    match vfs_open(trimmed, OpenMode::Write) {
        Ok(fd) => {
            let _ = vfs_write(fd, b"New empty file.");
            let _ = vfs_close(fd);
            println!("File '{}' created successfully", trimmed);
        }
        Err(e) => println!("Error: {}", e),
    }
}

pub fn ls_cmd(args: String) {
    let trimmed = args.trim();
    let target_dir = if trimmed.is_empty() { None } else { Some(trimmed) };
    if let Err(e) = vfs_list_dir(target_dir) { println!("Error: {}", e); }
}

pub fn mkdir_cmd(dirname: String) {
    let trimmed = dirname.trim();
    if trimmed.is_empty() { println!("Usage: mkdir <directory_path>"); return; }
    match vfs_create_dir(trimmed) {
        Ok(_) => println!("Directory '{}' created successfully", trimmed),
        Err(e) => println!("Error: {}", e),
    }
}

pub fn rm_cmd(filename: String) {
    let trimmed = filename.trim();
    if trimmed.is_empty() { println!("Usage: rm <file_path>"); return; }
    match vfs_delete_file(trimmed) {
        Ok(_) => println!("'{}' removed successfully.", trimmed),
        Err(e) => println!("Error: {}", e),
    }
}

pub fn cd_cmd(target: String) {
    let trimmed = target.trim();
    if trimmed.is_empty() { println!("Usage: cd <directory_path>"); return; }
    if let Err(e) = vfs_change_dir(trimmed) { println!("Error: {}", e); }
}

pub fn print_disk_status() {
    let mut disk_lock = DISK.lock();
    if let Some(ref mut disk) = *disk_lock {
        println!("--- Global Disk Status ---");
        println!("Status: ONLINE");
        println!("Capacity: {} sectors ({} MB)", disk.capacity(), (disk.capacity() * 512) / 1024 / 1024);
    } else {
        println!("Disk Status: OFFLINE");
    }
}

pub fn read_sector_cmd(sector_string: String) {
    let sector_id: u64 = sector_string.trim().parse().unwrap_or(0);
    let mut disk_lock = DISK.lock();
    if let Some(ref mut disk) = *disk_lock {
        let mut buf = [0u8; 512];
        match disk.read_blocks(sector_id as usize, &mut buf) {
            Ok(_) => {
                println!("Sector {} read successfully! First 16 bytes:", sector_id);
                println!("{:x?}", &buf[0..16]);
            }
            Err(e) => println!("Error reading sector {}: {:?}", sector_id, e),
        }
    } else {
        println!("Error: Disk is offline.");
    }
}