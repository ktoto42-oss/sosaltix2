use alloc::boxed::Box;
use alloc::vec::Vec;
use spin::Mutex;

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

pub fn register_root_fs(fs: Box<dyn FileSystem>) {
    *ROOT_FS.lock() = Some(fs);
}

pub fn vfs_read_file(path: &str) -> Option<Vec<u8>> {
    ROOT_FS.lock().as_ref()?.read_file(path)
}

pub fn vfs_write_file(path: &str, data: &[u8]) -> Result<(), &'static str> {
    if let Some(ref fs) = *ROOT_FS.lock() {
        fs.write_file(path, data)
    } else {
        Err("VFS: No root file system mounted")
    }
}

pub fn vfs_append_file(path: &str, data: &[u8]) -> Result<(), &'static str> {
    if let Some(ref fs) = *ROOT_FS.lock() {
        fs.append_file(path, data)
    } else {
        Err("VFS: No root file system mounted")
    }
}

pub fn vfs_create_dir(path: &str) -> Result<(), &'static str> {
    if let Some(ref fs) = *ROOT_FS.lock() {
        fs.create_dir(path)
    } else {
        Err("VFS: No root file system mounted")
    }
}

pub fn vfs_delete_file(path: &str) -> Result<(), &'static str> {
    if let Some(ref fs) = *ROOT_FS.lock() {
        fs.delete_file(path)
    } else {
        Err("VFS: No root file system mounted")
    }
}

pub fn vfs_list_dir(path: Option<&str>) -> Result<(), &'static str> {
    if let Some(ref fs) = *ROOT_FS.lock() {
        fs.list_dir(path)
    } else {
        Err("VFS: No root file system mounted")
    }
}

pub fn vfs_change_dir(path: &str) -> Result<(), &'static str> {
    if let Some(ref fs) = *ROOT_FS.lock() {
        fs.change_dir(path)
    } else {
        Err("VFS: No root file system mounted")
    }
}