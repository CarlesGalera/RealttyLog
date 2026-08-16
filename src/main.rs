mod app;
mod encoding;
mod search;
mod tailer;

use app::App;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "RealttyLog",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}
