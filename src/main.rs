mod app;
mod image_handler;
mod ui;

fn main() -> eframe::Result {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let options = eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1920.0, 1080.0]),
            ..Default::default()
        };

        eframe::run_native(
            "lyt",
            options,
            Box::new(move |cc| {
                cc.egui_ctx
                    .options_mut(|opt| opt.reduce_texture_memory = true);
                Ok(Box::new(app::App::default()))
            }),
        )
    })
}
