//! The flux layer: the mask bridge over `reactive_graph` (architecture §5.2).
//!
//! Three rules, all test-enforced:
//!
//! 1. **Effects never mutate widgets.** Every effect drain ends by queueing
//!    `Change` masks (the mask bridge); widget mutation happens only inside the
//!    pass system.
//! 2. **One write batch ⇒ at most one layout+paint cycle** (ADR-6 kill-switch).
//!    `ImmediateEffect` runs synchronously; `batch` defers and coalesces N
//!    synchronous signal writes into exactly one rerun.
//! 3. **Reactive work is upstream of invalidation, never tangled with it.**

use crate::arena::NodeId;
use crate::inspect::{BindingId, EdgeActivation, SignalKey, SignalWrite, signal_key_of};
use crate::mask::Change;
use reactive_graph::computed::Memo;
use reactive_graph::effect::{ImmediateEffect, batch};
use reactive_graph::owner::Owner;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::Set;

/// The reactive signal handle the flux issues (`reactive_graph::RwSignal`).
///
/// Copy, thread-safe, and the type rsx! view closures capture.
pub type FluxRwSignal<T> = RwSignal<T>;
use reactive_graph::traits::Get;
use rustc_hash::FxHashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};

/// Monotonic counters for the scheduling-contract tests (ADR-6).
#[derive(Debug, Default)]
pub struct FluxCounters {
    effect_reruns: AtomicU64,
}

impl FluxCounters {
    /// Number of effect reruns so far (the initial synchronous run counts).
    #[must_use]
    pub fn effect_reruns(&self) -> u64 {
        self.effect_reruns.load(Ordering::Relaxed)
    }

    fn bump(&self) {
        self.effect_reruns.fetch_add(1, Ordering::Relaxed);
    }

    /// Resets all counters (test convenience).
    pub fn reset(&self) {
        self.effect_reruns.store(0, Ordering::Relaxed);
    }
}

/// Pending per-node masks awaiting the next frame.
#[derive(Debug, Default)]
struct MaskQueue {
    pending: FxHashMap<NodeId, Change>,
}

/// The dev-mode causal capture (§13.3, ADR-0010): edge registrations,
/// activations, and provenance-tagged writes. Off by default — an effect
/// run pays one boolean check to find out, and nothing allocates. Both
/// event buffers are bounded rings (§14.3: every buffer has an eviction
/// policy); the edge table is keyed and bounded by binding count.
#[derive(Debug, Default)]
struct CausalCapture {
    enabled: bool,
    next_binding: u64,
    write_sequence: u64,
    /// The binding table: binding id → (signal key, node). The "who
    /// subscribes" answer.
    edges: FxHashMap<BindingId, (SignalKey, NodeId)>,
    /// Activations since the last drain (the pipeline folds them into the
    /// frame's [`FrameSummary`](crate::inspect::FrameSummary)).
    activations: std::collections::VecDeque<EdgeActivation>,
    /// Provenance-tagged writes since the last drain.
    writes: std::collections::VecDeque<SignalWrite>,
}

/// The activation ring's cap (oldest dropped first).
const ACTIVATION_RING_CAP: usize = 4_096;
/// The write-provenance ring's cap (oldest dropped first).
const WRITE_RING_CAP: usize = 1_024;

impl CausalCapture {
    fn register(&mut self, signal: SignalKey, node: NodeId) -> BindingId {
        self.next_binding += 1;
        let binding = self.next_binding;
        self.edges.insert(binding, (signal, node));
        binding
    }

    fn record_activation(
        &mut self,
        binding: BindingId,
        signal: SignalKey,
        node: NodeId,
        masks: Change,
    ) {
        if self.activations.len() >= ACTIVATION_RING_CAP {
            self.activations.pop_front();
        }
        self.activations.push_back(EdgeActivation {
            binding,
            signal,
            node,
            masks,
        });
    }

