
use native_windows_gui as nwg;

use std::path::Path;

use winreg;
use winreg::enums::*;
use winreg::RegKey;

use anyhow::{Context, Result};


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
