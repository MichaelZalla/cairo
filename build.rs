use std::process::Command;

fn main() {
    #[cfg(target_os = "windows")]
    let short_hash_output = Command::new("cmd")
        .args(["/C", "git rev-parse --short HEAD"])
        .output()
        .expect("Failed to execute process");

    #[cfg(target_os = "macos")]
    let short_hash_output = Command::new("/bin/bash")
        .args(["git rev-parse --short HEAD"])
        .output()
        .expect("Failed to execute process");

    let short_hash = String::from_utf8(short_hash_output.stdout).unwrap();

    println!("cargo:rustc-env=GIT_COMMIT_SHORT_HASH={}", short_hash);

    #[cfg(target_os = "windows")]
    {
        println!(r"cargo:rustc-link-search=%SDL_PATH%\SDL2-2.28.5\lib\x64\");
        println!(r"cargo:rustc-link-search=%SDL_PATH%\SDL2_ttf-2.20.2\lib\x64\");
        println!(r"cargo:rustc-link-search=%SDL_PATH%\SDL2_image-2.8.0\lib\x64\");
    }
}
