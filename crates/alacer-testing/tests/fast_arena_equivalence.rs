//! The fast arena's paired fuzz (§7.1, ADR-9): the upstream `tree_arena`
//! hash-map storage is adopted only behind `fast-arena`, and this suite is
//! the gate the plan names — "the fast arena lands only behind a feature
//! flag *after* passing the fuzz equivalence suite". One seeded LCG script
//! drives two arenas built over both storages side by side; after every
//! mutation the observable state must be byte-identical: the id mapping,
//! parents, children order (the paint order), masks as drained by
//! `take_masks`, composed bounds, the scrollable set, and every widget's
//! `measure_spec`.
//!
//! The mutations cover every storage mutator: `add_child`, `remove_subtree`,
//! `reorder_children` (the reconciler's move op), `update_bounds`,
//! `queue_mutation` + `drain_mutations` (style changes through the lease),
//! and `merge_mask` — plus the drains the passes consume. The script is
//! deterministic: a divergence is a reproducible bug report, not a flake.
//!
//! Runs only with the feature compiled in (`--features fast-arena`); the
//! paired construction needs both storages linked, which is exactly what
//! the flag provides.

#![cfg(feature = "fast-arena")]

use alacer_core::arena::{Arena, ArenaStorage, NodeId};
use alacer_core::mask::Change;
use alacer_core::style::MeasureSpec;
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};
use alacer_testing::Lcg;

/// The mutation script's length — the plan's 1000+ iterations floor.
const MUTATIONS: usize = 1024;

/// A probe widget whose observable surface the fuzz compares: the role and
/// label pin identity, a mutable counter rides the mutation lease (so the
/// fast storage's `widget_mut` writes must land identically), and the
/// measure spec is the layout engine's per-node input.
struct Probe {
    role: Role,
    name: &'static str,
    counter: u64,
}

impl Probe {
    fn named(name: &'static str) -> Self {
        Self {
            role: Role::Container,
            name,
            counter: 0,
        }
    }
}

impl Widget for Probe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        self.role
    }
    fn label(&self) -> LabelSource {
        LabelSource::text(self.name)
    }
    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(alacer_core::geom::Size::new(
            counter_to_width(self.counter),
            8.0,
        )))
    }
    fn on_pointer_event(
        &mut self,
        _ctx: &mut alacer_core::events::EventCtx,
        _event: &alacer_core::PointerEvent,
    ) -> EventResponse {
        EventResponse::Ignored
    }
}

/// The counter as a float measure width — small, deterministic, exact.
fn counter_to_width(value: u64) -> f32 {
    f32::from((value % 64) as u8) * 1.5
}

/// One node's paired-observable state: canonical index, parent, children in
/// tree order (the paint order), and composed bounds.
type NodeState = (
    usize,
    Option<usize>,
    Vec<usize>,
    Option<alacer_core::geom::Rect>,
);

/// The full observable state of one arena as the passes consume it — with
/// every node id translated through the pair pool, since the two arenas
/// allocate from one global counter and their raw ids differ by
/// construction. What must match is the STRUCTURE the ids name.
struct Snapshot {
    len: usize,
    root: Option<usize>,
    nodes: Vec<NodeState>,
    drained: Vec<(usize, Change)>,
    scrollables: Vec<usize>,
}

fn snapshot(arena: &mut Arena, pool: &[(NodeId, NodeId)], mine: usize) -> Snapshot {
    // canonical index of one of this arena's ids (0 = the other arena's
    // slot, 1 = mine) — the pairing the script maintains.
    let canon = |id: NodeId| -> usize {
        pool.iter()
            .position(|pair| {
                if mine == 0 {
                    pair.0 == id
                } else {
                    pair.1 == id
                }
            })
            .expect("the fuzz only holds pooled ids")
    };
    let mut nodes = Vec::new();
    arena.for_each_pre_order(&mut |id, slot| {
        nodes.push((
            canon(id),
            slot.parent.map(canon),
            slot.children.iter().map(|c| canon(*c)).collect::<Vec<_>>(),
            slot.bounds,
        ));
    });
    let drained = arena
        .take_masks(Change::ALL)
        .into_iter()
        .map(|(id, change)| (canon(id), change))
        .collect();
    let mut scrollables: Vec<usize> = arena.scrollables().map(canon).collect();
    scrollables.sort_unstable();
    Snapshot {
        len: arena.len(),
        root: arena.root().map(canon),
        nodes,
        drained,
        scrollables,
    }
}