    fn record_write(&mut self, signal: SignalKey, origin: std::borrow::Cow<'static, str>) {
        if self.writes.len() >= WRITE_RING_CAP {
            self.writes.pop_front();
        }
        self.write_sequence += 1;
        self.writes.push_back(SignalWrite {
            signal,
            sequence: self.write_sequence,
            origin,
        });
    }
}

impl MaskQueue {
    fn add(&mut self, node: NodeId, change: Change) {
        *self.pending.entry(node).or_default() |= change;
    }

    fn take(&mut self) -> FxHashMap<NodeId, Change> {
        std::mem::take(&mut self.pending)
    }
}

/// Drains a bounded ring into a vector (drain order: oldest first).
fn drain_ring<T>(ring: &mut std::collections::VecDeque<T>) -> Vec<T> {
    ring.drain(..).collect()
}

/// The provenance-tagged write handle ([`Flux::writer`]): event handlers
/// capture this instead of the flux instance. The write path itself is one
/// capture-lock + the signal set.
#[derive(Debug, Clone)]
pub struct FluxWriter {
    capture: Arc<Mutex<CausalCapture>>,
}

impl FluxWriter {
    /// Writes `value` into `signal` with a declared origin (§13.3 "who
    /// wrote this signal").
    pub fn write<T>(
        &self,
        signal: &RwSignal<T>,
        value: T,
        origin: impl Into<std::borrow::Cow<'static, str>>,
    ) where
        T: Send + Sync + 'static,
    {
        let key = signal_key_of(signal);
        if let Ok(mut capture) = self.capture.lock()
            && capture.enabled
        {
            capture.record_write(key, origin.into());
        }
        signal.set(value);
    }
}

/// Test-only: serializes access to reactive_graph's process-global `BATCH`
/// state.
///
/// Immediate effects flush per batch scope through a global; parallel test
/// threads that write signals would otherwise observe each other's batch
/// windows. Any test whose assertions depend on effect timing must hold this
/// guard. Not for runtime use.
#[doc(hidden)]
pub fn batch_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().expect("batch serializer")
}

/// The reactive runtime: signals, memos, and the mask bridge.
///
/// Cloned handles (signals) keep working for the lifetime of the process; the
/// queue, counters, and owner live here and are drained by the pass pipeline.
#[derive(Debug)]
pub struct Flux {
    queue: Arc<Mutex<MaskQueue>>,
    counters: Arc<FluxCounters>,
    capture: Arc<Mutex<CausalCapture>>,
    owner: Owner,
    effects: Vec<ImmediateEffect>,
}

impl Default for Flux {
    fn default() -> Self {
        Self::new()
    }
}

impl Flux {
    /// Creates the framework's reactive root owner.
    #[must_use]
    pub fn new() -> Self {
        let owner = Owner::new();
        Self {
            queue: Arc::new(Mutex::new(MaskQueue::default())),
            counters: Arc::new(FluxCounters::default()),
            capture: Arc::new(Mutex::new(CausalCapture::default())),
            owner,
            effects: Vec::new(),
        }
    }

    /// Enables the dev-mode causal capture (§13.3). Off by default; when on,
    /// every binding registration and effect rerun lands in the log the
    /// pipeline folds into the frame summary (ADR-0010).
    pub fn set_causal_capture(&self, on: bool) {
        if let Ok(mut capture) = self.capture.lock() {
            capture.enabled = on;
        }
    }

    /// Whether the causal capture is on.
    #[must_use]
    pub fn causal_capture_enabled(&self) -> bool {
        self.capture.lock().is_ok_and(|capture| capture.enabled)
    }

    /// Writes `value` into `signal` with a declared origin — the write
    /// provenance the Causality inspector reports (§13.3 "who wrote this
    /// signal"). A plain `signal.set(..)` bypasses provenance (recorded only
    /// when the write flows through this helper with capture on).
    pub fn write<T>(
        &self,
        signal: &RwSignal<T>,
        value: T,
        origin: impl Into<std::borrow::Cow<'static, str>>,
    ) where
        T: Send + Sync + 'static,
    {
        self.writer().write(signal, value, origin);
    }

