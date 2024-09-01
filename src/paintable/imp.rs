use std::cell::{Cell, RefCell};

use gtk::{gdk, glib, graphene, prelude::*, subclass::prelude::*};

use super::Frame;

#[derive(Default)]
pub struct SmartPaintable {
    pub frames: RefCell<Option<Vec<Frame>>>,
    pub next_frame: RefCell<Option<gdk::Texture>>,
    pub timeout_source_id: RefCell<Option<glib::SourceId>>,
    pub current_idx: Cell<usize>,

    pub rotation: Cell<i32>,
    pub is_playing: Cell<bool>
}

#[glib::object_subclass]
impl ObjectSubclass for SmartPaintable {
    const NAME: &'static str = "SmartPaintable";
    type Type = super::SmartPaintable;
    type Interfaces = (gdk::Paintable,);
}

impl ObjectImpl for SmartPaintable {}

impl PaintableImpl for SmartPaintable {
    fn intrinsic_height(&self) -> i32 {
        self.next_frame
            .borrow()
            .as_ref()
            .map(|texture| texture.height())
            .unwrap_or(-1)
    }

    fn intrinsic_width(&self) -> i32 {
        self.next_frame
            .borrow()
            .as_ref()
            .map(|texture| texture.width())
            .unwrap_or(-1)
    }

    fn snapshot(&self, snapshot: &gdk::Snapshot, width: f64, height: f64) {
        if let Some(texture) = &*self.next_frame.borrow() {
            let (cx, cy) = (width as f32 / 2.0, height as f32 / 2.0);

            snapshot.translate(&graphene::Point::new(cx, cy));
            snapshot.rotate(self.rotation.get() as f32);
            snapshot.translate(&graphene::Point::new(-cx, -cy));

            texture.snapshot(snapshot, width, height);
        } else {
            snapshot.append_color(
                &gdk::RGBA::BLACK,
                &graphene::Rect::new(0f32, 0f32, width as f32, height as f32)
            );
        }
    }
}
