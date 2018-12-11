use image::{DynamicImage, GenericImage, ImageError};
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    FileNotSupported,
}

pub struct Alphamap {
    src_img: DynamicImage,
}

impl Alphamap {
    pub fn from_png_file(
        file_path: &Path,
        desired_width: usize,
        desired_height: usize,
    ) -> Result<Self, Error> {
        let src_img = image::open(file_path)?;
        let (src_width, src_height) = src_img.dimensions();

        assert_eq!(src_width as usize, desired_width);
        assert_eq!(src_height as usize, desired_height);

        Ok(Self { src_img })
    }

    pub fn src_texture(&self) -> DynamicImage {
        DynamicImage::ImageRgb8(self.src_img.to_rgb())
    }
}

impl From<ImageError> for Error {
    fn from(e: ImageError) -> Error {
        match e {
            _ => Error::FileNotSupported,
        }
    }
}
