//! The scene journal: append-only paint records, replayable, node-layered
//! (GPUI's scene architecture, §9.1).
//!
//! Layers are per node; the paint pass replaces a node's layer only when its
//! `PAINT` mask is set — every other subtree replays from the journal
//! untouched. `snapshot` materializes primitives in paint order for the
//! renderers; per-kind batching for instanced GPU draws happens downstream.

use alacer_core::arena::NodeId;
use alacer_core::passes::SceneJournal;
use alacer_core::scene::{Primitive, SceneSnapshot};
use rustc_hash::FxHashMap;

/// The journal implementation the pipeline uses.
#[derive(Debug, Default)]
pub struct SceneJournalImpl {
    layers: FxHashMap<NodeId, Vec<Primitive>>,
    commits: u64,
    drops: u64,
}

impl SceneJournalImpl {
    /// An empty journal.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of layer commits so far (re-emissions included).
    #[must_use]
    pub const fn commits(&self) -> u64 {
        self.commits
    }

    /// Number of layer drops (structural churn).
    #[must_use]
    pub const fn drops(&self) -> u64 {
        self.drops
    }

    /// Number of live layers.
    #[must_use]
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// The recorded primitives of one node's layer (replay access).
    #[must_use]
    pub fn layer(&self, node: NodeId) -> Option<&[Primitive]> {
        self.layers.get(&node).map(Vec::as_slice)
    }
}

impl SceneJournal for SceneJournalImpl {
    fn commit_layer(&mut self, node: NodeId, primitives: Vec<Primitive>) {
        self.commits += 1;
        self.layers.insert(node, primitives);
    }

    fn drop_layer(&mut self, node: NodeId) {
        if self.layers.remove(&node).is_some() {
            self.drops += 1;
        }
    }

    fn layer(&self, node: NodeId) -> Option<Vec<Primitive>> {
        self.layers.get(&node).cloned()
    }

    fn snapshot(&self, order: &[NodeId]) -> SceneSnapshot {
        let primitives = order
            .iter()
            .filter_map(|node| self.layers.get(node))
            .flatten()
            .cloned()
            .collect();
        SceneSnapshot { primitives }
    }

    fn retain(&mut self, order: &[NodeId]) {
        let keep: rustc_hash::FxHashSet<NodeId> = order.iter().copied().collect();
        let before = self.layers.len();
        self.layers.retain(|node, _| keep.contains(node));
        self.drops += u64::try_from(before - self.layers.len()).unwrap_or(0);
    }

    fn clear(&mut self) {
        self.drops += u64::try_from(self.layers.len()).unwrap_or(0);
        self.layers.clear();
    }
}

/// Partitions a snapshot into per-kind primitive vectors for instanced
/// draws (§9.1 `batches()`). The materialization shape the bench and the
/// tests consume: the shipped backends walk the snapshot's primitives
/// directly (the wgpu renderer builds its own per-kind draw lists over the
/// same `SceneSnapshot`), so this partition is the measured vocabulary,
/// not a present path.
#[must_use]
pub fn partition(scene: &SceneSnapshot) -> SceneBatches {
    let mut batches = SceneBatches::default();
    for primitive in &scene.primitives {
        match primitive {
            Primitive::Quad(quad) => batches.quads.push(*quad),
            Primitive::Glyph(glyph) => batches.glyphs.push(*glyph),
        }
    }
    batches
}

/// Per-kind primitive vectors.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SceneBatches {
    /// All quads in paint order.
    pub quads: Vec<alacer_core::scene::Quad>,
    /// All glyphs in paint order.
    pub glyphs: Vec<alacer_core::scene::Glyph>,
}

#[cfg(test)]
mod tests {
    use super::{SceneJournalImpl, partition};
    use alacer_core::arena::NodeId;
    use alacer_core::color::Color;
    use alacer_core::geom::{Point, Rect, Size};
    use alacer_core::passes::SceneJournal;
    use alacer_core::scene::{Primitive, Quad};

    fn quad(x: f32, color: Color) -> Primitive {
        Primitive::Quad(Quad::filled(
            Rect::new(Point::new(x, 0.0), Size::new(10.0, 10.0)),
            color,
        ))
    }

    #[test]
    fn snapshot_materializes_in_paint_order() {
        let mut journal = SceneJournalImpl::new();
        let order = vec![NodeId::allocate(), NodeId::allocate()];
        journal.commit_layer(
            order[0],
            vec![quad(0.0, Color::BLACK), quad(1.0, Color::WHITE)],
        );

        let snapshot = journal.snapshot(&order);
        assert_eq!(snapshot.primitives.len(), 2);
        let Primitive::Quad(first) = &snapshot.primitives[0] else {
            panic!("quad expected");
        };
        assert_eq!(first.rect.origin.x, 0.0);
    }

    #[test]
    fn commit_replaces_a_layer_in_place() {
        let mut journal = SceneJournalImpl::new();
        let node = NodeId::allocate();
        let order = [node];
        journal.commit_layer(node, vec![quad(0.0, Color::BLACK)]);
        journal.commit_layer(node, vec![quad(5.0, Color::WHITE)]);
        assert_eq!(journal.layer_count(), 1, "replaced, not duplicated");
        assert_eq!(journal.commits(), 2);
        let snapshot = journal.snapshot(&order);
        assert_eq!(snapshot.primitives.len(), 1);
    }

    #[test]
    fn clean_layers_replay_from_the_journal() {
        let mut journal = SceneJournalImpl::new();
        let a = NodeId::allocate();
        journal.commit_layer(a, vec![quad(0.0, Color::BLACK)]);
        let first = journal.snapshot(&[a]);
        let second = journal.snapshot(&[a]);
        assert_eq!(first, second, "replay is deterministic");
    }

    #[test]
    fn retain_and_clear_drop_stale_layers() {
        let mut journal = SceneJournalImpl::new();
        let keep = NodeId::allocate();
        journal.commit_layer(keep, vec![quad(0.0, Color::BLACK)]);
        journal.retain(&[keep]);
        assert_eq!(journal.layer_count(), 1);

        journal.clear();
        assert_eq!(journal.layer_count(), 0);
        assert_eq!(journal.snapshot(&[keep]).primitives, []);
    }

    #[test]
    fn partition_splits_kinds_for_batching() {
        let mut journal = SceneJournalImpl::new();
        let node = NodeId::allocate();
        journal.commit_layer(
            node,
            vec![
                quad(0.0, Color::BLACK),
                Primitive::Glyph(alacer_core::scene::Glyph {
                    position: Point::new(1.0, 2.0),
                    size: 14.0,
                    color: Color::BLACK,
                    font_id: 1,
                    glyph_id: 65,
                    clip: None,
                }),
            ],
        );
        let snapshot = journal.snapshot(&[node]);
        let batches = partition(&snapshot);
        assert_eq!(batches.quads.len(), 1);
        assert_eq!(batches.glyphs.len(), 1);
    }
}