    /// A thread-independent write handle: the provenance-tagged write path
    /// without holding the flux (event handlers capture this, not the
    /// flux instance). One clone of the capture arc — cheap, `Send`.
    #[must_use]
    pub fn writer(&self) -> FluxWriter {
        FluxWriter {
            capture: Arc::clone(&self.capture),
        }
    }

    /// The binding table: every registered signal→node edge as
    /// `(binding, signal, node)` — the "who subscribes" answer (§13.3).
    ///
    /// # Panics
    ///
    /// If the capture lock is poisoned — a panicked writer means the
    /// frame is corrupt either way (the render binding's own policy).
    #[must_use]
    pub fn edge_table(&self) -> Vec<(BindingId, SignalKey, NodeId)> {
        self.capture
            .lock()
            .map(|capture| {
                let mut edges: Vec<(BindingId, SignalKey, NodeId)> = capture
                    .edges
                    .iter()
                    .map(|(binding, (signal, node))| (*binding, *signal, *node))
                    .collect();
                edges.sort_unstable_by_key(|(binding, _, _)| *binding);
                edges
            })
            .expect("flux capture lock")
    }

    /// Drains the pending provenance-tagged writes (the pipeline's frame
    /// summary source).
    ///
    /// # Panics
    ///
    /// If the capture lock is poisoned — a panicked writer means the
    /// frame is corrupt either way (the render binding's own policy).
    #[must_use]
    pub fn take_signal_writes(&self) -> Vec<SignalWrite> {
        self.capture
            .lock()
            .map(|mut capture| drain_ring(&mut capture.writes))
            .expect("flux capture lock")
    }

    /// Drains the pending edge activations (the pipeline's frame summary
    /// source).
    ///
    /// # Panics
    ///
    /// If the capture lock is poisoned — a panicked writer means the
    /// frame is corrupt either way (the render binding's own policy).
    #[must_use]
    pub fn take_edge_activations(&self) -> Vec<EdgeActivation> {
        self.capture
            .lock()
            .map(|mut capture| drain_ring(&mut capture.activations))
            .expect("flux capture lock")
    }

