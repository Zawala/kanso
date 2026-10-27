//! What a paste puts in the note. The text rules (a web address over selected
//! words, HTML as Markdown, a block on lines of its own, `<img>` tags) are
//! `kanso_engine::edit::paste`; an image becoming an attachment is here.

use kanso_core::links::LINK_SCHEME;

pub use kanso_engine::edit::paste::*;

/// What a pasted or dropped image is stored as: the API's image types as
/// they are, others converted to PNG.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stored {
    /// Kept as it is, with this content type and file extension.
    AsIs(&'static str, &'static str),
    /// Decoded and stored as a PNG.
    Png,
}

/// How to store an image, from its file extension or format name
/// (`png`, `jpg`, `tiff`…); `None` when it isn't an image Kanso reads.
pub fn stored_as(extension: &str) -> Option<Stored> {
    Some(match extension.to_ascii_lowercase().as_str() {
        "png" => Stored::AsIs("image/png", "png"),
        "jpg" | "jpeg" => Stored::AsIs("image/jpeg", "jpg"),
        "gif" => Stored::AsIs("image/gif", "gif"),
        "svg" => Stored::AsIs("image/svg+xml", "svg"),
        "heic" => Stored::AsIs("image/heic", "heic"),
        "heif" => Stored::AsIs("image/heif", "heif"),
        "webp" | "bmp" | "tif" | "tiff" | "ico" => Stored::Png,
        _ => return None,
    })
}

/// Stores an image (its bytes, and its extension or format name) as an
/// attachment, converting what the API doesn't take to PNG. Returns the
/// attachment's ID (`file:…`), or what to tell the user.
pub fn attach_image(
    store: &kanso_store::Store,
    bytes: &[u8],
    extension: &str,
    name: Option<&str>,
) -> Result<String, String> {
    use kanso_core::model::MAX_ATTACHMENT_SIZE;
    use std::borrow::Cow;
    let Some(kind) = stored_as(extension) else {
        return Err(
            "Only images can be attached: PNG, JPEG, GIF, SVG, HEIC, WebP, TIFF or BMP.".into(),
        );
    };
    let (data, content_type, ext) = match kind {
        Stored::AsIs(content_type, ext) => (Cow::Borrowed(bytes), content_type, ext),
        Stored::Png => (Cow::Owned(to_png(bytes)?), "image/png", "png"),
    };
    if data.len() > MAX_ATTACHMENT_SIZE {
        return Err(too_large(data.len(), MAX_ATTACHMENT_SIZE));
    }
    let stem: String = name
        .and_then(|n| {
            std::path::Path::new(n)
                .file_stem()?
                .to_str()
                .map(str::to_owned)
        })
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "image".into())
        .chars()
        .take(100)
        .collect();
    use base64::Engine as _;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
    let doc = serde_json::json!({
        "name": format!("{stem}.{ext}"),
        "contentType": content_type,
        "publicIn": [],
        "_attachments": {"index": {"content_type": content_type, "data": encoded}},
    });
    store
        .put(
            kanso_core::DocType::File,
            doc,
            kanso_store::Put::Upsert,
            kanso_store::Origin::Ui,
        )
        .map(|written| written.id)
        .map_err(|err| format!("Couldn't attach the image: {err}"))
}

/// An image in another format, as a PNG.
fn to_png(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let image =
        image::load_from_memory(bytes).map_err(|err| format!("Couldn't read the image: {err}"))?;
    let mut out = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut out, image::ImageFormat::Png)
        .map_err(|err| format!("Couldn't convert the image: {err}"))?;
    Ok(out.into_inner())
}

/// The Markdown that shows attachment `file_id` (`file:…`).
pub fn image_markdown(file_id: &str) -> String {
    format!("![]({LINK_SCHEME}{file_id})")
}

/// "4.2 MB": a size for a message.
pub fn megabytes(bytes: usize) -> String {
    let mb = bytes as f64 / (1024. * 1024.);
    if mb >= 10. {
        format!("{mb:.0} MB")
    } else {
        format!("{mb:.1} MB")
    }
}

