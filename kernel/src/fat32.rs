use crate::virtio::DISK;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;

const SECTOR_SIZE: usize = 512;
const END_OF_CLUSTER_CHAIN: u32 = 0x0FFFFFF8;

use spin::Mutex;

pub static CURRENT_DIR_CLUSTER: Mutex<u32> = Mutex::new(0);

#[repr(align(512))]
struct SafeSectorBuffer([u8; SECTOR_SIZE]);

#[derive(Clone)]
pub struct Fat32FileSystem {
    pub boot_sector: u64,
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub num_fats: u8,
    pub sectors_per_fat: u32,
    pub root_cluster: u32,
    pub fat_start_sector: u64,
    pub data_start_sector: u64,
}

impl Fat32FileSystem {
    pub fn init() -> Option<Self> {
        let mut safe_buf = SafeSectorBuffer([0u8; SECTOR_SIZE]);
        let mut boot_sector = 0u64;
        
        {
            let mut disk_lock = DISK.lock();
            if let Some(ref mut disk) = *disk_lock {
                disk.read_blocks(0, &mut safe_buf.0).ok()?;
            } else {
                return None;
            }
        }

        let mut buf = &safe_buf.0;
        if buf[510] != 0x55 || buf[511] != 0xAA {
            return None;
        }

        let mut bytes_per_sector = u16::from_le_bytes([buf[11], buf[12]]);
        let mut sectors_per_cluster = buf[13];
        
        let is_fat32 = bytes_per_sector == 512 && 
                       (sectors_per_cluster == 1 || sectors_per_cluster == 2 || 
                        sectors_per_cluster == 4 || sectors_per_cluster == 8 || 
                        sectors_per_cluster == 16 || sectors_per_cluster == 32 || 
                        sectors_per_cluster == 64 || sectors_per_cluster == 128);

        if !is_fat32 {
            let start_lba = u32::from_le_bytes([buf[446 + 8], buf[446 + 9], buf[446 + 10], buf[446 + 11]]);
            if start_lba != 0 {
                boot_sector = start_lba as u64;
                {
                    let mut disk_lock = DISK.lock();
                    if let Some(ref mut disk) = *disk_lock {
                        disk.read_blocks(boot_sector as usize, &mut safe_buf.0).ok()?;
                    }
                }
                buf = &safe_buf.0;

                if buf[510] != 0x55 || buf[511] != 0xAA {
                    return None;
                }

                bytes_per_sector = u16::from_le_bytes([buf[11], buf[12]]);
                sectors_per_cluster = buf[13];
                
                if bytes_per_sector != 512 || sectors_per_cluster == 0 {
                    return None; 
                }
            } else {
                return None;
            }
        }

        let reserved_sectors = u16::from_le_bytes([buf[14], buf[15]]);
        let num_fats = buf[16];
        let sectors_per_fat = u32::from_le_bytes([buf[36], buf[37], buf[38], buf[39]]);
        let root_cluster = u32::from_le_bytes([buf[44], buf[45], buf[46], buf[47]]);

        let fat_start_sector = reserved_sectors as u64;
        let data_start_sector = fat_start_sector + (num_fats as u64 * sectors_per_fat as u64);

        let fs = Fat32FileSystem {
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sectors,
            num_fats,
            sectors_per_fat,
            root_cluster,
            fat_start_sector,
            data_start_sector,
            boot_sector,
        };

        let mut current_lock = CURRENT_DIR_CLUSTER.lock();
        if *current_lock == 0 {
            *current_lock = fs.root_cluster; 
        }

        Some(fs)
    }

    fn cluster_to_sector(&self, cluster: u32) -> u64 {
        if cluster < 2 {
            return self.boot_sector + self.data_start_sector;
        }
        self.boot_sector + self.data_start_sector + ((cluster as u64 - 2) * self.sectors_per_cluster as u64)
    }

