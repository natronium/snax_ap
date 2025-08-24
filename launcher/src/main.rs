use constcat::concat;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

mod injector;
// mod loader_app;

fn main() {
    // let exe_location = detect_bugsnax_location().unwrap();
    // let snax_stdout = injector::install_mod(&exe_location);

    let default_connection = communication_wrapper::ConnectionInfo {
        host: "localhost".to_string(),
        port: 38281,
        slot_name: "Player1".to_string(),
        password: None,
    };

    let mut snax_wrapper = communication_wrapper::CommunicationWrapper::start(default_connection);

    std::thread::spawn(move || {
        while let Some(msg) = snax_wrapper.channel_in.blocking_recv() {
            println!("{msg:?}");
        }
    });

    std::thread::sleep(Duration::from_secs(2));

    snax_wrapper
        .channel_out
        .blocking_send(communication_wrapper::APMessage::LocationsToScout {
            location_ids: vec![200],
        })
        .unwrap();
    std::thread::sleep(Duration::from_secs(2));

    snax_wrapper
        .channel_out
        .blocking_send(communication_wrapper::APMessage::LocationsToCheck {
            location_ids: vec![200],
        })
        .unwrap();
    std::thread::sleep(Duration::from_secs(2));

    snax_wrapper
        .channel_out
        .blocking_send(communication_wrapper::APMessage::GoalCompletion {})
        .unwrap();
}

mod communication_wrapper;

const BUGSNAX_DIR: &str = r#"C:\Program Files (x86)\Steam\steamapps\common\Bugsnax"#;

const BUGSNAX_EXE_PATH: &str = concat!(BUGSNAX_DIR, "\\Bugsnax.exe");

fn detect_bugsnax_location() -> Option<PathBuf> {
    let path = Path::new(BUGSNAX_EXE_PATH).to_path_buf();
    if path.exists() { Some(path) } else { None }
}
