
use std::io::Write;

fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("res/shadowsocks.ico")
        .set_language(
            winapi::um::winnt::MAKELANGID(
                winapi::um::winnt::LANG_ENGLISH,
                winapi::um::winnt::SUBLANG_ENGLISH_US
            )
        );

    match res.compile() {
        Err(e) => {
            write!(std::io::stderr(), "{}", e).unwrap();
            std::process::exit(1);
        }
        Ok(_) => {}
    }
}
