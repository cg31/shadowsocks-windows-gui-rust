#![windows_subsystem = "windows"]

use native_windows_gui as nwg;

use log::info;
use log::error;

mod client;
mod config;
mod dialog;
mod utils;


fn main() {
    let mut p = utils::exe_path();
    p.push("russ.log");

    let _ = utils::init_log(p);

    info!("russ starts up");

    if let Err(_) = nwg::init() {
        error!("Failed to init Native Windows GUI");
        return;
    }

    let mut font = nwg::Font::default();

    let _ = nwg::Font::builder()
        .family("Segoe UI")
        .size(16)
        .build(&mut font);

    nwg::Font::set_global_default(Some(font));

    dialog::open();
}

