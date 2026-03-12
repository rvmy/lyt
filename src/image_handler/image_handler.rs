use crate::image_handler::decoder::DecodedImage;
use crate::image_handler::errors::ImageHandlerError;
use memmap2::Mmap;
use std::fs::File;
use std::path::PathBuf;

enum ImageFormat {
    Jpeg,
    Png,
}

impl ImageFormat {
    fn detect(mmap: &Mmap) -> Result<Self, ImageHandlerError> {
        match infer::get(&mmap[..]) {
            Some(k) => match k.extension() {
                "jpg" | "jpeg" => Ok(Self::Jpeg),
                "png" => Ok(Self::Png),
                _ => Err(ImageHandlerError::UnSupportedFormat),
            },
            None => Err(ImageHandlerError::UnSupportedFormat),
        }
    }
}
pub struct ImageHandler {
    pub path: PathBuf,
    pub size_bytes: Option<u64>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub texture: Option<egui::TextureHandle>,
}

impl std::fmt::Debug for ImageHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageHandler")
            .field("path", &self.path)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("texture", &self.texture.is_some())
            .finish()
    }
}
impl ImageHandler {
    pub fn new(path: PathBuf) -> ImageHandler {
        Self {
            path,
            size_bytes: None,
            width: None,
            height: None,
            texture: None,
        }
    }

    pub fn decode(&mut self) -> DecodedImage {
        let file = File::open(&self.path).unwrap();
        let size_bytes = file.metadata().unwrap().len();
        let mmap = unsafe { Mmap::map(&file) }.unwrap();

        let format = ImageFormat::detect(&mmap).unwrap();

        let decoded_image = match format {
            ImageFormat::Jpeg => DecodedImage::decode_jpg(&mmap).unwrap(),
            ImageFormat::Png => DecodedImage::decode_png(&mmap).unwrap(),
        };

        self.size_bytes = Some(size_bytes);
        self.height = Some(decoded_image.height);
        self.width = Some(decoded_image.width);
        decoded_image
    }

    pub fn gpu_upload(&mut self, ctx: &egui::Context, pixels: Vec<u8>) {
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [self.width.unwrap(), self.height.unwrap()],
            &pixels,
        );
        let texture = ctx.load_texture(
            self.path.to_string_lossy(),
            color_image,
            egui::TextureOptions::NEAREST,
        );
        self.texture = Some(texture);
    }
}
