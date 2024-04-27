use std::path::Path;

use cairo::{buffer::Buffer2D, hdr::load::load_hdr, vec::vec3::Vec3};

fn main() -> Result<(), String> {
    let filepath = Path::new("./examples/ibl/assets/rural_asphalt_road_4k.hdr");

    match load_hdr(filepath) {
        Ok(hdr) => {
            println!("{:?}", hdr.source);
            println!("{:?}", hdr.headers);
            println!("Decoded {} bytes from file.", hdr.bytes.len());

            let vecs = hdr.to_vec3();

            let buffer = Buffer2D::<Vec3>::from_data(
                hdr.source.width as u32,
                hdr.source.height as u32,
                vecs,
            );

            println!("{}x{}", buffer.width, buffer.height);
        }
        Err(e) => {
            return Err(format!("Failed to read HDR file: {}", e).to_string());
        }
    }

    Ok(())
}
