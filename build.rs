
use std::io::Write;

fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("res/shadowsocks.ico")
        .set_language(0x0409); // LANG_ENGLISH

    match res.compile() {
        Err(e) => {
            write!(std::io::stderr(), "{}", e).unwrap();
            std::process::exit(1);
        }
        Ok(_) => {}
    }
}
