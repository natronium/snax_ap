use std::{io::{BufRead, BufReader}, process::{Command, Stdio}};
use constcat::concat;
use dll_syringe::{process::OwnedProcess, Syringe};

const BUGSNAX_DIR: &str = r#"C:\Program Files (x86)\Steam\steamapps\common\Bugsnax"#;
const BUGSNAX_EXE_PATH: &str = concat!(BUGSNAX_DIR, "\\Bugsnax.exe");

fn main() {
    let mut child = Command::new(BUGSNAX_EXE_PATH)
        .current_dir(BUGSNAX_DIR)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to launch bugsnax");

    let target_process = OwnedProcess::from_pid(child.id()).expect("could not find child process by id");
    let syringe = Syringe::for_process(target_process);
    let _injected_payload = syringe.inject("./target/debug/snax_lib.dll").unwrap();
    
    let snax_stdout = child.stdout.take().expect("could not take snax stdout");

    let reader = BufReader::new(snax_stdout);

    for line in reader.lines() {
        let line = line.expect("could not read line from snax stdout");
        println!("{line}");
    }


    let ecode = child.wait().unwrap();
    println!("Bugsnax exited with code: {ecode}");
}
