use image::{DynamicImage, GenericImage, ImageError, Pixel};
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    FileNotSupported,
}

pub struct Alphamap {
    src_img: DynamicImage,
}

impl Alphamap {
    pub fn from_png_file<P: AsRef<Path>>(
        file_path: Option<P>,
        desired_width: usize,
        desired_height: usize,
    ) -> Result<Self, Error> {
        let src_img = if let Some(p) = file_path {
            image::open(p)?
        } else {
            // Use a default image with Red channel maxed out, assumes a single
            // texture will be used
            let mut img = DynamicImage::new_rgb8(desired_width as _, desired_height as _);
            let (w, h) = img.dimensions();
            for y in 0..h {
                for x in 0..w {
                    img.put_pixel(x as _, y as _, Pixel::from_channels(255, 0, 0, 0));
                }
            }
            img
        };

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
