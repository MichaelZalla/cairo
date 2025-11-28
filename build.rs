use std::env::var;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    #[cfg(target_os = "windows")]
    let short_hash_output = Command::new("cmd")
        .args(["/C", "git rev-parse --short HEAD"])
        .output()
        .expect("Failed to execute process");

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    let short_hash_output = Command::new("/bin/bash")
        .args(["git rev-parse --short HEAD"])
        .output()
        .expect("Failed to execute process");

    let short_hash = String::from_utf8(short_hash_output.stdout).unwrap();

    println!("cargo:rustc-env=GIT_COMMIT_SHORT_HASH={}", short_hash);

    #[cfg(target_os = "windows")]
    {
        let sdl_path = var("SDL_PATH").unwrap();

        println!(r"cargo:rustc-link-search={}\SDL2-2.28.5\lib\x64\", sdl_path);

        println!(
            r"cargo:rustc-link-search={}\SDL2_ttf-2.20.2\lib\x64\",
            sdl_path
        );

        println!(
            r"cargo:rustc-link-search={}\SDL2_image-2.8.0\lib\x64\",
            sdl_path
        );

        // Copy SDL2 DLLs to target directories so executables can find them
        let dll_sources = [
            (
                format!(r"{}\SDL2-2.28.5\lib\x64\SDL2.dll", sdl_path),
                "SDL2.dll",
            ),
            (
                format!(r"{}\SDL2_ttf-2.20.2\lib\x64\SDL2_ttf.dll", sdl_path),
                "SDL2_ttf.dll",
            ),
            (
                format!(r"{}\SDL2_image-2.8.0\lib\x64\SDL2_image.dll", sdl_path),
                "SDL2_image.dll",
            ),
        ];

        let target_dirs = ["target/debug", "target/release"];

        for target_dir in &target_dirs {
            // Create target directory if it doesn't exist
            let target_path = Path::new(target_dir);

            if !target_path.exists() {
                fs::create_dir_all(target_path).ok();
            }

            // Copy each DLL to the target directory
            for (source, dll_name) in &dll_sources {
                let dest = target_path.join(dll_name);

                // Only copy if source exists and dest doesn't exist or is outdated
                if Path::new(source).exists() {
                    let should_copy = !dest.exists() || {
                        let src_metadata = fs::metadata(source).ok();
                        let dst_metadata = fs::metadata(&dest).ok();

                        match (src_metadata, dst_metadata) {
                            (Some(src), Some(dst)) => src.modified().ok() > dst.modified().ok(),
                            _ => true,
                        }
                    };

                    if should_copy {
                        fs::copy(source, &dest).ok();
                    }
                }
            }
        }
    }
}