    /// Creates a signal.
    pub fn signal<T: Send + Sync + 'static>(&self, value: T) -> RwSignal<T> {
        self.owner.with(|| RwSignal::new(value))
    }

    /// Creates a memo with automatic dependency tracking.
    pub fn memo<T, F>(&self, fun: F) -> Memo<T>
    where
        T: Send + Sync + PartialEq + 'static,
        F: Fn(Option<&T>) -> T + Send + Sync + 'static,
    {
        self.owner.with(|| Memo::new(fun))
    }

    /// Runs `f` as one write batch: nested-safe, coalesced effect replay at
    /// scope end. N synchronous writes inside become **one** effect rerun.
    pub fn batch<R>(&self, f: impl FnOnce() -> R) -> R {
        batch(f)
    }

    /// The batch primitive, callable from anywhere (event handlers capture
    /// signal handles, not the flux instance — batch is a global scope).
    pub fn batch_of<R>(f: impl FnOnce() -> R) -> R {
        batch(f)
    }

    /// The mask bridge: subscribes `node` to `signal`.
    ///
    /// On every effect rerun (initial run included) `on_change` inspects the
    /// new value and returns the masks to queue for `node`. The closure must
    /// not touch widgets — it computes masks, nothing else (rule 1).
    pub fn bind<T, F>(&mut self, signal: RwSignal<T>, node: NodeId, on_change: F)
    where
        T: Send + Sync + Clone + 'static,
        F: Fn(&T) -> Change + Send + Sync + 'static,
    {
        let queue = Arc::clone(&self.queue);
        let counters = Arc::clone(&self.counters);
        let capture = Arc::clone(&self.capture);
        // Register the edge up front (the "who subscribes" table); the id is
        // captured by the effect so every rerun records its activation.
        let binding = self.capture.lock().map_or_else(
            |_| 0_u64,
            |mut capture| {
                if capture.enabled {
                    capture.register(signal_key_of(&signal), node)
                } else {
                    0
                }
            },
        );
        let signal_key = signal_key_of(&signal);
        let effect = self.owner.with(|| {
            ImmediateEffect::new(move || {
                let value = signal.get();
                let change = on_change(&value);
                if let Ok(mut capture) = capture.lock()
                    && capture.enabled
                {
                    capture.record_activation(binding, signal_key, node, change);
                }
                if let Ok(mut q) = queue.lock() {
                    q.add(node, change);
                }
                counters.bump();
            })
        });
        self.effects.push(effect);
    }

    /// Binds a signal to a [`RenderBinding`] slot and queues `mask` on change —
    /// the per-dynamic-slot render binding (§5.1, `RenderEffect` semantics).
    ///
    /// # Panics
    ///
    /// When the effect body runs, if the binding's lock is poisoned — a
    /// panicked writer means the frame is corrupt either way.
    pub fn bind_value<T, U, F>(
        &mut self,
        signal: RwSignal<T>,
        node: NodeId,
        binding: &RenderBinding<U>,
        mask: Change,
        map: F,
    ) where
        T: Send + Sync + Clone + 'static,
        U: Send + Sync + 'static,
        F: Fn(&T) -> U + Send + Sync + 'static,
    {
        let binding = Arc::clone(&binding.0);
        self.bind(signal, node, move |value| {
            *binding.write().expect("render binding lock") = map(value);
            mask
        });
    }

    /// Drains the pending mask queue (pass 4's flux side).
    ///
    /// # Panics
    ///
    /// If the mask queue's lock is poisoned — a panicked writer means
    /// the frame is corrupt either way, and a drain that silently
    /// answered empty would drop every pending mask: the UI freezes
    /// without a word (the render binding's own policy, applied where
    /// the frame's correctness rides the drain).
    #[must_use]
    pub fn take_pending_masks(&self) -> FxHashMap<NodeId, Change> {
        self.queue
            .lock()
            .map(|mut q| q.take())
            .expect("flux mask queue lock")
    }

    /// The counters behind the scheduling-contract tests.
    #[must_use]
    pub const fn counters(&self) -> &Arc<FluxCounters> {
        &self.counters
    }
}

/// A shared, live value written by effects and read by widgets — the render
/// slot a dynamic piece of UI is bound to.
///
/// Widgets read at paint time; effects write at drain time; neither touches
/// the other's borrow. This is what makes "reactivity is upstream of
/// invalidation" structural.
#[derive(Debug)]
pub struct RenderBinding<T>(Arc<RwLock<T>>);

impl<T: Send + Sync + 'static> RenderBinding<T> {
    /// Creates a binding with an initial value.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }

    /// Overwrites the value (effect side).
    ///
    /// # Panics
    ///
    /// If the binding's lock is poisoned — a panicked writer means the frame
    /// is corrupt either way.
    pub fn set(&self, value: T) {
        *self.0.write().expect("render binding lock") = value;
    }

    /// Reads the value (widget side).
    ///
    /// # Panics
    ///
    /// Panics if the lock is poisoned.
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        self.0.read().expect("render binding lock")
    }

    /// Runs `f` with shared access to the value.
    ///
    /// # Panics
    ///
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self.read();
        f(&guard)
    }
}

