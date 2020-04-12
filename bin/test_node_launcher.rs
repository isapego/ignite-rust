use std::env;
use std::env::current_dir;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Child, Command};

fn start<P: AsRef<OsStr>>(path: P) -> Child {
    let ignite_home = env::var_os("IGNITE_HOME").expect("IGNITE_HOME is not set");
    let bin_path = Path::new(&ignite_home).join("bin").join("ignite");

    let script_path = if cfg!(target_os = "windows") {
        bin_path.with_extension("bat")
    } else {
        bin_path.with_extension("sh")
    };

    Command::new(script_path)
        .arg(&path)
        .spawn()
        .expect("failed to spawn new node")
}

fn main() {
    let current_dir = current_dir().unwrap();
    let cfg = current_dir.join("tests").join("cfg").join("default.xml");

    let mut node = start(cfg);

    node.wait().unwrap();
}
