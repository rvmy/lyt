use crate::image_handler::ImageHandler;
use crate::ui;
use eframe::egui;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

#[derive(Default, PartialEq)]
pub enum Mode {
    #[default]
    Welcome,
    Viewer,
}

pub struct App {
    pub mode: Mode,
    pub images: Arc<Mutex<Vec<ImageHandler>>>,
    pub cursor: Arc<AtomicUsize>,
    pub notify: Arc<Notify>,
    pub scene_rect: egui::Rect,
    pub cache_size: usize,
    pub is_loading: Arc<AtomicBool>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            mode: Mode::Welcome,
            images: Arc::new(Mutex::new(Vec::new())),
            cursor: Arc::new(AtomicUsize::new(0)),
            scene_rect: egui::Rect::ZERO,
            notify: Arc::new(Notify::new()),
            cache_size: 2,
            is_loading: Arc::new(AtomicBool::new(false)),
        }
    }
}

fn unload_far_textures(images: &Arc<Mutex<Vec<ImageHandler>>>, idx: usize, cache_size: usize) {
    let keep = idx.saturating_sub(cache_size)..=idx.saturating_add(cache_size);
    let mut guard = images.lock().unwrap();
    guard
        .iter_mut()
        .enumerate()
        .filter(|(i, _)| !keep.contains(i))
        .for_each(|(_, img)| img.texture = None);
}

impl App {
    pub fn navigate(&mut self, delta: i32, ctx: &egui::Context) {
        let current = self.cursor.load(Ordering::Relaxed);
        let len = self.images.lock().unwrap().len();
        if len == 0 {
            return;
        }

        let next = (current as i32 + delta).rem_euclid(len as i32) as usize;

        let already_loaded = self
            .images
            .lock()
            .unwrap()
            .get(next)
            .map(|img| img.texture.is_some())
            .unwrap_or(false);

        if !already_loaded {
            self.is_loading.store(true, Ordering::Relaxed);

            let path = {
                let guard = self.images.lock().unwrap();
                guard.get(next).unwrap().path.clone()
            };

            let images = Arc::clone(&self.images);
            let is_loading = Arc::clone(&self.is_loading);
            let ctx = ctx.clone();

            tokio::spawn(async move {
                let mut handler = ImageHandler::new(path);
                let data = handler.decode();
                handler.gpu_upload(&ctx, data.pixles);

                images.lock().unwrap()[next] = handler;
                is_loading.store(false, Ordering::Relaxed);
                ctx.request_repaint();
            });
        }

        unload_far_textures(&self.images, next, self.cache_size);
        self.cursor.store(next, Ordering::Relaxed);
        self.scene_rect = egui::Rect::ZERO;
        ctx.request_repaint();
    }

    pub fn open_image(&mut self, path: &Path, ctx: &egui::Context) {
        let mut first_image = ImageHandler::new(path.to_path_buf());
        let data = first_image.decode();
        first_image.gpu_upload(ctx, data.pixles);

        {
            let mut guard = self.images.lock().unwrap();
            guard.clear();
            guard.push(first_image);
        }

        self.cursor.store(0, Ordering::Relaxed);
        self.scene_rect = egui::Rect::ZERO;
        self.mode = Mode::Viewer;

        let current_path = path.to_path_buf();
        let images = Arc::clone(&self.images);
        let cursor = Arc::clone(&self.cursor);
        let notify = Arc::clone(&self.notify);
        let ctx = ctx.clone();

        tokio::spawn(async move {
            let Some(dir) = current_path.parent() else {
                return;
            };
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };

            let mut paths: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    matches!(
                        p.extension().and_then(|s| s.to_str()),
                        Some("jpg" | "jpeg" | "png")
                    )
                })
                .collect();
            paths.sort();

            let idx = paths.iter().position(|p| p == &current_path).unwrap_or(0);
            let mut all: Vec<ImageHandler> = paths.into_iter().map(ImageHandler::new).collect();

            {
                let mut guard = images.lock().unwrap();
                all[idx] = guard.remove(0);
                *guard = all;
            }

            cursor.store(idx, Ordering::Relaxed);
            ctx.request_repaint();
            notify.notify_one();
        });
    }

    pub fn open_folder(&mut self, _path: &Path, _ctx: &egui::Context) {}

    pub fn current_texture(&self) -> Option<egui::TextureHandle> {
        let guard = self.images.lock().unwrap();
        guard
            .get(self.cursor.load(Ordering::Relaxed))?
            .texture
            .clone()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::O) && !i.modifiers.ctrl) {
            if let Some(p) = rfd::FileDialog::new()
                .add_filter("Images", &["jpg", "jpeg"])
                .pick_file()
            {
                self.open_image(&p, ctx);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.mode {
            Mode::Welcome => {
                if let Some(action) = ui::welcome::show(ui) {
                    match action {
                        ui::welcome::Action::OpenImage(p) => self.open_image(&p, ctx),
                        ui::welcome::Action::OpenFolder(p) => self.open_folder(&p, ctx),
                    }
                }
            }
            Mode::Viewer => {
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                    self.navigate(1, ctx);
                }
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                    self.navigate(-1, ctx);
                }
                ui::viewer::show(ui, ctx, self);
            }
        });
    }
}
