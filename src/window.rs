use crate::constants::*;
use crate::paintable::SmartPaintable;

use std::{cell::{Cell, OnceCell, RefCell}, path::PathBuf, rc::Rc, sync::Mutex};

use gtk::{prelude::*, Adjustment};


pub struct Window {
    pub zoom: Cell<usize>,
    pub cursor: Cell<usize>,
    pub rotation: Cell<i32>,
    pub is_thumbnail_mode: Cell<bool>,

    pub files: Rc<Mutex<Vec<PathBuf>>>,
    pub thumbnails: Rc<Mutex<Vec<SmartPaintable>>>,

    pub focused_image: RefCell<SmartPaintable>,

    pub gtk_window: OnceCell<gtk::ApplicationWindow>,

    pub focused_view: OnceCell<gtk::Box>,
    pub image: OnceCell<gtk::Picture>,

    pub bar: OnceCell<gtk::Box>,
    bar_text_left: OnceCell<gtk::Label>,
    bar_text_right: OnceCell<gtk::Label>,

    pub thumbnail_view: OnceCell<gtk::Box>,
    pub thumbnail_scroller: OnceCell<gtk::ScrolledWindow>,
    pub thumbnail_grid: OnceCell<gtk::FlowBox>,
    previous_highlighted_image: Cell<usize>
}

impl Window {
    pub fn new() -> Self {
        Self {
            zoom: Cell::new(100),
            cursor: Cell::new(1),
            rotation: Cell::new(0),
            is_thumbnail_mode: Cell::new(false),

            files: Rc::new(Mutex::new(Vec::new())),
            thumbnails: Rc::new(Mutex::new(Vec::new())),
            focused_image: RefCell::new(SmartPaintable::default()),

            gtk_window: OnceCell::new(),

            focused_view: OnceCell::new(),
            image: OnceCell::new(),

            bar: OnceCell::new(),
            bar_text_left: OnceCell::new(),
            bar_text_right: OnceCell::new(),

            thumbnail_view: OnceCell::new(),
            thumbnail_scroller: OnceCell::new(),
            thumbnail_grid: OnceCell::new(),
            previous_highlighted_image: Cell::new(1)
        }
    }

    pub fn construct(&self) {
       let content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();

        self.gtk_window.set(gtk::ApplicationWindow::builder()
            .title("ponyview")
            .child(&content)
            .width_request(WINDOW_WIDTH as i32)
            .height_request(WINDOW_HEIGHT as i32)
            .build()).unwrap();

        self.image.set(gtk::Picture::builder()
            .hexpand(true)
            .vexpand(true)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Fill)
            .can_shrink(true)
            .build()).unwrap();

        self.focused_view.set(gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .visible(!self.is_thumbnail_mode.get())
            .build()).unwrap();
        self.focused_view.get().unwrap().append(self.image.get().unwrap());

        self.bar.set(gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .height_request(15)
            .build()).unwrap();

        self.bar_text_left.set(gtk::Label::builder()
            .build()).unwrap();

        self.bar_text_right.set(gtk::Label::builder()
            .halign(gtk::Align::End)
            .hexpand(true)
            .build()).unwrap();

        self.bar.get().unwrap().add_css_class("bar");
        self.bar.get().unwrap().append(self.bar_text_left.get().unwrap());
        self.bar.get().unwrap().append(self.bar_text_right.get().unwrap());

        self.thumbnail_grid.set(gtk::FlowBox::builder()
            .row_spacing(20)
            .column_spacing(THUMBNAIL_CHILD_PADDING as u32)
            .max_children_per_line(100)
            .can_focus(false)
            .homogeneous(true)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            .selection_mode(gtk::SelectionMode::None)
            .build()).unwrap();

        self.thumbnail_scroller.set(gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            // TODO: we can't disable it but maybe we can hide it?
            .vscrollbar_policy(gtk::PolicyType::Never)
            .child(self.thumbnail_grid.get().unwrap())
            .build()).unwrap();

        self.thumbnail_view.set(gtk::Box::builder()
            .hexpand(true)
            .vexpand(true)
            .visible(self.is_thumbnail_mode.get())
            .build()).unwrap();

        self.thumbnail_view.get().unwrap().append(self.thumbnail_scroller.get().unwrap());

