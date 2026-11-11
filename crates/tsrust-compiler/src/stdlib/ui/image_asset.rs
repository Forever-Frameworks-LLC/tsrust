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
