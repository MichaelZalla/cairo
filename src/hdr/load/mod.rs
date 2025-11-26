use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader, Read},
    mem,
    path::Path,
};

use crate::hdr::{HDR_FILE_PRELUDE, HdrRadianceFormat, HdrSource};

use super::{HDR_CHANNELS_PER_SAMPLE, Hdr, rgbe::Rgbe};

// Reference source:
// https://docs.rs/libhdr/1.0.0/src/libhdr/lib.rs.html

pub fn load_hdr(filepath: &Path) -> Result<Hdr, io::Error> {
    let file = match File::open(filepath) {
        Ok(file) => file,
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Unable to locate file '{}': {}", filepath.display(), e),
            ));
        }
    };

    let mut reader = BufReader::new(file);

    {
        let mut first_line = String::default();

        reader.read_line(&mut first_line)?;

        assert!(first_line.starts_with(HDR_FILE_PRELUDE));
    }

    let mut format = String::default();
    let mut headers = HashMap::new();

    let mut next_line = String::default();

    loop {
        next_line.clear();

        match reader.read_line(&mut next_line) {
            Ok(_) => {
                if next_line.trim().is_empty() {
                    break;
                }

                let mut r = next_line.split('=');

                match r.next() {
                    Some(key) => match r.next() {
                        Some(value) => {
                            let key_trimmed = key.trim().to_string();
                            let value_trimmed = value.trim().to_string();

                            if key_trimmed == "FORMAT" {
                                format.clone_from(&value_trimmed);
                            }

                            headers.insert(key_trimmed, value_trimmed);
                        }
                        None => continue,
                    },
                    None => continue,
                }
            }
            Err(_) => break,
        }
    }

    next_line.clear();

    let mut height: usize = 0;
    let mut width: usize = 0;

    let mut flip_vertical = false;
    let mut flip_horizontal = false;

    if reader.read_line(&mut next_line).is_ok() {
        let tokens: Vec<&str> = next_line.split(' ').map(|s| s.trim()).collect();

        // https://paulbourke.net/dataformats/pic/
        //
        // "The standard coordinate system for Radiance images would have the
        // following resolution string -Y N +X N. This indicates that the
        // vertical axis runs down the file and the X axis is to the right
        // (imagining the image as a rectangular block of data). A -X would
        // indicate a horizontal flip of the image. A +Y would indicate a
        // vertical flip".

        if tokens[0].starts_with('+') {
            flip_vertical = true;
        }

        height = tokens[1].parse::<usize>().unwrap();

        if tokens[2].starts_with('-') {
            flip_horizontal = true;
        }

        width = tokens[3].parse::<usize>().unwrap();
    }

    // @TODO Support '32-bit_rle_xyze' format?
    assert!(format == "32-bit_rle_rgbe");

    let hdr_source = HdrSource {
        filename: filepath.display().to_string(),
        radiance_format: HdrRadianceFormat::RleRgbe32,
        width,
        height,
        flip_horizontal,
        flip_vertical,
    };

    let bytes = decode_rle_bytes(width, height, &mut reader)?;

    assert!(bytes.len() == width * height * HDR_CHANNELS_PER_SAMPLE);

    Ok(Hdr {
        source: hdr_source,
        headers,
        bytes,
    })
}

fn decode_rle_bytes(
    width: usize,
    height: usize,
    reader: &mut BufReader<File>,
) -> Result<Vec<u8>, io::Error> {
    let mut output_buffer = vec![0; width * height * HDR_CHANNELS_PER_SAMPLE];

    let mut row_header = vec![0; 4];

    // Validate first row's header (if row headers are present).\

    // @TODO Handle case where rows data is not preceded by a rows header.
    {
        reader.read_exact(&mut row_header)?;

        let scanline_width = row_header[2] as usize;

        assert!(scanline_width << 8 | (row_header[3] as usize) == width);
    }

    let vec = vec![0; width * mem::size_of::<Rgbe>()];
    let mut row_buffer = vec;

    for row_index in 0..height {
        let mut run_header = vec![0; 2];

        if row_index != 0 {
            // Validate this row's header.
            reader.read_exact(&mut row_header)?;

            let scanline_width = row_header[2] as usize;

            assert!(scanline_width << 8 | (row_header[3] as usize) == width);
        }

        // Process each channel value.
        for channel_index in 0..HDR_CHANNELS_PER_SAMPLE {
            let channel_buffer_start = channel_index * width;
            let channel_buffer_end = (channel_index + 1) * width;
            let channel_buffer = &mut row_buffer[channel_buffer_start..channel_buffer_end];

            let mut cursor = 0;

            while cursor < channel_buffer.len() {
                reader.read_exact(&mut run_header)?;

                // Check if a run begins here.
                if run_header[0] > 128 {
                    // Run.

                    let run_length = (run_header[0] - 128) as usize;

                    assert!(run_length > 0);
                    assert!(run_length <= (channel_buffer.len() - cursor));

                    let run_buffer = &mut channel_buffer[cursor..cursor + run_length];

                    for channel_value in run_buffer.iter_mut() {
                        *channel_value = run_header[1];
                    }

                    cursor += run_length;
                } else {
                    // Not a run.

                    let mut count = run_header[0] as usize;

                    assert!(count > 0);
                    assert!(count <= (channel_buffer.len() - cursor));

                    channel_buffer[cursor] = run_header[1];

                    cursor += 1;
                    count -= 1;

                    if cursor > 0 {
                        let next = cursor + count;

                        let remainder_buffer = &mut channel_buffer[cursor..next];

                        reader.read_exact(remainder_buffer)?;

                        cursor = next;
                    }
                }
            }
        }

        // Copy the row buffer's data into the output buffer.

        let row_output = &mut output_buffer[(row_index * width * HDR_CHANNELS_PER_SAMPLE)
            ..((row_index + 1) * width * HDR_CHANNELS_PER_SAMPLE)];

        let row_output_rgbe = row_output.chunks_mut(4);

        let red = row_buffer[0..width].iter();
        let green = row_buffer[width..width * 2].iter();
        let blue = row_buffer[width * 2..width * 3].iter();
        let e = row_buffer[width * 3..].iter();

        for ((((row_output_rgbe, r), g), b), e) in
            row_output_rgbe.zip(red).zip(green).zip(blue).zip(e)
        {
            row_output_rgbe[0] = *r;
            row_output_rgbe[1] = *g;
            row_output_rgbe[2] = *b;
            row_output_rgbe[3] = *e;
        }
    }

    Ok(output_buffer)
}
