use constcat::concat;
use dll_syringe::{Syringe, process::OwnedProcess};
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

const BUGSNAX_DIR: &str = r#"C:\Program Files (x86)\Steam\steamapps\common\Bugsnax"#;
const BUGSNAX_EXE_PATH: &str = concat!(BUGSNAX_DIR, "\\Bugsnax.exe");

fn main() {
    let mut child = Command::new(BUGSNAX_EXE_PATH)
        .current_dir(BUGSNAX_DIR)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to launch bugsnax");

    let target_process =
        OwnedProcess::from_pid(child.id()).expect("could not find child process by id");
    let syringe = Syringe::for_process(target_process);
    let injected_payload = syringe.inject("./target/debug/snax_lib.dll").unwrap();
    let init = unsafe {
        syringe.get_raw_procedure::<extern "C" fn() -> ()>(injected_payload, "install_hooks")
    }
    .expect("failed to load procedure \"install_hooks\"")
    .expect("couldn't find procedure");

    init.call().expect("install_hooks RPC failed");

    let snax_stdout = child.stdout.take().expect("could not take snax stdout");

    let reader = BufReader::new(snax_stdout);

    for line in reader.lines() {
        let line = line.expect("could not read line from snax stdout");
        println!("{line}");
    }

    let ecode = child.wait().unwrap();
    println!("Bugsnax exited with code: {ecode}");
}
