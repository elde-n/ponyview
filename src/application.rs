use crate::commands;
use crate::constants::*;
use crate::thumbnails;
use crate::window::Window;
use crate::paintable::SmartPaintable;


use clap;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::{gdk, glib};
use gtk::prelude::*;

use std::path::PathBuf;


pub struct Application {
    window: &'static Window,
}

impl Application {
    pub fn new() -> glib::ExitCode {
        let app = gtk::Application::builder()
            .application_id(APP_ID)
            .build();

        let application = Self {
            window: Box::leak(Box::new(Window::new()))
        };

        app.connect_activate(move |app| application.on_activate(app));
        app.connect_startup(|_| load_css());

        let empty: Vec<String> = vec![];
        app.run_with_args(&empty)
    }

    fn on_activate(&self, application: &gtk::Application) {
        let mut command = commands::commands();
        let options = command.clone().get_matches();

        let mut active: Vec<PathBuf> = Vec::new();
        active.extend(options.get_many::<PathBuf>("files")
            .unwrap_or_default()
            .map(|f| f.to_path_buf()));

        while active.len() > 0 {
            for file in active.pop() {
                if file.is_dir() {
                    if options.get_flag("recursive") {
                        active.extend(file.read_dir().unwrap().map(|e| e.unwrap().path()));
                    }
                } else {
                    self.window.files.lock().unwrap().push(file.to_path_buf());
                }
            }
        }

        if options.get_flag("stdin") {
            if options.get_many::<PathBuf>("files").unwrap_or_default().len() == 0 {
                command.print_help();
                std::process::exit(1)
            }

            // TODO: parse files from stdin
        }

        self.window.is_thumbnail_mode.set(options.get_flag("thumbnail"));
        self.window.construct();

        self.window.bar.get().unwrap().set_visible(!options.get_flag("no-bar"));

        if options.get_flag("fullscreen") && !self.window.is_fullscreen() {
            self.window.toggle_fullscreen();
        }

        for file in self.window.files.lock().unwrap().iter() {
            let paintable = thumbnails::from_cache(file);
            let paintable = if !paintable.is_some() {
                thumbnails::to_cache(file);
                thumbnails::from_cache(file).unwrap()
            } else { paintable.unwrap() };

            let frame = gtk::Box::builder()
                .halign(gtk::Align::Start)
                .hexpand(true)
                .width_request(paintable.intrinsic_width())
                .height_request(paintable.intrinsic_height())
                .build();

            let thumbnail = gtk::Picture::builder()
                .paintable(&paintable)
                .build();

            frame.append(&thumbnail);
            self.window.thumbnail_grid.get().unwrap().append(&frame);
            self.window.thumbnails.lock().unwrap().push(paintable);
        }

        if self.window.files.lock().unwrap().len() > 0 {
            self.window.set_focused_image(self.window.cursor.get());
            self.window.set_highlighted_image(self.window.cursor.get());
        }

        let controller = gtk::EventControllerKey::new();
        let _connect_key_pressed = controller.connect_key_pressed(glib::clone!(@strong self.window as window => move |_, key, _state, modifier| {
            match key {
                gdk::Key::q => {
                    window.quit(options.get_flag("stdout"));
                }

                gdk::Key::h => {
                    if window.is_thumbnail_mode.get() {
                        let cursor = window.cursor.get();
                        if cursor > 1 { window.set_highlighted_image(cursor - 1) }
                    } else { }

                    window.cursor_changed();
                }

                gdk::Key::j => {
                    if window.is_thumbnail_mode.get() {
                        let binding = window.files.lock().unwrap();
                        let length = binding.len();
                        std::mem::drop(binding);

                        let cursor = window.cursor.get();
                        let stride = flow_box_get_stride(window.thumbnail_grid.get().unwrap(), length);

                        if length >= cursor + stride { window.set_highlighted_image(cursor + stride) }
                    } else { }

                    window.cursor_changed();
                }

                gdk::Key::k => {
                    if window.is_thumbnail_mode.get() {
                        let guard = window.files.lock().unwrap();
                        let length = guard.len();
                        std::mem::drop(guard);

                        let cursor = window.cursor.get();
                        let stride = flow_box_get_stride(window.thumbnail_grid.get().unwrap(), length);

                        if cursor > stride { window.set_highlighted_image(cursor - stride) }
                    } else { }

                    window.cursor_changed();
                }

                gdk::Key::l => {
                    if window.is_thumbnail_mode.get() {
                        let cursor = window.cursor.get();
                        if window.files.lock().unwrap().len() > cursor {
                            window.set_highlighted_image(cursor + 1);
                        }
                    } else { }

                    window.cursor_changed();
                }

                gdk::Key::g => {
                    window.cursor.set(1);
                    window.cursor_changed();
                }
                gdk::Key::G => {
                    let guard = window.files.lock().unwrap();
                    let length = guard.len();
                    std::mem::drop(guard);

                    window.cursor.set(length);
                    window.cursor_changed();
                }

                gdk::Key::n => {
                    let cursor = window.cursor.get();
                    if window.files.lock().unwrap().len() > cursor {
                        window.cursor.set(cursor + 1);
                        window.cursor_changed();
                    }
                }

                gdk::Key::p => {
                    let cursor = window.cursor.get();
                    if cursor > 1 {
                        window.cursor.set(cursor - 1);
                        window.cursor_changed();
                    }
                }

                gdk::Key::m => { window.mark_image(window.cursor.get()) }

                gdk::Key::r => {
                    window.image.get().unwrap().set_file(window.image.get().unwrap().file().as_ref());
                }

                gdk::Key::R => {
                    // TODO: reload all thumbnails in thumbnail mode
                }

                gdk::Key::D => {
                    if window.cursor.get() > 0 {
                        let mut guard = window.files.lock().unwrap();
                        guard.remove(0);
                        std::mem::drop(guard);

                        window.cursor.set(window.cursor.get() - 1);
                        if window.cursor.get() > 0 { window.cursor_changed() }
                    }
                }

                gdk::Key::f => { window.toggle_fullscreen() }
                gdk::Key::b => {
                    window.bar.get().unwrap().set_visible(!window.bar.get().unwrap().get_visible());
                }

                gdk::Key::a => {
                    if !window.is_thumbnail_mode.get() {
                        if modifier.contains(gdk::ModifierType::CONTROL_MASK) {
                            let paintable = window.focused_image.borrow();
                            if paintable.is_playing() { paintable.pause() } else { paintable.resume() }
                        } else {
                            let mut font_options = gtk::cairo::FontOptions::new().unwrap();
                            font_options.set_antialias(gtk::cairo::Antialias::Best);
                            window.image.get().unwrap().set_font_options(Some(&font_options));
                        }
                    }
                }

                gdk::Key::A => {
                }

                gdk::Key::vertbar => { window.flip_focused_image(true) }
                gdk::Key::underscore => { window.flip_focused_image(false) }
                gdk::Key::less => { window.rotate_focused_image(-90) }
                gdk::Key::greater => { window.rotate_focused_image(90) }

                gdk::Key::equal => {
                }
                gdk::Key::minus => {
                }
                gdk::Key::plus => {
                }
                gdk::Key::z => {
                }

                gdk::Key::colon => {
                    // TODO cmdbar
                }

                gdk::Key::Return | gdk::Key::space => {
                    if !modifier.contains(gdk::ModifierType::CONTROL_MASK) {
                        window.is_thumbnail_mode.set(!window.is_thumbnail_mode.get());
                        window.focused_view.get().unwrap().set_visible(!window.is_thumbnail_mode.get());
                        window.thumbnail_view.get().unwrap().set_visible(window.is_thumbnail_mode.get());

                        window.cursor_changed();
                    } else if key == gdk::Key::space && !window.is_thumbnail_mode.get() {
                        let paintable = window.focused_image.borrow();
                        if paintable.is_playing() { paintable.pause() } else { paintable.resume() }
                    }
                }

                _ => {}
            };

            glib::Propagation::Proceed
        }));

        self.window.gtk_window.get().unwrap().add_css_class("window");
        self.window.gtk_window.get().unwrap().add_controller(controller);
        self.window.gtk_window.get().unwrap().set_application(Some(application));
        self.window.gtk_window.get().unwrap().present();
    }
}

fn flow_box_get_stride(flow_box: &gtk::FlowBox, length: usize) -> usize {
    if length < 1 { return 0 }

    let child = flow_box.child_at_index(0);
    let y = child.unwrap().allocation().y();

    let mut index = 1;
    for i in 1..length {
        let child = flow_box.child_at_index(i as i32);
        let cy = child.unwrap().allocation().y();

        index = i;
        if y != cy { break }
    }

    index
}


fn load_css() {
    // Load the CSS file and add it to the provider
    let provider = gtk::CssProvider::new();
    provider.load_from_string(include_str!("style/style.css"));

    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}
