#![windows_subsystem = "windows"]

use native_windows_gui as nwg;

use log::info;

mod client;
mod config;
mod dialog;
mod utils;


fn main() {
    let mut p = utils::exe_path();
    p.push("russ.log");

    utils::init_log(p);

    info!("russ starts up");

    nwg::init().expect("Failed to init Native Windows GUI");
    let mut font = nwg::Font::default();

    nwg::Font::builder()
        .family("Segoe UI")
        .size(16)
        .build(&mut font)
        .expect("Failed to build font");

    nwg::Font::set_global_default(Some(font));

    dialog::open();
}

