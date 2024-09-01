pub mod window;
pub mod commands;
pub mod constants;
pub mod paintable;
pub mod thumbnails;
pub mod application;


use gtk::glib;


fn main() -> glib::ExitCode {
    application::Application::new()
}
