//! One decoded image per mounted preview; layout changes never restart its loader.
use gpui_kit::gpui::{ImageCacheError, RenderImage};
use std::sync::Arc;

#[derive(Default)]
pub(super) struct ImageAsset {
    source: String,
    decoded: Option<Arc<RenderImage>>,
}

impl ImageAsset {
    pub(super) fn resolve(
        &mut self,
        source: &str,
        load: impl FnOnce() -> Option<Result<Arc<RenderImage>, ImageCacheError>>,
    ) -> Option<Result<Arc<RenderImage>, ImageCacheError>> {
        if self.source != source {
            self.source = source.to_owned();
            self.decoded = None;
        }
        if let Some(image) = &self.decoded {
            return Some(Ok(image.clone()));
        }
        let result = load();
        if let Some(Ok(image)) = &result {
            self.decoded = Some(image.clone());
        }
        result
    }
}

#[cfg(target_os = "macos")]
pub(super) struct AvifLoader;

#[cfg(target_os = "macos")]
impl gpui_kit::gpui::Asset for AvifLoader {
    type Source = std::sync::Arc<std::path::Path>;
    type Output = Result<Arc<RenderImage>, ImageCacheError>;

    fn load(path: Self::Source, cx: &mut gpui_kit::gpui::App) -> impl Future<Output = Self::Output> + Send + 'static {
        let renderer = cx.svg_renderer();
        async move {
            use gpui_kit::gpui::{Image, ImageFormat};
            use objc2_app_kit::NSBitmapImageRep;
            use objc2_foundation::NSData;

            let bytes = std::fs::read(path)?;
            let bitmap = NSBitmapImageRep::imageRepWithData(&NSData::with_bytes(&bytes))
                .and_then(|rep| rep.TIFFRepresentation())
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Could not decode AVIF"))?;
            Image::from_bytes(ImageFormat::Tiff, bitmap.to_vec()).to_image_data(renderer).map_err(Into::into)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoded_image_survives_resize_frames_and_releases_on_source_change() {
        let image = Arc::new(RenderImage::new(Vec::new()));
        let mut asset = ImageAsset::default();
        assert!(asset.resolve("first.png", || None).is_none());
        let first = asset
            .resolve("first.png", || Some(Ok(image.clone())))
            .unwrap()
            .unwrap();
        assert!(Arc::ptr_eq(&first, &image));
        drop(first);
        for _ in 0..128 {
            let frame = asset
                .resolve("first.png", || panic!("resize restarted image loading"))
                .unwrap()
                .unwrap();
            assert!(Arc::ptr_eq(&frame, &image));
        }
        assert_eq!(Arc::strong_count(&image), 2);
        assert!(asset.resolve("second.png", || None).is_none());
        assert_eq!(
            Arc::strong_count(&image),
            1,
            "the previous source must be released"
        );
        let failed = asset.resolve("second.png", || {
            Some(Err(
                std::io::Error::from(std::io::ErrorKind::NotFound).into()
            ))
        });
        assert!(
            matches!(failed, Some(Err(_))),
            "real failures remain distinguishable from loading"
        );
        let weak = Arc::downgrade(&image);
        drop(asset.resolve("second.png", || Some(Ok(image.clone()))));
        drop(image);
        assert!(weak.upgrade().is_some());
        drop(asset);
        assert!(
            weak.upgrade().is_none(),
            "unmount must release the decoded image"
        );
    }
}
