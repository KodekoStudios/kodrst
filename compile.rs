use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

fn main() {
    let targets = vec![
        [   "x86_64-unknown-linux-gnu"      ,     "x64-linux-gnu"],
        [  "aarch64-unknown-linux-gnu"      , "aarch64-linux-gnu"],
        [    "armv7-unknown-linux-gnueabihf",   "armv7-linux-gnu"],
        [      "arm-unknown-linux-gnueabihf",     "arm-linux-gnu"],
        ["riscv64gc-unknown-linux-gnu"      , "riscv64-linux-gnu"],
    ];

    let _ = fs::create_dir_all("artifacts");

    for [target, out] in targets {
        println!("\n🔨 Compiling for target: {target}");

        let install_status = Command::new("rustup")
            .args(["target", "add", target])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("❌ Failed to run rustup");

        if !install_status.success() {
            eprintln!("⚠️ The target could not be installed: {target}\n");
            continue;
        }

        let build_status = Command::new("cargo")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .args(["zigbuild", "--release", "--target", target])
            .status()
            .expect("❌ Failed to run cargo-zigbuild");

        if build_status.success() {
            let src = format!("target/{target}/release/libkodrst.so");
            let dst = format!("artifacts/{out}.so");

            if Path::new(&src).exists() {
                fs::copy(&src, &dst)
                    .expect("❌ Failed to copy output .so file");

                println!("✅ Copied {src} → {dst}");
                continue;                
            }
            
            eprintln!("⚠️ Compiled .so file not found at {src}");
            continue;
        }

        eprintln!("⚠️ Failed compilation for {target}");
    }
}
