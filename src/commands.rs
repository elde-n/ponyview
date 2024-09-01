use clap::Command;

pub fn commands() -> Command {
    clap::command!()
        .args(&[
            clap::arg!([files] ... "Files")
                .value_parser(clap::value_parser!(std::path::PathBuf)),
            clap::arg!(-c --config <FILE> "Sets a custom config file")
                .required(false)
                .value_parser(clap::value_parser!(std::path::PathBuf)),
            clap::arg!(-b --"no-bar" "Start with the statusbar hidden"),
            clap::arg!(-f --fullscreen "Start in fullscreen mode"),
            clap::arg!(-q --quiet "Quiet."),
            clap::arg!(-i --stdin "Read names of files to open from standard input"),
            clap::arg!(-o --stdout "Write list of all marked files to standard output on quit"),
            clap::arg!(-r --recursive "Search for images in a directory recursively"), // SCARY
            clap::arg!(-t --thumbnail "Start in thumbnail mode"),
            clap::arg!(-v --version "Print version information to standard output and exit"),
            clap::arg!(-z --zoom "Set the zoom level percentage")])
        .disable_version_flag(true)
}