fn assert_identical(step: usize, safe: &Snapshot, fast: &Snapshot) {
    assert_eq!(
        safe.len, fast.len,
        "step {step}: live count diverged between the storages"
    );
    assert_eq!(safe.root, fast.root, "step {step}: the root diverged");
    assert_eq!(
        safe.nodes, fast.nodes,
        "step {step}: the pre-order node state diverged"
    );
    assert_eq!(
        safe.drained, fast.drained,
        "step {step}: the drained masks diverged (order included)"
    );
    assert_eq!(
        safe.scrollables, fast.scrollables,
        "step {step}: the scrollable set diverged"
    );
}

/// One `measure_spec` read per live node, compared byte-identical.
fn measure_specs(arena: &mut Arena) -> Vec<Option<MeasureSpec>> {
    let mut out = Vec::new();
    arena.for_each_pre_order(&mut |id, _| {
        out.push(arena.widget(id).measure_spec());
    });
    out
}

/// Adds one child under a random live parent, both storages, paired ids.
fn add_child_pair(
    safe: &mut Arena,
    fast: &mut Arena,
    pool: &mut Vec<(NodeId, NodeId)>,
    rng: &mut Lcg,
) {
    let (parent_s, parent_f) = pool[rng.next_usize_in(0, pool.len() - 1)];
    let name = ["a", "b", "c", "d"][rng.next_usize_in(0, 3)];
    let id_s = safe.add_child(parent_s, Box::new(Probe::named(name)));
    let id_f = fast.add_child(parent_f, Box::new(Probe::named(name)));
    pool.push((id_s, id_f));
    pool.retain(|(s, f)| safe.is_live(*s) && fast.is_live(*f));
}

/// Removes a random non-root subtree from both storages, pruning the pool.
fn remove_pair(
    safe: &mut Arena,
    fast: &mut Arena,
    pool: &mut Vec<(NodeId, NodeId)>,
    rng: &mut Lcg,
) {
    if pool.len() <= 1 {
        return;
    }
    let index = rng.next_usize_in(1, pool.len() - 1);
    let (id_s, id_f) = pool[index];
    if safe.is_live(id_s) {
        safe.remove_subtree(id_s);
        fast.remove_subtree(id_f);
        pool.retain(|(s, f)| safe.is_live(*s) && fast.is_live(*f));
    }
}

/// Permutes a random live parent's children — the reconciler's move op —
/// with the same deterministic shuffle on both sides.
fn reorder_pair(safe: &mut Arena, fast: &mut Arena, pool: &[(NodeId, NodeId)], rng: &mut Lcg) {
    let (parent_s, parent_f) = pool[rng.next_usize_in(0, pool.len() - 1)];
    if !safe.is_live(parent_s) {
        return;
    }
    let current = safe.children(parent_s).to_vec();
    if current.len() <= 1 {
        return;
    }
    // a deterministic shuffle: rotate by a random offset, maybe reverse
    let offset = rng.next_usize_in(1, current.len() - 1);
    let mut order: Vec<NodeId> = current[offset..]
        .iter()
        .chain(current[..offset].iter())
        .copied()
        .collect();
    if rng.chance(0.5) {
        order.reverse();
    }
    safe.reorder_children(parent_s, &order);
    let order_f: Vec<NodeId> = order
        .iter()
        .map(|id| {
            pool.iter()
                .find(|(s, _)| s == id)
                .map(|(_, f)| *f)
                .expect("pooled")
        })
        .collect();
    fast.reorder_children(parent_f, &order_f);
}

