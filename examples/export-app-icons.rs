//! Export the runtime Spotiurge mark for native packages, without another renderer.
//!
//! `cargo run --locked --no-default-features --example export-app-icons -- NEW_DIRECTORY`
use image::ImageEncoder;
use std::{collections::BTreeMap, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let directory = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("Pass a new output directory")?,
    );
    if directory.exists() {
        return Err("Choose a new output directory; existing icons are preserved".into());
    }
    fs::create_dir_all(&directory)?;
    let mut pngs = BTreeMap::new();
    for size in [16usize, 24, 32, 48, 64, 128, 256, 512, 1024] {
        let pixels = spotifast::util::app_icon_rgba(size);
        let mut png = Vec::new();
        image::codecs::png::PngEncoder::new(&mut png).write_image(
            &pixels,
            size as u32,
            size as u32,
            image::ExtendedColorType::Rgba8,
        )?;
        fs::write(directory.join(format!("icon-{size}.png")), &png)?;
        pngs.insert(size, png);
    }

    // Modern macOS reads PNG members. Include both base and Retina slots.
    let slots: [(&[u8; 4], usize); 11] = [
        (b"icp4", 16),
        (b"icp5", 32),
        (b"icp6", 64),
        (b"ic07", 128),
        (b"ic08", 256),
        (b"ic09", 512),
        (b"ic10", 1024),
        (b"ic11", 32),
        (b"ic12", 64),
        (b"ic13", 256),
        (b"ic14", 512),
    ];
    let mut members = Vec::new();
    for (kind, size) in slots {
        let png = &pngs[&size];
        members.extend_from_slice(kind);
        members.extend_from_slice(&u32::try_from(png.len() + 8)?.to_be_bytes());
        members.extend_from_slice(png);
    }
    let mut icns = b"icns".to_vec();
    icns.extend_from_slice(&u32::try_from(members.len() + 8)?.to_be_bytes());
    icns.extend_from_slice(&members);
    fs::write(directory.join("spotiurge.icns"), icns)?;

    // Windows Vista and later support PNG entries in an ICO container.
    let sizes = [16usize, 24, 32, 48, 64, 128, 256];
    let mut ico = vec![0, 0, 1, 0];
    ico.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
    let mut offset = u32::try_from(6 + sizes.len() * 16)?;
    for size in sizes {
        let png = &pngs[&size];
        let dimension = if size == 256 { 0 } else { size as u8 };
        ico.extend_from_slice(&[dimension, dimension, 0, 0]);
        ico.extend_from_slice(&1u16.to_le_bytes());
        ico.extend_from_slice(&32u16.to_le_bytes());
        let length = u32::try_from(png.len())?;
        ico.extend_from_slice(&length.to_le_bytes());
        ico.extend_from_slice(&offset.to_le_bytes());
        offset += length;
    }
    for size in sizes {
        ico.extend_from_slice(&pngs[&size]);
    }
    fs::write(directory.join("spotiurge.ico"), ico)?;
    println!("{}", directory.display());
    Ok(())
}