        content.append(self.bar.get().unwrap());
        content.append(self.focused_view.get().unwrap());
        content.append(self.thumbnail_view.get().unwrap());
    }

    pub fn marked_files(&self) -> Vec<usize> {
        let guard = self.files.lock().unwrap();
        let file_count = guard.len();
        let mut marked_files: Vec<usize> = Vec::new();

        for i in 0 .. file_count - 1 {
            let child = self.thumbnail_grid.get().unwrap().child_at_index((i) as i32)
                .unwrap()
                .child()
                .unwrap();

            if child.has_css_class("is-marked") { marked_files.push(i + 1) }
        }

        std::mem::drop(guard);

        marked_files
    }

    pub fn is_fullscreen(&self) -> bool {
        self.gtk_window.get().unwrap().is_fullscreen()
    }

    pub fn set_focused_image(&self, cursor: usize) {
        let focused_image = self.load_image(cursor);

        self.image.get().unwrap().set_paintable(Some(&focused_image));
        self.cursor.set(cursor);

        self.focused_image.replace(focused_image);
        self.update_bar(&self.focused_image.borrow());
    }

    pub fn set_highlighted_image(&self, cursor: usize) {
        self.cursor.set(cursor);

        let child = self.thumbnail_grid.get().unwrap().child_at_index((cursor - 1) as i32)
            .unwrap()
            .child()
            .unwrap();

        let previous_child = self.thumbnail_grid.get().unwrap()
            .child_at_index((self.previous_highlighted_image.get() - 1) as i32)
            .unwrap()
            .child()
            .unwrap();

        // TODO: make border not jiggle the listbox
        previous_child.remove_css_class("highlighted-thumbnail");
        child.add_css_class("highlighted-thumbnail");

        let guard = self.thumbnails.lock().unwrap();
        self.previous_highlighted_image.set(cursor);
        self.update_bar(guard.get(cursor - 1).unwrap());
    }

    pub fn load_image(&self, cursor: usize) -> SmartPaintable {
        let guard = self.files.lock().unwrap();
        let paintable = SmartPaintable::default();

        paintable.load_from_file(guard.get(cursor - 1).unwrap());
        paintable
    }

    pub fn mark_image(&self, cursor: usize) {
        let child = self.thumbnail_grid.get().unwrap().child_at_index((cursor - 1) as i32)
            .unwrap()
            .child()
            .unwrap();

        // TODO: use properties instead of a css class
        // TODO: add a visual indicator for marked files
        if child.has_css_class("is-marked") {
            child.remove_css_class("is-marked");
        } else {
            child.add_css_class("is-marked");
        }
    }

    pub fn rotate_focused_image(&self, angle: i32) {
        if self.is_thumbnail_mode.get() { return }
        self.rotation.set(self.rotation.get() + angle);
        self.focused_image.borrow().rotate(self.rotation.get());
    }

    pub fn flip_focused_image(&self, is_horizontal: bool) {
        if self.is_thumbnail_mode.get() { return }
        self.focused_image.borrow().flip(is_horizontal);
    }

    pub fn toggle_fullscreen(&self) {
        let window = self.gtk_window.get().unwrap();
        if window.is_fullscreen() { window.unfullscreen() } else { window.fullscreen() }
    }

    pub fn cursor_changed(&self) {
        if self.is_thumbnail_mode.get() {
            self.set_highlighted_image(self.cursor.get());

            let scroll = self.thumbnail_scroller.get().unwrap();
            let adjustment = scroll.vadjustment();

            let child = self.thumbnail_grid.get().unwrap().child_at_index((self.cursor.get() - 1) as i32)
                .unwrap();

            adjustment.set_value(child.allocation().y() as f64);
            scroll.set_vadjustment(Some(&adjustment));
        } else {
            self.set_focused_image(self.cursor.get());
        }
    }

    fn update_bar(&self, image: &SmartPaintable) {
        let guard = self.files.lock().unwrap();
        let file_count = guard.len();

        let file = guard.get(self.cursor.get() - 1).unwrap();
        let file_name = file.file_name().unwrap().to_str().unwrap();
        let metadata = std::fs::File::open(file).unwrap().metadata().unwrap();

        self.bar_text_left.get().unwrap().set_text(
            (format!("  {}\t{}x{}\t{}",
                format_byte_unit(metadata.len()),
                image.intrinsic_width(),
                image.intrinsic_height(),
                file_name)).as_str());

        self.bar_text_right.get().unwrap().set_text(
            (format!("{}%\t{}/{}  ",
                self.zoom.get(),
                self.cursor.get(),
                file_count)).as_str());
    }

    pub fn quit(&self, stdout: bool) {
        if stdout {
            let marked_files = self.marked_files();

            let guard = self.files.lock().unwrap();
            for i in marked_files {
                let file = guard.get(i).unwrap().to_str().unwrap();
                println!("{file}");
            }
        }

        std::process::exit(0);
    }
}

// TODO: rewrite this
fn format_byte_unit(number: u64) -> String {
    let e = 1024 as u64;
    if number / e > 0 {
        if (number / e) / e > 0 {
            return ((number / e) / e).to_string() + "MB"
        }

        return (number / e).to_string() + "KB"
    }

    number.to_string() + "B"
}
