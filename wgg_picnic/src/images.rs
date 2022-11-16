use crate::models::ImageSize;

const STATIC_URL: &str = "https://storefront-prod.nl.picnicinternational.com/static";

/// Retrieve the image url for the provided image.
///
/// Note that no credentials are needed to retrieve these images, and can therefore be used at will.
/// This will retrieve images from the Dutch frontend, in most cases prefer to use [crate::PicnicApi::image_url].
pub fn image_url(image_id: impl AsRef<str>, size: ImageSize) -> String {
    format!("{}/images/{}/{}.png", STATIC_URL, image_id.as_ref(), size)
}
