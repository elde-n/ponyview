mod imp;
mod frame;

use std::{io::Cursor, path::PathBuf};

use gtk::{gdk, gio, glib, prelude::*, subclass::prelude::*};
use image::{codecs::gif::GifDecoder, AnimationDecoder, ImageReader};
use frame::Frame;


glib::wrapper! {
    pub struct SmartPaintable(ObjectSubclass<imp::SmartPaintable>) @implements gdk::Paintable;
}

impl Default for SmartPaintable {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl SmartPaintable {
    /// The loading consists of decoding the gif with a GIFDecoder, then storing
    /// the frames so that the paintable can render them.

    pub fn load_from_bytes(&self, bytes: &[u8], extension: &str) -> Result<(), Box<dyn std::error::Error>> {
        let imp = self.imp();
        imp.current_idx.set(0);

        if let Some(source_id) = imp.timeout_source_id.take() { source_id.remove() }

        let is_animated = extension == "gif";

        let read = Cursor::new(bytes);
        if is_animated {
            let decoder = GifDecoder::new(read)?;

            let frames = decoder
                .into_frames()
                .collect_frames()?
                .into_iter()
                .map(Frame::from)
                .collect::<Vec<Frame>>();

            imp.frames.replace(Some(frames));
        } else {
            let mut reader = ImageReader::new(read);
            reader.set_format(
                image::ImageFormat::from_extension(extension)
                .expect("Failed to guess image format from extension"));
            let frame = image::Frame::new(reader
                .decode()
                .expect("Failed to parse image")
                .into_rgba8());

            imp.frames.replace(Some(vec![Frame::from(frame)]));
        }

        self.setup_next_frame();
        Ok(())
    }

    pub fn load_from_file(&self, file: &std::path::Path) -> Result<(), Box<dyn std::error::Error>>  {
        let (bytes, _) = gio::File::for_path(file).load_contents(gio::Cancellable::NONE).unwrap();
        self.load_from_bytes(&bytes, file.extension().unwrap_or_default().to_str().unwrap())
    }

    fn setup_next_frame(&self) {
        let imp = self.imp();
        let idx = imp.current_idx.get();
        let frames_ref = imp.frames.borrow();

        if !imp.is_playing.get() && idx > 0 { return }
        if !imp.timeout_source_id.borrow().is_none() { return }

        // if we have stored no frames then we early return early
        // and instead render a default frame in `imp::SmartPaintable::snapshot`
        let frames = match &*frames_ref {
            Some(frames) => frames,
            None => return,
        };


        let next_frame = frames.get(idx).unwrap();
        imp.next_frame.replace(Some(next_frame.texture.clone()));

        // invalidate the contents so that the new frame will be rendered
        self.invalidate_contents();

        let update_next_frame_callback = glib::clone!(
            @weak self as paintable => move || {
                paintable.imp().timeout_source_id.take();
                paintable.setup_next_frame();
            }
        );

        let source_id = glib::timeout_add_local_once(
            next_frame.frame_duration, update_next_frame_callback);
        imp.timeout_source_id.replace(Some(source_id));

        // setup the index for the next call to setup_next_frame
        let mut new_idx = idx + 1;
        if new_idx >= frames.len() { new_idx = 0 }
        imp.current_idx.set(new_idx);
    }

    pub fn is_playing(&self) -> bool {
        return self.imp().is_playing.get()
    }

    pub fn resume(&self) {
        self.imp().is_playing.set(true);
        self.setup_next_frame();
    }

    pub fn pause(&self) {
        self.imp().is_playing.set(false);
    }

    pub fn restart(&self) {
        self.imp().current_idx.set(0);
    }

    pub fn rotate(&self, angle: i32) {
        self.imp().rotation.set(angle);
    }

    pub fn flip(&self, is_horizontal: bool) { }
}
