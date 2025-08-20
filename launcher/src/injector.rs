use dll_syringe::Syringe;
use dll_syringe::process::OwnedProcess;
use nameof::name_of;
use snax_lib::install_hooks;
use std::path::Path;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;

pub fn install_mod(exe_path: &Path) -> ChildStdout {
    let mut child = Command::new(exe_path)
        .current_dir(
            exe_path
                .parent()
                .expect("Bugsnax.exe path has no parent. What directory does it live in??"),
        )
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to launch bugsnax");

    let target_process =
        OwnedProcess::from_pid(child.id()).expect("could not find child process by id");
    let syringe = Syringe::for_process(target_process);
    let injected_payload = syringe.inject("./target/debug/snax_lib.dll").unwrap();

    let init = unsafe {
        syringe
            .get_raw_procedure::<extern "C" fn() -> ()>(injected_payload, name_of!(install_hooks))
    }
    .expect("failed to load procedure \"install_hooks\"")
    .expect("couldn't find procedure");

    init.call().expect("install_hooks RPC failed");

    child.stdout.take().expect("could not take snax stdout")
}
