use std::path::Path;

use cairo::hdr::load::load_hdr;

fn main() -> Result<(), String> {
    let filepath = Path::new("./examples/ibl/assets/rural_asphalt_road_4k.hdr");

    match load_hdr(filepath) {
        Ok(hdr) => {
            println!("{:?}", hdr.source);
            println!("{:?}", hdr.headers);
            println!("Decoded {} bytes from file.", hdr.bytes.len());

            let hdr_texture = hdr.to_texture_map();

            println!("{}x{}", hdr_texture.width, hdr_texture.height);
        }
        Err(e) => {
            return Err(format!("Failed to read HDR file: {}", e).to_string());
        }
    }

    Ok(())
}
