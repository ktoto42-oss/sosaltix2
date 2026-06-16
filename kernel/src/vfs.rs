use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpenMode {
    Read,
    Write,
    Append,
}

#[derive(Debug, Clone)]
pub struct OpenFile {
    pub path: String,
    pub offset: usize,
    pub mode: OpenMode,
}

pub trait FileSystem: Send + Sync {
    fn read_file(&self, path: &str) -> Option<Vec<u8>>;
    fn write_file(&self, path: &str, data: &[u8]) -> Result<(), &'static str>;
    fn append_file(&self, path: &str, data: &[u8]) -> Result<(), &'static str>;
    fn create_dir(&self, path: &str) -> Result<(), &'static str>;
    fn delete_file(&self, path: &str) -> Result<(), &'static str>;
    fn list_dir(&self, path: Option<&str>) -> Result<(), &'static str>;
    fn change_dir(&self, path: &str) -> Result<(), &'static str>;
}

pub static ROOT_FS: Mutex<Option<Box<dyn FileSystem>>> = Mutex::new(None);

pub static FILE_TABLE: Mutex<Vec<Option<OpenFile>>> = Mutex::new(Vec::new());

pub fn register_root_fs(fs: Box<dyn FileSystem>) {
    *ROOT_FS.lock() = Some(fs);
}

pub fn vfs_open(path: &str, mode: OpenMode) -> Result<usize, &'static str> {
    let fs_lock = ROOT_FS.lock();
    let fs = fs_lock.as_ref().ok_or("VFS: No root file system mounted")?;

    if mode == OpenMode::Read {
        if fs.read_file(path).is_none() {
            return Err("VFS: File not found");
        }
    } else if mode == OpenMode::Write {
        fs.write_file(path, &[])?;
    }

    let mut table = FILE_TABLE.lock();
    let open_file = OpenFile {
        path: String::from(path),
        offset: 0,
        mode,
    };

    for (fd, slot) in table.iter_mut().enumerate() {
        if slot.is_none() {
            *slot = Some(open_file);
            return Ok(fd);
        }
    }

    table.push(Some(open_file));
    Ok(table.len() - 1)
}

pub fn vfs_close(fd: usize) -> Result<(), &'static str> {
    let mut table = FILE_TABLE.lock();
    if fd < table.len() && table[fd].is_some() {
        table[fd] = None;
        Ok(())
    } else {
        Err("VFS: Invalid or already closed file descriptor")
    }
}

pub fn vfs_read(fd: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut table = FILE_TABLE.lock();
    if fd >= table.len() || table[fd].is_none() {
        return Err("VFS: Invalid file descriptor");
    }

    let file = table[fd].as_mut().unwrap();
    if file.mode != OpenMode::Read {
        return Err("VFS: File not opened for reading");
    }

    let fs_lock = ROOT_FS.lock();
    let fs = fs_lock.as_ref().ok_or("VFS: No root FS")?;

    if let Some(data) = fs.read_file(&file.path) {
        if file.offset >= data.len() {
            return Ok(0);
        }

        let available = data.len() - file.offset;
        let to_read = core::cmp::min(buf.len(), available);

        buf[..to_read].copy_from_slice(&data[file.offset..file.offset + to_read]);
        file.offset += to_read;

        Ok(to_read)
    } else {
        Err("VFS: Failed to read underlying file data")
    }
}

pub fn vfs_write(fd: usize, buf: &[u8]) -> Result<usize, &'static str> {
    let mut table = FILE_TABLE.lock();
    if fd >= table.len() || table[fd].is_none() {
        return Err("VFS: Invalid file descriptor");
    }

    let file = table[fd].as_mut().unwrap();
    if file.mode == OpenMode::Read {
        return Err("VFS: File opened as read-only");
    }

    let fs_lock = ROOT_FS.lock();
    let fs = fs_lock.as_ref().ok_or("VFS: No root FS")?;

    match file.mode {
        OpenMode::Append => {
            fs.append_file(&file.path, buf)?;
            file.offset += buf.len();
            Ok(buf.len())
        }
        OpenMode::Write => {
            if file.offset == 0 {
                fs.write_file(&file.path, buf)?;
                file.offset += buf.len();
                Ok(buf.len())
            } else {
                let mut data = fs.read_file(&file.path).unwrap_or_else(Vec::new);
                if file.offset + buf.len() > data.len() {
                    data.resize(file.offset + buf.len(), 0);
                }
                data[file.offset..file.offset + buf.len()].copy_from_slice(buf);
                fs.write_file(&file.path, &data)?;
                file.offset += buf.len();
                Ok(buf.len())
            }
        }
        _ => Err("VFS: Write operation not supported for this mode"),
    }
}

pub fn vfs_create_dir(path: &str) -> Result<(), &'static str> {
    ROOT_FS.lock().as_ref().ok_or("VFS: No root FS")?.create_dir(path)
}

pub fn vfs_delete_file(path: &str) -> Result<(), &'static str> {
    ROOT_FS.lock().as_ref().ok_or("VFS: No root FS")?.delete_file(path)
}

pub fn vfs_list_dir(path: Option<&str>) -> Result<(), &'static str> {
    ROOT_FS.lock().as_ref().ok_or("VFS: No root FS")?.list_dir(path)
}

pub fn vfs_change_dir(path: &str) -> Result<(), &'static str> {
    ROOT_FS.lock().as_ref().ok_or("VFS: No root FS")?.change_dir(path)
}