impl<T> Clone for RenderBinding<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::{Flux, RenderBinding};
    use crate::mask::Change;
    use reactive_graph::traits::{Get, Set};

    #[test]
    fn bind_runs_once_immediately_and_queues_initial_masks() {
        let _guard = super::batch_test_lock();
        let mut flux = Flux::new();
        let node = crate::arena::NodeId::INVALID;
        let signal = flux.signal(0i32);
        flux.bind(signal, node, |v: &i32| {
            let _ = v;
            Change::PAINT
        });

        assert_eq!(
            flux.counters().effect_reruns(),
            1,
            "initial synchronous run"
        );
        let pending = flux.take_pending_masks();
        assert_eq!(pending.get(&node), Some(&Change::PAINT));
        assert!(
            flux.take_pending_masks().is_empty(),
            "drain empties the queue"
        );
    }

    #[test]
    fn unbatched_writes_rerun_per_write_but_one_tick_consumes_them() {
        let _guard = super::batch_test_lock();
        let mut flux = Flux::new();
        let node = crate::arena::NodeId::INVALID;
        let signal = flux.signal(0i32);
        flux.bind(signal, node, |v: &i32| {
            if *v > 0 {
                Change::PAINT
            } else {
                Change::empty()
            }
        });

        for i in 1..=5 {
            signal.set(i);
        }
        assert_eq!(
            flux.counters().effect_reruns(),
            6,
            "initial + one per write"
        );
        let pending = flux.take_pending_masks();
        assert_eq!(
            pending.get(&node),
            Some(&Change::PAINT),
            "masks merge, never multiply"
        );
    }

    #[test]
    fn one_write_batch_coalesces_into_one_effect_rerun() {
        let _guard = super::batch_test_lock();
        // The M0 kill-switch (ADR-6): 5 synchronous writes inside one batch
        // must produce exactly one effect rerun — never a rebuild storm.
        let mut flux = Flux::new();
        let node = crate::arena::NodeId::INVALID;
        let signal = flux.signal(0i32);
        flux.bind(signal, node, |v: &i32| {
            let _ = v;
            Change::LAYOUT.union(Change::PAINT)
        });

        flux.counters().reset();
        flux.batch(|| {
            for i in 1..=5 {
                signal.set(i);
            }
        });

        assert_eq!(
            flux.counters().effect_reruns(),
            1,
            "1-bit coalescing: N writes, 1 rerun"
        );
    }

    #[test]
    fn bind_value_writes_the_render_slot_and_requests_masks() {
        let _guard = super::batch_test_lock();
        let mut flux = Flux::new();
        let node = crate::arena::NodeId::INVALID;
        let signal = flux.signal("a".to_string());
        let binding = RenderBinding::new(String::new());
        let mask = Change::LAYOUT.union(Change::PAINT);
        flux.bind_value(signal, node, &binding, mask, |v: &String| v.to_uppercase());

        signal.set("hello".to_string());
        let pending = flux.take_pending_masks();
        assert_eq!(pending.get(&node), Some(&mask));
        assert_eq!(binding.with(|v| v.to_uppercase()), "HELLO");
    }

    #[test]
    fn memo_tracks_its_sources() {
        let _guard = super::batch_test_lock();
        let flux = Flux::new();
        let count = flux.signal(2i32);
        let doubled = flux.memo(move |_| count.get() * 2);
        assert_eq!(doubled.get(), 4);
        count.set(10);
        assert_eq!(doubled.get(), 20);
    }

    #[test]
    fn nested_batches_keep_masks_merged_into_one_queue_entry() {
        let _guard = super::batch_test_lock();
        // reactive_graph flushes immediate effects per batch scope (documented
        // "Unless batch is used" semantics); the frame-level contract does not
        // depend on that: whatever the rerun count, masks merge into one queue
        // entry, so one tick consumes them as ONE layout+paint cycle.
        let mut flux = Flux::new();
        let node = crate::arena::NodeId::INVALID;
        let signal = flux.signal(0i32);
        flux.bind(signal, node, |_: &i32| Change::PAINT);
        flux.counters().reset();

        flux.batch(|| {
            signal.set(1);
            flux.batch(|| {
                signal.set(2);
                signal.set(3);
            });
            signal.set(4);
        });

        assert!(flux.counters().effect_reruns() >= 1, "at least one rerun");
        assert!(
            flux.counters().effect_reruns() <= 3,
            "coalesced per batch scope, never per write"
        );
        let pending = flux.take_pending_masks();
        assert_eq!(pending.len(), 1, "all reruns merge into one node entry");
        assert_eq!(pending.get(&node), Some(&Change::PAINT));
    }
}