/// The notice for an image over the attachment limit.
pub fn too_large(bytes: usize, limit: usize) -> String {
    format!(
        "That image is {}. Attachments can be up to {}.",
        megabytes(bytes),
        megabytes(limit)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    #[test]
    fn image_types() {
        assert_eq!(stored_as("PNG"), Some(Stored::AsIs("image/png", "png")));
        assert_eq!(stored_as("jpeg"), Some(Stored::AsIs("image/jpeg", "jpg")));
        assert_eq!(stored_as("tiff"), Some(Stored::Png));
        assert_eq!(stored_as("pdf"), None);
        assert_eq!(image_markdown("file:abc"), "![](inkdrop://file:abc)");
        assert_eq!(
            too_large(12 * 1024 * 1024, 10 * 1024 * 1024),
            "That image is 12 MB. Attachments can be up to 10 MB."
        );
        assert_eq!(megabytes(4_404_019), "4.2 MB");
    }

    fn store() -> (tempfile::TempDir, kanso_store::Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = kanso_store::Store::open(dir.path().join("k.db")).unwrap();
        (dir, store)
    }

    /// An image of `w` by `h` pixels encoded as `format`.
    fn encoded(w: u32, h: u32, seed: u8, format: image::ImageFormat) -> Vec<u8> {
        let image = image::RgbaImage::from_fn(w, h, |x, y| {
            image::Rgba([seed.wrapping_add(x as u8), y as u8, seed, 255])
        });
        let mut out = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut out, format)
            .unwrap();
        out.into_inner()
    }

    #[test]
    fn images_become_attachments() {
        let (_dir, store) = store();
        let png = encoded(3, 2, 7, image::ImageFormat::Png);
        let id = attach_image(&store, &png, "png", Some("Screen Shot.png")).unwrap();
        let (content_type, blob) = store.attachment(&id).unwrap().unwrap();
        assert_eq!(content_type, "image/png");
        assert_eq!(blob.data, png);
        let doc = store.get(&id).unwrap().unwrap();
        assert_eq!(doc.fields["name"], "Screen Shot.png");

        // TIFF isn't one of the API's types: stored as a PNG of the same picture.
        let tiff = encoded(4, 5, 9, image::ImageFormat::Tiff);
        let id = attach_image(&store, &tiff, "tiff", None).unwrap();
        let (content_type, blob) = store.attachment(&id).unwrap().unwrap();
        assert_eq!(content_type, "image/png");
        let back = image::load_from_memory(&blob.data).unwrap().to_rgba8();
        let original = image::load_from_memory(&tiff).unwrap().to_rgba8();
        assert_eq!(back, original);
        assert_eq!(store.get(&id).unwrap().unwrap().fields["name"], "image.png");

        assert!(
            attach_image(&store, b"%PDF", "pdf", None)
                .unwrap_err()
                .contains("Only images")
        );
        let big = vec![0u8; 12 * 1024 * 1024 + 1];
        assert_eq!(
            attach_image(&store, &big, "png", None).unwrap_err(),
            "That image is 12 MB. Attachments can be up to 10 MB."
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 32, ..config() })]

        /// Round trip: an image in a format Kanso converts comes back as
        /// the same pixels.
        #[test]
        fn prop_converted_images_keep_their_pixels(w in 1u32..12, h in 1u32..12, seed: u8, format in prop::sample::select(vec![
            (image::ImageFormat::Tiff, "tiff"), (image::ImageFormat::Bmp, "bmp"),
        ])) {
            let (_dir, store) = store();
            let bytes = encoded(w, h, seed, format.0);
            let id = attach_image(&store, &bytes, format.1, None).unwrap();
            let (_, blob) = store.attachment(&id).unwrap().unwrap();
            let back = image::load_from_memory(&blob.data).unwrap().to_rgba8();
            prop_assert_eq!(back, image::load_from_memory(&bytes).unwrap().to_rgba8());
        }
    }
}
