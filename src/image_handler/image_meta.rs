use memmap2::Mmap;
use std::fs::File;
use std::path::Path;
use thiserror::Error;
use zune_core::bytestream::ZCursor;
use zune_core::options::DecoderOptions;
use zune_jpeg::JpegDecoder;
use zune_png::PngDecoder;

#[derive(Debug, Clone, Error)]
pub enum ImageMetaError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("Unsupported image format")]
    UnSupportedFormat,
}
impl From<std::io::Error> for ImageMetaError {
    fn from(e: std::io::Error) -> Self {
        ImageMetaError::Io(e.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct ImageMeta {
    pub path: String,
    pub size_bytes: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    format: ImageFormat,
    pub pixles: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
enum ImageFormat {
    Jpeg,
    Png,
}

impl ImageMeta {
    pub fn new(file_path: &str) -> Result<Self, ImageMetaError> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let size_bytes = metadata.len();
        let mmap = unsafe { Mmap::map(&file)? };
        let format = match infer::get(&mmap[..]) {
            Some(kind) => match kind.extension() {
                "jpg" | "jpeg" => ImageFormat::Jpeg,
                "png" => ImageFormat::Png,

                _ => return Err(ImageMetaError::UnSupportedFormat),
            },
            None => return Err(ImageMetaError::UnSupportedFormat),
        };

        let mut image = Self {
            path: file_path.to_string(),
            size_bytes,
            width: None,
            height: None,
            format,
            pixles: None,
        };

        image.decode(&mmap)?;

        Ok(image)
    }

    fn decode(&mut self, bytes: &Mmap) -> Result<(), ImageMetaError> {
        let (pixles, width, height) = match self.format {
            ImageFormat::Jpeg => Self::decode_jpg(bytes)?,
            ImageFormat::Png => Self::decode_png(bytes)?,
            _ => return Err(ImageMetaError::UnSupportedFormat),
        };

        self.pixles = Some(pixles);
        self.width = Some(width);
        self.height = Some(height);
        Ok(())
    }

    fn decode_jpg(bytes: &Mmap) -> Result<(Vec<u8>, u32, u32), ImageMetaError> {
        let mut options = DecoderOptions::default();
        options = options.jpeg_set_out_colorspace(zune_core::colorspace::ColorSpace::RGBA);

        let mut decoder = JpegDecoder::new_with_options(ZCursor::new(&bytes[..]), options);
        let pixels = decoder
            .decode()
            .map_err(|_| ImageMetaError::UnSupportedFormat)?;

        let info = decoder.info().unwrap();
        Ok((pixels, info.width as u32, info.height as u32))
    }

    fn decode_png(bytes: &Mmap) -> Result<(Vec<u8>, u32, u32), ImageMetaError> {
        let mut options = DecoderOptions::default();
        options = options.png_set_add_alpha_channel(true);
        let mut decoder = PngDecoder::new_with_options(ZCursor::new(&bytes), options);
        let pixels = decoder
            .decode_raw()
            .map_err(|_| ImageMetaError::UnSupportedFormat)?;
        let info = decoder.info().unwrap();
        Ok((pixels, info.width as u32, info.height as u32))
    }
}
