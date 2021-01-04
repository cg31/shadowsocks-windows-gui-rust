
extern crate embed_resource;
extern crate build;

fn main() {
    embed_resource::compile("./res/resources.rc");
    if cfg!(feature = "file-dialog") {
        build::link("shell32", true);
    }
}