    fn get_next_cluster(&self, current_cluster: u32) -> u32 {
        if current_cluster < 2 {
            return END_OF_CLUSTER_CHAIN;
        }
        let fat_offset = current_cluster as u64 * 4;
        let sector = self.boot_sector + self.fat_start_sector + (fat_offset / SECTOR_SIZE as u64);
        let offset = (fat_offset % SECTOR_SIZE as u64) as usize;

        let mut safe_buf = SafeSectorBuffer([0u8; SECTOR_SIZE]);
        {
            let mut disk_lock = DISK.lock();
            if let Some(ref mut d) = *disk_lock {
                if d.read_blocks(sector as usize, &mut safe_buf.0).is_ok() {
                    let buf = &safe_buf.0;
                    let next = u32::from_le_bytes([buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3]]);
                    let next_cluster = next & 0x0FFFFFFF;
                
                    if next_cluster == current_cluster || next_cluster < 2 {
                        return END_OF_CLUSTER_CHAIN;
                    }
                    return next_cluster;
                }
            }
        }
        END_OF_CLUSTER_CHAIN
    }

    pub fn list_dir(&self, target: Option<&str>) -> Result<(), &'static str> {
        let target_cluster = match target {
            Some(path) if !path.trim().is_empty() => {
                let (cluster, is_dir, _) = self.resolve_path(path)?;
                if !is_dir {
                    return Err("Target is a file, not a directory");
                }
                cluster
            }
            _ => *CURRENT_DIR_CLUSTER.lock(),
        };

        let mut cluster = target_cluster;
        let cluster_bytes = self.sectors_per_cluster as usize * SECTOR_SIZE;
        let mut buf = vec![0u8; cluster_bytes];

        while cluster < END_OF_CLUSTER_CHAIN {
            let start_sector = self.cluster_to_sector(cluster);
            {
                let mut disk_lock = DISK.lock();
                if let Some(ref mut d) = *disk_lock {
                    for i in 0..self.sectors_per_cluster as usize {
                        let offset = i * SECTOR_SIZE;
                        let _ = d.read_blocks((start_sector + i as u64) as usize, &mut buf[offset..offset + SECTOR_SIZE]);
                    }
                }
            }

            for entry in buf.chunks_exact(32) {
                if entry[0] == 0x00 { break; }
                if entry[0] == 0xE5 { continue; }

                if entry[0..11] == *b".          " || entry[0..11] == *b"..         " {
                    continue;
                }

                let mut name = String::new();
                for &b in &entry[0..8] { if b != b' ' { name.push(b as char); } }
                let mut ext = String::new();
                for &b in &entry[8..11] { if b != b' ' { ext.push(b as char); } }

                let is_dir = (entry[11] & 0x10) != 0;
                if is_dir {
                    crate::println!("{}/", name);
                } else {
                    if ext.is_empty() {
                        crate::println!("{}", name);
                    } else {
                        crate::println!("{}.{}", name, ext);
                    }
                }
            }
            cluster = self.get_next_cluster(cluster);
        }

        Ok(())
    }

    pub fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        let (file_cluster, is_dir, file_size) = self.resolve_path(path).ok()?;
        
        if is_dir {
            return None; 
        }

        let cluster_bytes = self.sectors_per_cluster as usize * SECTOR_SIZE;
        let mut file_data = Vec::with_capacity(file_size as usize);
        let mut bytes_left = file_size as usize;
        let mut current_cluster = file_cluster;

        while current_cluster < END_OF_CLUSTER_CHAIN && current_cluster >= 2 && bytes_left > 0 {
            let f_sector = self.cluster_to_sector(current_cluster);
            let mut cluster_buf = vec![0u8; cluster_bytes];

            {
                let mut d_lock = DISK.lock();
                if let Some(ref mut d) = *d_lock {
                    for i in 0..self.sectors_per_cluster as usize {
                        let offset = i * SECTOR_SIZE;
                        let mut temp = SafeSectorBuffer([0u8; SECTOR_SIZE]);
                        let _ = d.read_blocks((f_sector + i as u64) as usize, &mut temp.0);
                        cluster_buf[offset..offset + SECTOR_SIZE].copy_from_slice(&temp.0);
                    }
                }
            }

            let to_copy = core::cmp::min(bytes_left, cluster_bytes);
            file_data.extend_from_slice(&cluster_buf[0..to_copy]);
            bytes_left -= to_copy;

            let next_c = self.get_next_cluster(current_cluster);
            if next_c == current_cluster { break; }
            current_cluster = next_c;
        }

        Some(file_data)
    }

    fn parse_to_8_3(&self, filename: &str) -> Result<([u8; 8], [u8; 3]), &'static str> {
        let mut name = [b' '; 8];
        let mut ext = [b' '; 3];
        let mut parts = filename.split('.');
    
        let n_str = parts.next().unwrap_or("");
        let e_str = parts.next().unwrap_or("");
    
        if n_str.len() > 8 || e_str.len() > 3 {
            return Err("Filename too long! Standard 8.3 limit exceeded.");
        }
    
        for (i, b) in n_str.as_bytes().iter().enumerate() { name[i] = b.to_ascii_uppercase(); }
        for (i, b) in e_str.as_bytes().iter().enumerate() { ext[i] = b.to_ascii_uppercase(); }
        Ok((name, ext))
    }

    fn find_free_cluster(&self) -> Option<u32> {
        let mut safe_buf = SafeSectorBuffer([0u8; SECTOR_SIZE]);
        for s in 0..self.sectors_per_fat {
            let sector = self.boot_sector + self.fat_start_sector + s as u64;
            {
                let mut disk_lock = DISK.lock();
                if let Some(ref mut d) = *disk_lock {
                    d.read_blocks(sector as usize, &mut safe_buf.0).ok()?;
                }
            }
            let buf = &safe_buf.0;
            for offset in (0..SECTOR_SIZE).step_by(4) {
                let val = u32::from_le_bytes([buf[offset], buf[offset+1], buf[offset+2], buf[offset+3]]) & 0x0FFFFFFF;
                if val == 0 {
                    let cluster = (s as u32 * (SECTOR_SIZE as u32 / 4)) + (offset as u32 / 4);
                    if cluster >= 2 { return Some(cluster); }
                }
            }
        }
        None
    }

    fn write_fat_entry(&self, cluster: u32, value: u32) -> Option<()> {
        if cluster < 2 { return None; }
        let fat_offset = cluster as u64 * 4;
        let sector = self.boot_sector + self.fat_start_sector + (fat_offset / SECTOR_SIZE as u64);
        let offset = (fat_offset % SECTOR_SIZE as u64) as usize;
        let mut safe_buf = SafeSectorBuffer([0u8; SECTOR_SIZE]);
        {
            let mut disk_lock = DISK.lock();
            if let Some(ref mut d) = *disk_lock {
                d.read_blocks(sector as usize, &mut safe_buf.0).ok()?;
                let val_bytes = (value & 0x0FFFFFFF).to_le_bytes();
                safe_buf.0[offset..offset+4].copy_from_slice(&val_bytes);
                d.write_blocks(sector as usize, &safe_buf.0).ok()?;
            }
        }
        Some(())
    }

    pub fn write_file(&self, path: &str, data: &[u8]) -> Result<(), &'static str> {
        let (parent_cluster, filename) = self.resolve_parent_and_name(path)?;
        let mut cluster = parent_cluster;
        let cluster_bytes = self.sectors_per_cluster as usize * SECTOR_SIZE;
        let mut buf = vec![0u8; cluster_bytes];

        let _ = self.delete_file(path);

        let (name_8, ext_3) = self.parse_to_8_3(&filename)?;

        let first_cluster = self.find_free_cluster().ok_or("Disk Full: No free clusters found")?;
        self.write_fat_entry(first_cluster, END_OF_CLUSTER_CHAIN).ok_or("Failed to update FAT")?;

        while cluster < END_OF_CLUSTER_CHAIN {
            let start_sector = self.cluster_to_sector(cluster);
            {
                let mut disk_lock = DISK.lock();
                if let Some(ref mut d) = *disk_lock {
                    for i in 0..self.sectors_per_cluster as usize {
                        let offset = i * SECTOR_SIZE;
                        let _ = d.read_blocks((start_sector + i as u64) as usize, &mut buf[offset..offset + SECTOR_SIZE]);
                    }
                }
            }

            for (idx, entry) in buf.chunks_exact_mut(32).enumerate() {
                if entry[0] == 0x00 || entry[0] == 0xE5 {
                    entry[0..8].copy_from_slice(&name_8);
                    entry[8..11].copy_from_slice(&ext_3);
                    entry[11] = 0x00; 

                    let high_bytes = ((first_cluster >> 16) as u16).to_le_bytes();
                    let low_bytes = (first_cluster as u16).to_le_bytes();
                    entry[20..22].copy_from_slice(&high_bytes);
                    entry[26..28].copy_from_slice(&low_bytes);

                    let size_bytes = (data.len() as u32).to_le_bytes();
                    entry[28..32].copy_from_slice(&size_bytes);

                    let sub_sector_idx = idx * 32 / SECTOR_SIZE;
                    let target_sector = start_sector + sub_sector_idx as u64;
                    let buf_offset = sub_sector_idx * SECTOR_SIZE;
                    {
                        let mut disk_lock = DISK.lock();
                        if let Some(ref mut d) = *disk_lock {
                            d.write_blocks(target_sector as usize, &buf[buf_offset..buf_offset + SECTOR_SIZE]).map_err(|_| "Disk write failed")?;
                        }
                    }

                    let mut bytes_written = 0;
                    let mut current_file_cluster = first_cluster;

                    while bytes_written < data.len() {
                        let f_sector = self.cluster_to_sector(current_file_cluster);
                        let mut chunk_buf = vec![0u8; cluster_bytes];
                        
                        let to_copy = core::cmp::min(data.len() - bytes_written, cluster_bytes);
                        chunk_buf[..to_copy].copy_from_slice(&data[bytes_written..bytes_written + to_copy]);

                        let mut d_lock = DISK.lock();
                        if let Some(ref mut d) = *d_lock {
                            for i in 0..self.sectors_per_cluster as usize {
                                let offset = i * SECTOR_SIZE;
                                let _ = d.write_blocks((f_sector + i as u64) as usize, &chunk_buf[offset..offset + SECTOR_SIZE]);
                            }
                        }

                        bytes_written += to_copy;
                        if bytes_written < data.len() {
                            let next_c = self.find_free_cluster().ok_or("Disk full during write")?;
                            self.write_fat_entry(current_file_cluster, next_c);
                            self.write_fat_entry(next_c, END_OF_CLUSTER_CHAIN);
                            current_file_cluster = next_c;
                        }
                    }

                    return Ok(());
                }
            }
            cluster = self.get_next_cluster(cluster);
        }
        Err("Directory is full!")
    }

    pub fn delete_file(&self, path: &str) -> Result<(), &'static str> {
        let (parent_cluster, filename) = self.resolve_parent_and_name(path)?;
        let mut cluster = parent_cluster;
        let cluster_bytes = self.sectors_per_cluster as usize * SECTOR_SIZE;
        let mut buf = vec![0u8; cluster_bytes];

        while cluster < END_OF_CLUSTER_CHAIN && cluster >= 2 {
            let start_sector = self.cluster_to_sector(cluster);
            {
                let mut disk_lock = DISK.lock();
                if let Some(ref mut d) = *disk_lock {
                    for i in 0..self.sectors_per_cluster as usize {
                        let offset = i * SECTOR_SIZE;
                        let mut temp = SafeSectorBuffer([0u8; SECTOR_SIZE]);
                        let _ = d.read_blocks((start_sector + i as u64) as usize, &mut temp.0);
                        buf[offset..offset + SECTOR_SIZE].copy_from_slice(&temp.0);
                    }
                }
            }

            for (idx, entry) in buf.chunks_exact_mut(32).enumerate() {
                if entry[0] == 0x00 { return Err("File not found"); }
                if entry[0] == 0xE5 || entry[11] == 0x0F { continue; }

                let mut name = String::new();
                for i in 0..8 { if entry[i] != b' ' { name.push(entry[i] as char); } }
                let mut ext = String::new();
                for i in 8..11 { if entry[i] != b' ' { ext.push(entry[i] as char); } }
                let full_name = if ext.is_empty() { name } else { [name, ext].join(".") };

                if full_name == filename.to_ascii_uppercase() {
                    let high = u16::from_le_bytes([entry[20], entry[21]]) as u32;
                    let low = u16::from_le_bytes([entry[26], entry[27]]) as u32;
                    let mut file_cluster = (high << 16) | low;

                    while file_cluster < END_OF_CLUSTER_CHAIN && file_cluster >= 2 {
                        let next_c = self.get_next_cluster(file_cluster);
                        self.write_fat_entry(file_cluster, 0x00000000);
                        if next_c == file_cluster { break; }
                        file_cluster = next_c;
                    }

                    entry[0] = 0xE5;

                    let sub_sector_idx = idx * 32 / SECTOR_SIZE;
                    let target_sector = start_sector + sub_sector_idx as u64;
                    let buf_offset = sub_sector_idx * SECTOR_SIZE;

                    let mut temp_write = SafeSectorBuffer([0u8; SECTOR_SIZE]);
                    temp_write.0.copy_from_slice(&buf[buf_offset..buf_offset + SECTOR_SIZE]);
                    {
                        let mut disk_lock = DISK.lock();
                        if let Some(ref mut d) = *disk_lock {
                            d.write_blocks(target_sector as usize, &temp_write.0).map_err(|_| "Failed to clear dir entry")?;
                        }
                    }

                    return Ok(());
                }
            }
            cluster = self.get_next_cluster(cluster);
        }
        Err("File not found")
    }

    pub fn create_dir(&self, path: &str) -> Result<(), &'static str> {
        let (parent_cluster, dirname) = self.resolve_parent_and_name(path)?;
        let (name_8, ext_3) = self.parse_to_8_3(&dirname)?;
        let mut cluster = parent_cluster;
        let cluster_bytes = self.sectors_per_cluster as usize * SECTOR_SIZE;
        let mut buf = vec![0u8; cluster_bytes];

        let new_cluster = self.find_free_cluster().ok_or("Disk full")?;
        self.write_fat_entry(new_cluster, END_OF_CLUSTER_CHAIN);

        while cluster < END_OF_CLUSTER_CHAIN {
            let start_sector = self.cluster_to_sector(cluster);
            {
                let mut disk_lock = DISK.lock();
                if let Some(ref mut d) = *disk_lock {
                    for i in 0..self.sectors_per_cluster as usize {
                        let offset = i * SECTOR_SIZE;
                        let _ = d.read_blocks((start_sector + i as u64) as usize, &mut buf[offset..offset + SECTOR_SIZE]);
                    }
                }
            }

            for (idx, entry) in buf.chunks_exact_mut(32).enumerate() {
                if entry[0] == 0x00 || entry[0] == 0xE5 {
                    entry[0..8].copy_from_slice(&name_8);
                    entry[8..11].copy_from_slice(&ext_3);
                    entry[11] = 0x10;

                    let high_bytes = ((new_cluster >> 16) as u16).to_le_bytes();
                    let low_bytes = (new_cluster as u16).to_le_bytes();
                    entry[20..22].copy_from_slice(&high_bytes);
                    entry[26..28].copy_from_slice(&low_bytes);

                    let sub_sector_idx = idx * 32 / SECTOR_SIZE;
                    let target_sector = start_sector + sub_sector_idx as u64;
                    let buf_offset = sub_sector_idx * SECTOR_SIZE;

                    {
                        let mut disk_lock = DISK.lock();
                        if let Some(ref mut d) = *disk_lock {
                            d.write_blocks(target_sector as usize, &buf[buf_offset..buf_offset + SECTOR_SIZE]).map_err(|_| "Write failed")?;
                        }
                    } 

                    let mut dir_content = vec![0u8; cluster_bytes];
                    
                    dir_content[0..11].copy_from_slice(b".          ");
                    dir_content[11] = 0x10;
                    dir_content[20..22].copy_from_slice(&high_bytes);
                    dir_content[26..28].copy_from_slice(&low_bytes);

                    dir_content[32..43].copy_from_slice(b"..         ");
                    dir_content[43] = 0x10;

                    let f_sector = self.cluster_to_sector(new_cluster);

                    {
                        let mut d_lock = DISK.lock();
                        if let Some(ref mut d) = *d_lock {
                            for i in 0..self.sectors_per_cluster as usize {
                                let offset = i * SECTOR_SIZE;
                                let _ = d.write_blocks((f_sector + i as u64) as usize, &dir_content[offset..offset + SECTOR_SIZE]);
                            }
                        }
                    }
                    
                    return Ok(());
                }
            }
            cluster = self.get_next_cluster(cluster);
        }
        Err("Root directory full")
    }

    pub fn change_dir(&self, target: &str) -> Result<(), &'static str> {
        let (target_cluster, is_dir, _) = self.resolve_path(target)?;
        
        if is_dir {
            *CURRENT_DIR_CLUSTER.lock() = target_cluster;
            Ok(())
        } else {
            Err("Target is a file, not a directory")
        }
    }

    pub fn append_to_file(&self, filename: &str, data: &[u8]) -> Result<(), &'static str> {
        if data.is_empty() { return Ok(()); }

        let current_dir_cluster = *CURRENT_DIR_CLUSTER.lock();
        let mut cluster = current_dir_cluster;
        let cluster_bytes = self.sectors_per_cluster as usize * SECTOR_SIZE;
        let mut buf = vec![0u8; cluster_bytes];

        let mut found_entry = None;
        let mut entry_cluster = 0;
        let mut entry_idx = 0;
        let mut entry_start_sector = 0;

        while cluster < END_OF_CLUSTER_CHAIN {
            let start_sector = self.cluster_to_sector(cluster);
            {
                let mut disk_lock = DISK.lock();
                if let Some(ref mut d) = *disk_lock {
                    for i in 0..self.sectors_per_cluster as usize {
                        let offset = i * SECTOR_SIZE;
                        let _ = d.read_blocks((start_sector + i as u64) as usize, &mut buf[offset..offset + SECTOR_SIZE]);
                    }
                }
            }

            for (idx, entry) in buf.chunks_exact(32).enumerate() {
                if entry[0] == 0x00 { break; }
                if entry[0] == 0xE5 || entry[11] == 0x0F { continue; }

                let mut name = String::new();
                for i in 0..8 { if entry[i] != b' ' { name.push(entry[i] as char); } }
                let mut ext = String::new();
                for i in 8..11 { if entry[i] != b' ' { ext.push(entry[i] as char); } }
                let full_name = if ext.is_empty() { name } else { [name, ext].join(".") };

                if full_name == filename.to_ascii_uppercase() {
                    if (entry[11] & 0x10) != 0 {
                        return Err("Target is a directory, cannot append data to it");
                    }
                    found_entry = Some(entry.to_vec());
                    entry_cluster = cluster;
                    entry_idx = idx;
                    entry_start_sector = start_sector;
                    break;
                }
            }
            if found_entry.is_some() { break; }
            cluster = self.get_next_cluster(cluster);
        }

        let entry_bytes = match found_entry {
            Some(e) => e,
            None => return self.write_file(filename, data),
        };

        let current_size = u32::from_le_bytes([entry_bytes[28], entry_bytes[29], entry_bytes[30], entry_bytes[31]]) as usize;
        let high = u16::from_le_bytes([entry_bytes[20], entry_bytes[21]]) as u32;
        let low = u16::from_le_bytes([entry_bytes[26], entry_bytes[27]]) as u32;
        let first_cluster = (high << 16) | low;

        let mut last_cluster = first_cluster;
        let mut c = first_cluster;
        while c < END_OF_CLUSTER_CHAIN && c >= 2 {
            last_cluster = c;
            c = self.get_next_cluster(c);
        }

        let mut bytes_written = 0;
        let mut current_cluster = last_cluster;

        let mut last_cluster_offset = current_size % cluster_bytes;

        while bytes_written < data.len() {
            let f_sector = self.cluster_to_sector(current_cluster);
            let mut cluster_buf = vec![0u8; cluster_bytes];

            {
                let mut d_lock = DISK.lock();
                if let Some(ref mut d) = *d_lock {
                    for i in 0..self.sectors_per_cluster as usize {
                        let offset = i * SECTOR_SIZE;
                        let _ = d.read_blocks((f_sector + i as u64) as usize, &mut cluster_buf[offset..offset + SECTOR_SIZE]);
                    }
                }
            }

            let space_left = cluster_bytes - last_cluster_offset;
            let to_copy = core::cmp::min(data.len() - bytes_written, space_left);

            cluster_buf[last_cluster_offset..last_cluster_offset + to_copy].copy_from_slice(&data[bytes_written..bytes_written + to_copy]);

            {
                let mut d_lock = DISK.lock();
                if let Some(ref mut d) = *d_lock {
                    for i in 0..self.sectors_per_cluster as usize {
                        let offset = i * SECTOR_SIZE;
                        let _ = d.write_blocks((f_sector + i as u64) as usize, &cluster_buf[offset..offset + SECTOR_SIZE]);
                    }
                }
            }

            bytes_written += to_copy;
            last_cluster_offset = 0;

            if bytes_written < data.len() {
                let next_c = self.find_free_cluster().ok_or("Disk full during append")?;
                self.write_fat_entry(current_cluster, next_c);
                self.write_fat_entry(next_c, END_OF_CLUSTER_CHAIN);
                current_cluster = next_c;
            }
        }

        let mut dir_buf = vec![0u8; cluster_bytes];
        {
            let mut disk_lock = DISK.lock();
            if let Some(ref mut d) = *disk_lock {
                for i in 0..self.sectors_per_cluster as usize {
                    let offset = i * SECTOR_SIZE;
                    let _ = d.read_blocks((entry_start_sector + i as u64) as usize, &mut dir_buf[offset..offset + SECTOR_SIZE]);
                }
            }
        }

        let entry_mut = &mut dir_buf[entry_idx * 32..(entry_idx + 1) * 32];
        let new_size = (current_size + data.len()) as u32;
        entry_mut[28..32].copy_from_slice(&new_size.to_le_bytes());

        let sub_sector_idx = entry_idx * 32 / SECTOR_SIZE;
        let target_sector = entry_start_sector + sub_sector_idx as u64;
        let buf_offset = sub_sector_idx * SECTOR_SIZE;
        {
            let mut disk_lock = DISK.lock();
            if let Some(ref mut d) = *disk_lock {
                d.write_blocks(target_sector as usize, &dir_buf[buf_offset..buf_offset + SECTOR_SIZE]).map_err(|_| "Failed to update file size in directory")?;
            }
        }

        Ok(())
    }

    pub fn find_entry_in_cluster(&self, parent_cluster: u32, name: &str) -> Result<(u32, bool, u32), &'static str> {
        let trimmed = name.trim();
        if trimmed.is_empty() { return Err("Empty component name"); }

        let mut target_bytes = [b' '; 11];
        if trimmed == "." {
            target_bytes[0] = b'.';
        } else if trimmed == ".." {
            target_bytes[0] = b'.';
            target_bytes[1] = b'.';
        } else {
            let (n8, e3) = self.parse_to_8_3(trimmed)?;
            target_bytes[0..8].copy_from_slice(&n8);
            target_bytes[8..11].copy_from_slice(&e3);
        }

        let mut cluster = if parent_cluster == 0 { self.root_cluster } else { parent_cluster };
        let cluster_bytes = self.sectors_per_cluster as usize * SECTOR_SIZE;
        let mut buf = vec![0u8; cluster_bytes];

        while cluster < END_OF_CLUSTER_CHAIN {
            let start_sector = self.cluster_to_sector(cluster);
            {
                let mut disk_lock = DISK.lock();
                if let Some(ref mut d) = *disk_lock {
                    for i in 0..self.sectors_per_cluster as usize {
                        let offset = i * SECTOR_SIZE;
                        let _ = d.read_blocks((start_sector + i as u64) as usize, &mut buf[offset..offset + SECTOR_SIZE]);
                    }
                }
            }

            for entry in buf.chunks_exact(32) {
                if entry[0] == 0x00 { break; }
                if entry[0] == 0xE5 { continue; }

                if entry[0..11] == target_bytes {
                    let is_dir = (entry[11] & 0x10) != 0;
                    let high = u16::from_le_bytes([entry[20], entry[21]]) as u32;
                    let low = u16::from_le_bytes([entry[26], entry[27]]) as u32;
                    let mut target_cluster = (high << 16) | low;

                    if target_cluster == 0 {
                        target_cluster = self.root_cluster;
                    }
                    
                    let size = u32::from_le_bytes([entry[28], entry[29], entry[30], entry[31]]);
                    return Ok((target_cluster, is_dir, size));
                }
            }
            cluster = self.get_next_cluster(cluster);
        }

        Err("File or directory not found")
    }

    pub fn resolve_path(&self, path: &str) -> Result<(u32, bool, u32), &'static str> {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Ok((*CURRENT_DIR_CLUSTER.lock(), true, 0));
        }

        let mut current_cluster = if trimmed.starts_with('/') {
            self.root_cluster
        } else {
            *CURRENT_DIR_CLUSTER.lock()
        };

        if trimmed == "/" {
            return Ok((self.root_cluster, true, 0));
        }

        let components: alloc::vec::Vec<&str> = trimmed
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        let mut is_dir = true;
        let mut size = 0;

        for component in components {
            if !is_dir {
                return Err("Path resolution error: encountered a file where a directory was expected");
            }

            let (next_cluster, next_is_dir, next_size) = self.find_entry_in_cluster(current_cluster, component)?;
            current_cluster = next_cluster;
            is_dir = next_is_dir;
            size = next_size;
        }

        Ok((current_cluster, is_dir, size))
    }

    pub fn resolve_parent_and_name(&self, path: &str) -> Result<(u32, String), &'static str> {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Err("Empty path");
        }

        if let Some(idx) = trimmed.rfind('/') {
            let parent_path = &trimmed[..idx];
            let name = &trimmed[idx + 1..];

            if name.is_empty() {
                return Err("Path cannot end with a slash for this operation");
            }

            let parent_path = if parent_path.is_empty() { "/" } else { parent_path };

            let (parent_cluster, is_dir, _) = self.resolve_path(parent_path)?;
            if !is_dir {
                return Err("Parent component is not a directory");
            }

            Ok((parent_cluster, String::from(name)))
        } else {
            Ok((*CURRENT_DIR_CLUSTER.lock(), String::from(trimmed)))
        }
    }
}

impl crate::vfs::FileSystem for Fat32FileSystem {
    fn read_file(&self, path: &str) -> Option<alloc::vec::Vec<u8>> {
        self.read_file(path)
    }

    fn write_file(&self, path: &str, data: &[u8]) -> Result<(), &'static str> {
        self.write_file(path, data)
    }

    fn append_file(&self, path: &str, data: &[u8]) -> Result<(), &'static str> {
        self.append_to_file(path, data)
    }

    fn create_dir(&self, path: &str) -> Result<(), &'static str> {
        self.create_dir(path)
    }

    fn delete_file(&self, path: &str) -> Result<(), &'static str> {
        self.delete_file(path)
    }

    fn list_dir(&self, path: Option<&str>) -> Result<(), &'static str> {
        self.list_dir(path)
    }

    fn change_dir(&self, path: &str) -> Result<(), &'static str> {
        self.change_dir(path)
    }
}