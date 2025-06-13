use std::process::{Command, Stdio};
use std::path::Path;
use std::fs;

fn main() {
    let targets = vec![
        // Linux
        [     "x86_64-unknown-linux-gnu"      ,     "x64-linux-gnu", "so"],
        [    "aarch64-unknown-linux-gnu"      , "aarch64-linux-gnu", "so"],
        [      "armv7-unknown-linux-gnueabihf",   "armv7-linux-gnu", "so"],
        [        "arm-unknown-linux-gnueabihf",     "arm-linux-gnu", "so"],
        [  "riscv64gc-unknown-linux-gnu"      , "riscv64-linux-gnu", "so"],
        [       "i686-unknown-linux-gnu"      ,     "x86-linux-gnu", "so"],
        ["powerpc64le-unknown-linux-gnu"      , "ppc64le-linux-gnu", "so"],

        // Windows (GNU)
        [ "x86_64-pc-windows-gnu",     "x64-windows-gnu", "dll"],
     // [   "i686-pc-windows-gnu",     "x86-windows-gnu", "dll"],
     // ["aarch64-pc-windows-gnu", "aarch64-windows-gnu", "dll"],
     // [       "arm-windows-gnu",     "arm-windows-gnu", "dll"],

        // Android
     // ["aarch64-linux-android"    , "aarch64-android", "so"],
     // [  "armv7-linux-androideabi",   "armv7-android", "so"],
     // [ "x86_64-linux-android"    ,     "x64-android", "so"],
    ];


    let _ = fs::create_dir_all("artifacts");

    for [target, out, ext] in targets {
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
            let src = format!(
                "target/{target}/release/{}kodrst.{ext}",
                ext.eq("so").then(|| "lib").unwrap_or("")
            );
            let dst = format!("artifacts/{out}.{ext}");

            if Path::new(&src).exists() {
                fs::copy(&src, &dst)
                    .expect(format!("❌ Failed to copy output .{ext} file").as_str());

                println!("✅ Copied {src} → {dst}");
                continue;                
            }
            
            eprintln!("⚠️ Compiled .{ext} file not found at {src}");
            continue;
        }

        eprintln!("⚠️ Failed compilation for {target}");
    }
}
