//! The font registry: interned font tags shared by layout, rasterization, and
//! the scene journal.
//!
//! A glyph carries a `font_id: u64`; renderers resolve it to actual bytes
//! here. Interning deduplicates by blob identity so each font file is copied
//! exactly once.

use parley::FontData;
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// Interned font storage: tag → (bytes, collection index).
///
/// Deduplication uses the blob's own unique id (linebender resource handles
/// carry one) — no pointer arithmetic, no byte copying for repeats.
#[derive(Debug, Default)]
pub struct FontRegistry {
    by_blob: FxHashMap<u64, u64>,
    fonts: Vec<(u64, Arc<[u8]>, u32)>,
}

impl FontRegistry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Interns a parley font, returning its stable tag.
    #[must_use]
    pub fn intern(&mut self, font: &FontData) -> u64 {
        let blob_id = font.data.id();
        if let Some(&id) = self.by_blob.get(&blob_id) {
            return id;
        }
        let bytes: &[u8] = font.data.as_ref();
        let id = u64::try_from(self.fonts.len()).expect("font registry fits u64");
        self.by_blob.insert(blob_id, id);
        self.fonts
            .push((id, Arc::from(bytes.to_vec().into_boxed_slice()), font.index));
        id
    }

    /// Resolves a font tag to its bytes and collection index.
    #[must_use]
    pub fn font_data(&self, id: u64) -> Option<(Arc<[u8]>, u32)> {
        self.fonts
            .iter()
            .find(|(tag, _, _)| *tag == id)
            .map(|(_, data, index)| (Arc::clone(data), *index))
    }

    /// Number of interned fonts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.fonts.len()
    }

    /// `true` when no font is interned.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fonts.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::FontRegistry;
    use parley::FontData;
    use std::sync::Arc;

    fn fake_font(tag: u8) -> FontData {
        FontData::new(
            linebender_resource_handle::Blob::new(
                Arc::new(vec![tag, 1, 2, 3, 4].into_boxed_slice())
                    as Arc<dyn AsRef<[u8]> + Send + Sync>,
            ),
            0,
        )
    }

    #[test]
    fn interning_the_same_blob_is_stable() {
        let mut registry = FontRegistry::new();
        let font = fake_font(7);
        let a = registry.intern(&font);
        let b = registry.intern(&font);
        assert_eq!(a, b, "same blob → same tag");
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn distinct_blobs_get_distinct_tags() {
        let mut registry = FontRegistry::new();
        let a = registry.intern(&fake_font(1));
        let b = registry.intern(&fake_font(2));
        assert_ne!(a, b);
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn resolution_returns_bytes_and_index() {
        let mut registry = FontRegistry::new();
        let id = registry.intern(&fake_font(9));
        let (data, index) = registry.font_data(id).expect("interned");
        assert_eq!(data.as_ref(), &[9, 1, 2, 3, 4]);
        assert_eq!(index, 0);
        assert!(registry.font_data(999).is_none());
    }
}
