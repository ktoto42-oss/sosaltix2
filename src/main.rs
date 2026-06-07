use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};
use std::env;
use std::path::Path; 
use std::process::{Command, exit};

fn main() {
    let uefi_path = env!("UEFI_PATH");

    let args: Vec<String> = env::args().collect();
    let prog = &args[0];

    let uefi = match args.get(1).map(|s| s.to_lowercase()) {
        Some(ref s) if s == "uefi" => true,
        Some(ref s) if s == "bios" => false,
        Some(ref s) if s == "-h" || s == "--help" => {
            println!("Usage: {prog} [uefi|bios]");
            println!("  uefi  - boot using OVMF (UEFI)");
            exit(0);
        }
        _ => {
            eprintln!("Usage: {prog} [uefi|bios]");
            exit(1);
        }
    };

    let disk_path = "target/disk.img";
    if !Path::new(disk_path).exists() {
        println!("Test disk image not found. Creating a 64MB raw disk at {}...", disk_path);
        
        if let Some(parent) = Path::new(disk_path).parent() {
            std::fs::create_dir_all(parent).expect("failed to create target directory");
        }

        let img_status = Command::new("qemu-img")
            .args(&["create", "-f", "raw", disk_path, "64M"])
            .status()
            .expect("failed to execute qemu-img");

        if !img_status.success() {
            eprintln!("Failed to create disk image via qemu-img");
            exit(1);
        }
    }

    let mut cmd = Command::new("qemu-system-x86_64");
    
    cmd.arg("-machine").arg("q35");
    cmd.arg("-vga").arg("virtio");
    cmd.arg("-enable-kvm");
    cmd.arg("-serial").arg("mon:stdio");
    cmd.arg("-display").arg("gtk");
    cmd.arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04");
    cmd.arg("-drive").arg(format!("file={},if=none,id=disk0,format=raw", disk_path));
    cmd.arg("-device").arg("virtio-blk-pci,drive=disk0");

    if uefi {
        let prebuilt =
            Prebuilt::fetch(Source::LATEST, "target/ovmf").expect("failed to update prebuilt");

        let code = prebuilt.get_file(Arch::X64, FileType::Code);
        let vars = prebuilt.get_file(Arch::X64, FileType::Vars);

        cmd.arg("-drive")
            .arg(format!("format=raw,file={uefi_path}"));
        cmd.arg("-drive").arg(format!(
            "if=pflash,format=raw,unit=0,file={},readonly=on",
            code.display()
        ));
        cmd.arg("-drive").arg(format!(
            "if=pflash,format=raw,unit=1,file={},snapshot=on",
            vars.display()
        ));
    }

    let mut child = cmd.spawn().expect("failed to start qemu-system-x86_64");
    let status = child.wait().expect("failed to wait on qemu");
    match status.code().unwrap_or(1) {
        0x10 => 0,
        0x11 => 1,
        _    => 2,
    };
}