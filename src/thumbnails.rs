use crate::constants::*;
use crate::paintable::SmartPaintable;

use std::io::Cursor;
use gtk::{gio, prelude::*, subclass::prelude::*};
use image::{codecs::gif::GifDecoder, AnimationDecoder, DynamicImage, ImageReader};


fn cache_path() -> std::path::PathBuf {
    let path = 'path: {
        let path = std::env::var("XDG_CACHE_HOME");
        if path.is_ok() { break 'path std::path::PathBuf::from(&path.unwrap()) }

        let path = std::env::home_dir().unwrap().join(".cache");
        path
    }.join("ponyview");

    std::fs::create_dir_all(path.clone());
    return path
}

fn find_file_without_extension(file: &std::path::Path) -> Option<std::path::PathBuf> {
    for child in file.parent().unwrap().read_dir().unwrap() {
        let file_name = file.file_name().unwrap().to_str().unwrap();

        let binding = child.unwrap();
        if binding.file_name().to_str().unwrap().starts_with(file_name) {
            return Some(binding.path())
        }
    }

    return None
}

fn calculate_size(f_width: u32, f_height: u32) -> (f64, f64) {
    let aspect_ratio = (f_width as f64 / f_height as f64);
    let mut width = THUMBNAIL_CHILD_SIZE as f64;
    let mut height = width * aspect_ratio;

    if height > width {
        height = THUMBNAIL_CHILD_SIZE as f64;
        width = height * aspect_ratio;
    }

    (width, height)
}

pub fn to_cache(file: &std::path::Path) {
    let hash = blake3::hash(file.as_os_str().to_str().unwrap().as_bytes())
        .to_hex()
        .to_lowercase();

    // TODO: make the thumbnail save options a config option
    let format = image::ImageFormat::Avif;
    let extension = *(format.extensions_str().first().unwrap());

    // TODO: add support for all animated images (not just yiphs)
    let is_animated = extension == "gif";
    let (bytes, _) = gio::File::for_path(file).load_contents(gio::Cancellable::NONE).unwrap();

    let binding = cache_path().join(hash);
    let cache_file = binding.as_path().with_extension(extension);

    let read = Cursor::new(bytes);
    if is_animated {
        let decoder = GifDecoder::new(read).unwrap();
        let mut frames = decoder.into_frames();
        let frame = frames.next().unwrap().unwrap();
        let (width, height) = calculate_size(frame.buffer().width(), frame.buffer().height());

        let frames = frames
            .map(|f| DynamicImage::from(f.unwrap().into_buffer()))
            .map(|image| {
                image.resize(width as u32, height as u32, image::imageops::FilterType::Gaussian);
            });
    } else {
        let mut reader = ImageReader::new(read);
        reader.set_format(
            image::ImageFormat::from_extension(extension)
            .expect("Failed to guess image format from extension"));

        let image = reader
            .decode()
            .expect("Failed to parse image");

        let (width, height) = calculate_size(image.width(), image.height());
        image.resize(width as u32, height as u32, image::imageops::FilterType::Gaussian)
            .save_with_format(cache_file, format)
            .expect("Failed to cache image");
    }
}

pub fn from_cache(file: &std::path::Path) -> Option<SmartPaintable> {
    let hash = blake3::hash(file.as_os_str().to_str().unwrap().as_bytes())
        .to_hex()
        .to_lowercase();

    let binding = cache_path().join(hash);
    let cache_file = binding.as_path();

    let binding = find_file_without_extension(cache_file);
    if binding.is_none() { return None }

    let binding = binding.unwrap();

    let cache_file = binding.as_path();
    if !cache_file.exists() { return None }

    let paintable = SmartPaintable::default();
    paintable.load_from_file(cache_file);
    Some(paintable)
}

pub fn clear_cache() {

}
