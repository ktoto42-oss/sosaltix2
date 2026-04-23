use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    // ← Имя переменной: CARGO_BIN_FILE_{UPPERCASE_PACKAGE_NAME}_{BIN_NAME}
    // У нас: package = "sosaltix2", bin = "kernel"
    let kernel = env::var_os("CARGO_BIN_FILE_SOSALTIX2_KERNEL")
        .context("kernel artifact not found: CARGO_BIN_FILE_SOSALTIX2_KERNEL")?;
    
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // BIOS образ
    let bios_img = out_dir.join("bios.img");
    bootloader::BiosBoot::new()
        .create_disk_image(&kernel, &bios_img)
        .context("failed to create BIOS disk image")?;
    
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}