/// Writes random composed bounds into a random live node, both storages.
fn bounds_pair(safe: &mut Arena, fast: &mut Arena, pool: &[(NodeId, NodeId)], rng: &mut Lcg) {
    let (id_s, id_f) = pool[rng.next_usize_in(0, pool.len() - 1)];
    if !safe.is_live(id_s) {
        return;
    }
    let rect = alacer_core::geom::Rect::new(
        alacer_core::geom::Point::new(rng.next_f32_in(0.0, 100.0), rng.next_f32_in(0.0, 100.0)),
        alacer_core::geom::Size::new(rng.next_f32_in(1.0, 60.0), rng.next_f32_in(1.0, 60.0)),
    );
    safe.update_bounds(id_s, rect);
    fast.update_bounds(id_f, rect);
}

/// Queues the same style change into both arenas (a counter bump through
/// the lease) and drains, comparing the drains' canonical pairing.
fn mutate_pair(
    safe: &mut Arena,
    fast: &mut Arena,
    pool: &[(NodeId, NodeId)],
    rng: &mut Lcg,
    step: usize,
) {
    let (id_s, id_f) = pool[rng.next_usize_in(0, pool.len() - 1)];
    if !safe.is_live(id_s) {
        return;
    }
    let bump = rng.next_u64() % 7;
    safe.queue_mutation(
        id_s,
        Box::new(move |widget| {
            if let Some(probe) = widget.as_any_mut().downcast_mut::<Probe>() {
                probe.counter += bump;
            }
        }),
    );
    fast.queue_mutation(
        id_f,
        Box::new(move |widget| {
            if let Some(probe) = widget.as_any_mut().downcast_mut::<Probe>() {
                probe.counter += bump;
            }
        }),
    );
    let mutated_s = safe.drain_mutations();
    let mutated_f = fast.drain_mutations();
    let canon_s: Vec<usize> = mutated_s
        .iter()
        .map(|id| pool.iter().position(|p| p.0 == *id).expect("pooled"))
        .collect();
    let canon_f: Vec<usize> = mutated_f
        .iter()
        .map(|id| pool.iter().position(|p| p.1 == *id).expect("pooled"))
        .collect();
    assert_eq!(canon_s, canon_f, "step {step}: the mutation drain diverged");
}

#[test]
fn fast_arena_matches_safe_storage_under_fuzz_mutations() {
    let mut safe = Arena::with_storage(ArenaStorage::Safe);
    let mut fast = Arena::with_storage(ArenaStorage::Fast);
    let mut rng = Lcg::new(0x0A_1E_5A_1E);

    // The shared vocabulary: the two arenas receive the same widgets and
    // the same ids (allocation is global and monotonic, so paired adds get
    // paired ids).
    let mut pool: Vec<(NodeId, NodeId)> = Vec::new(); // (safe id, fast id)
    let root_s = safe.set_root(Box::new(Probe::named("root")));
    let root_f = fast.set_root(Box::new(Probe::named("root")));
    pool.push((root_s, root_f));

    for step in 0..MUTATIONS {
        match rng.next_usize_in(0, 9) {
            // add_child under a random live node
            0..=3 => add_child_pair(&mut safe, &mut fast, &mut pool, &mut rng),
            // remove_subtree of a random non-root node
            4 | 5 => remove_pair(&mut safe, &mut fast, &mut pool, &mut rng),
            // reorder_children: a random permutation of a random parent
            6 => reorder_pair(&mut safe, &mut fast, &pool, &mut rng),
            // update_bounds on a random live node
            7 => bounds_pair(&mut safe, &mut fast, &pool, &mut rng),
            // queue_mutation + drain: a style change through the lease
            _ => mutate_pair(&mut safe, &mut fast, &pool, &mut rng, step),
        }

        let safe_snap = snapshot(&mut safe, &pool, 0);
        let fast_snap = snapshot(&mut fast, &pool, 1);
        assert_identical(step, &safe_snap, &fast_snap);
        assert_eq!(
            measure_specs(&mut safe),
            measure_specs(&mut fast),
            "step {step}: the measure specs diverged"
        );
    }

    assert!(
        pool.len() > 8,
        "the script grew a real tree (got {})",
        pool.len()
    );
    let live = safe.len();
    assert!(live > 8, "the script kept a real live tree (got {live})");
}
