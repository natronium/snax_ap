use crate::loader_app::SnaxAPLoader;

mod injector;
mod loader_app;

fn main() -> eframe::Result {
    egui_logger::builder().init().expect("could not initialize egui logger");
    eframe::run_native(
        "Bugsnax Archipelago Launcher",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::<SnaxAPLoader>::default())),
    )
}
