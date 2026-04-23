use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("bios");
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let img = match mode {
        "bios" => format!("{}/bios.img", out_dir),
        "uefi" => format!("{}/uefi.img", out_dir),
        _ => panic!("unknown mode: {}", mode),
    };
    
    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.arg("-drive").arg(format!("format=raw,file={}", img));
    
    // Для отладки
    if mode == "uefi" {
        cmd.arg("-bios").arg("OVMF.4m.fd"); // путь к вашему OVMF
    }
    
    let status = cmd.status().expect("failed to run QEMU");
    std::process::exit(status.code().unwrap_or(1));
}