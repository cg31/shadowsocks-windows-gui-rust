
use native_windows_gui as nwg;

use std::path::Path;
use std::path::PathBuf;

use winreg;
use winreg::enums::*;
use winreg::RegKey;

use anyhow::{Context, Result};

use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};


pub fn string_to_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

pub fn load_bitmap(bin: &[u8]) -> nwg::Bitmap {
    let mut bitmap = nwg::Bitmap::default();
    let res = nwg::Bitmap::builder().source_bin(Some(bin)).strict(true).build(&mut bitmap);
    assert!(res.is_ok());
    bitmap
}

pub fn _load_icon(bin: &[u8]) -> nwg::Icon {
    let mut icon = nwg::Icon::default();
    let res = nwg::Icon::builder().source_bin(Some(bin)).strict(true).build(&mut icon);
    assert!(res.is_ok());
    icon
}

pub fn _load_embed_icon(bin: &str) -> nwg::Icon {
    let mut icon = nwg::Icon::default();
    let res = nwg::Icon::builder().source_embed_str(Some(bin)).strict(true).build(&mut icon);
    assert!(res.is_ok());
    icon
}

pub fn exe_path() -> PathBuf {
    let mut p = std::env::current_exe().unwrap();
    p.pop();
    p
}

// add program to startup
pub fn autostart(enable: bool) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = Path::new(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run");
    let (key, _) = hkcu.create_subkey(&path).context("unable to create regkey")?;

    if enable {
        let p = std::env::current_exe().context("unable to get exe path")?;
        key.set_value("Russ", &p.as_os_str()).context("unable to set regkey")?;
    } else {
        key.delete_value("Russ").context("unable to delete regkey")?;
    }
    Ok(())
}

pub fn init_log<P: AsRef<Path>>(file_path: P) {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} {l} - {m}\n")))
        .build(file_path).unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder()
                   .appender("logfile")
                   .build(LevelFilter::Info)).unwrap();

    log4rs::init_config(config).unwrap();
}
