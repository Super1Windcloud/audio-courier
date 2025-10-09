use std::env;
use std::path::{Path, PathBuf};

fn main() {
    tauri_build::build();

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    match target_os.as_str() {
        "windows" => {
            println!("cargo:rerun-if-changed=build.rs");

            // 检查是否存在 vosk-win64 目录
            let vosk_path = PathBuf::from("vosk-win64-0.3.45");
            if !vosk_path.exists() {
                println!("cargo:warning=Vosk 库目录不存在: {:?}", vosk_path);
                println!("cargo:warning=请下载 Vosk Windows 库并解压到项目根目录");
                return;
            }

            println!("cargo:rustc-link-search=native={}", vosk_path.display());

            // 检查架构并链接对应的库
            let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
            match target_arch.as_str() {
                "x86_64" => {
                    // 对于 64 位 Windows，链接动态库
                    println!("cargo:rustc-link-lib=dylib=libvosk");

                    // 复制必要的 DLL 到输出目录
                    let out_dir = env::var("OUT_DIR").unwrap();
                    let target_dir = get_target_dir(&out_dir);
                    copy_dlls(&vosk_path, &target_dir);
                }
                "x86" => {
                    println!("cargo:rustc-link-lib=dylib=libvosk");
                }
                _ => {
                    println!("cargo:warning=不支持的 Windows 架构: {}", target_arch);
                }
            }
        }
        "macos" => {
            println!("cargo:rustc-link-lib=dylib=vosk");
        }
        "linux" => {
            println!("cargo:rustc-link-lib=dylib=vosk");
        }
        _ => {
            println!("cargo:warning=暂时不支持的平台: {}", target_os);
        }
    }
}

// 从 OUT_DIR 推导出 target 目录
fn get_target_dir(out_dir: &str) -> PathBuf {
    let mut path = PathBuf::from(out_dir);

    while let Some(parent) = path.parent() {
        if parent.file_name() == Some(std::ffi::OsStr::new("target")) {
            return parent.to_path_buf();
        }
        path = parent.to_path_buf();
    }

    // 如果找不到 target 目录，使用默认路径
    PathBuf::from("target")
}

#[cfg(windows)]
fn copy_dlls(vosk_path: &Path, target_dir: &Path) {
    use std::fs;

    let dll_files = [
        "libvosk.dll",
        "libgcc_s_seh-1.dll",
        "libstdc++-6.dll",
        "libwinpthread-1.dll",
    ];

    // 需要复制 DLL 到的多个目录
    let copy_locations = [
        target_dir.to_path_buf(),
        target_dir.join("debug"),
        target_dir.join("debug").join("deps"),
        target_dir.join("release"),
        target_dir.join("release").join("deps"),
    ];

    for location in &copy_locations {
        if let Err(e) = fs::create_dir_all(location) {
            println!("cargo:warning=无法创建目录 {}: {}", location.display(), e);
            continue;
        }

        for dll_name in &dll_files {
            let src = vosk_path.join(dll_name);
            let dst = location.join(dll_name);

            if src.exists() {
                if let Err(e) = fs::copy(&src, &dst) {
                    println!(
                        "cargo:warning=无法复制 DLL {} 到 {}: {}",
                        dll_name,
                        location.display(),
                        e
                    );
                } else {
                    println!(
                        "cargo:warning=已复制 DLL: {} -> {}",
                        src.display(),
                        dst.display()
                    );
                }
            } else {
                println!("cargo:warning=找不到 DLL: {}", src.display());
            }
        }
    }
}

#[cfg(not(windows))]
fn copy_dlls(_vosk_path: &Path, _target_dir: &Path) {
    // 非 Windows 平台不需要复制 DLL
}
