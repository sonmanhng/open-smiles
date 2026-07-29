mod core;
mod ui;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("OpenSMILES"),
        ..Default::default()
    };

    eframe::run_native(
        "OpenSMILES",
        options,
        Box::new(|_cc| Ok(Box::new(ui::app::OpenSmilesApp::default()))),
    )
}
