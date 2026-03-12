use crate::image_handler::errors::DecodedImageError;
use memmap2::Mmap;
use zune_core::bytestream::ZCursor;
use zune_core::options::DecoderOptions;
use zune_jpeg::JpegDecoder;
use zune_png::PngDecoder;

pub struct DecodedImage {
    pub pixles: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl DecodedImage {
    pub fn decode_jpg(bytes: &Mmap) -> Result<DecodedImage, DecodedImageError> {
        let mut options = DecoderOptions::default();
        options = options.jpeg_set_out_colorspace(zune_core::colorspace::ColorSpace::RGBA);

        let mut decoder = JpegDecoder::new_with_options(ZCursor::new(&bytes[..]), options);
        let pixels = decoder
            .decode()
            .map_err(|e| DecodedImageError::DecodeError(e.to_string()))?;

        let info = decoder
            .info()
            .ok_or(DecodedImageError::DecodeError("No info".into()))?;

        Ok(Self {
            pixles: pixels,
            width: info.width as usize,
            height: info.height as usize,
        })
    }

    pub fn decode_png(bytes: &Mmap) -> Result<DecodedImage, DecodedImageError> {
        let mut options = DecoderOptions::default();
        options = options.png_set_add_alpha_channel(true);
        let mut decoder = PngDecoder::new_with_options(ZCursor::new(&bytes), options);
        let pixels = decoder
            .decode_raw()
            .map_err(|e| DecodedImageError::DecodeError(e.to_string()))?;

        let info = decoder
            .info()
            .ok_or(DecodedImageError::DecodeError("No info".into()))?;

        Ok(Self {
            pixles: pixels,
            width: info.width as usize,
            height: info.height as usize,
        })
    }
}
