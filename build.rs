use std::env::var;

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
    }
}
