use std::process::Command;
use constcat::concat;

const BUGSNAX_DIR: &str = r#"C:\Program Files (x86)\Steam\steamapps\common\Bugsnax"#;
const BUGSNAX_EXE_PATH: &str = concat!(BUGSNAX_DIR, "\\Bugsnax.exe");

fn main() {
    let mut child = Command::new(BUGSNAX_EXE_PATH)
        .current_dir(BUGSNAX_DIR)
        .spawn()
        .expect("Failed to launch bugsnax");
    let ecode = child.wait();
    println!("Bugsnax exited with code: {ecode:#?}");
}
