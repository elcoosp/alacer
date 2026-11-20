//! The glyph-atlas LRU's thrash bench (§9.4): the hot-working-set script
//! the policy resists, priced against the full-clear discipline it
//! replaces. The script — a hot set served every cycle beside a cold
//! stream that never returns — is the shape that wipes a full-clear at
//! every capacity bound, and the wipe's cost is the point: in the atlas
//! a miss re-uploads the glyph's coverage (`write_texture`), so the bench
//! prices a miss as an upload-shaped payload copy both arms pay. The LRU
//! keeps the hot set resident (its misses are the cold stream's alone);
//! the full-clear re-uploads the whole hot set at every clear. The
//! headless eviction-order contract lives in `cache.rs`'s tests; this
//! bench prices the two disciplines under the atlas's cost model.
//!
//! This bench rides *outside* the published corpus: the coordinated rule
//! grows the baseline at the next published cut, never at a crate bench's
//! (the m138 widget-bench rule's shape).

use alacer_text::cache::Lru;
use criterion::{Criterion, criterion_group, criterion_main};
use rustc_hash::FxHashMap;

/// The hot working set: the glyphs every frame re-requests.
const HOT: usize = 32;
/// The cold stream: the glyphs that flash once and never return.
const COLD: usize = 512;
/// The resident bound both disciplines hold — the shape where the hot set
/// fits comfortably and the stream is what overflows.
const BOUND: usize = HOT * 2;
/// One glyph coverage's upload payload — a large glyph's coverage (a
/// 128×128 CJK/emoji bitmap, 16 KB): the cost a miss re-pays in the
/// atlas, a `write_texture` shaped copy.
const UPLOAD_PAYLOAD: usize = 16 * 1024;

/// The upload a miss pays: one payload copy appended into the staging
/// area (the `write_texture` shape — a fresh bitmap's bytes riding to a
/// driver staging buffer, never the same hot 16 KB twice). The payload
/// rides `black_box` and varies by `seed`: an observable upload the
/// optimizer cannot fold into a constant.
fn upload(scratch: &mut Vec<u8>, seed: u8) {
    scratch.extend_from_slice(std::hint::black_box(&[seed; UPLOAD_PAYLOAD]));
}

/// The iteration's observable answer: the staging tail plus the policy's
/// ledger — the state the optimizer must keep real.
fn observed(scratch: &[u8], evictions: u64) -> u64 {
    let last = u64::from(*scratch.last().expect("an upload happened"));
    last + evictions
}

/// The LRU: the hot set stays resident, the misses are the cold stream's
/// alone, the evictions take the cold tail.
fn bench_thrash_lru(c: &mut Criterion) {
    c.bench_function("lru/thrash_lru_32", |b| {
        b.iter_batched(
            || Vec::with_capacity((BOUND + HOT) * UPLOAD_PAYLOAD),
            |mut scratch| {
                let mut book = Lru::new();
                let mut seed: u8 = 0;
                for hot in 0..HOT {
                    seed = seed.wrapping_add(1);
                    book.admit(hot);
                    upload(&mut scratch, seed);
                }
                for cold in HOT..HOT + COLD {
                    for hot in 0..HOT {
                        // A hit is a touch — no upload.
                        book.touch(&hot);
                    }
                    if !book.touch(&cold) {
                        // A miss re-uploads the glyph.
                        seed = seed.wrapping_add(1);
                        upload(&mut scratch, seed);
                        book.admit(cold);
                        while book.len() > BOUND {
                            let _ = book.evict_lru();
                        }
                    }
                }
                observed(&scratch, book.evictions())
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

/// The replaced discipline: a stamp map that full-clears at the bound —
/// the M0 atlas's policy — where every clear wipes the hot set and the
/// next cycle re-uploads all of it.
fn bench_thrash_full_clear(c: &mut Criterion) {
    c.bench_function("lru/thrash_full_clear_32", |b| {
        b.iter_batched(
            || Vec::with_capacity((BOUND + HOT) * UPLOAD_PAYLOAD),
            |mut scratch| {
                let mut stamps: FxHashMap<usize, u64> = FxHashMap::default();
                let mut tick: u64 = 0;
                let mut seed: u8 = 0;
                for hot in 0..HOT {
                    tick += 1;
                    seed = seed.wrapping_add(1);
                    stamps.insert(hot, tick);
                    upload(&mut scratch, seed);
                }
                for cold in HOT..HOT + COLD {
                    for hot in 0..HOT {
                        if let std::collections::hash_map::Entry::Vacant(slot) = stamps.entry(hot) {
                            // The clear wiped this hot glyph: it misses and
                            // re-uploads — every clear, the whole hot set.
                            seed = seed.wrapping_add(1);
                            upload(&mut scratch, seed);
                            tick += 1;
                            slot.insert(tick);
                        }
                    }
                    tick += 1;
                    seed = seed.wrapping_add(1);
                    stamps.insert(cold, tick);
                    upload(&mut scratch, seed);
                    if stamps.len() > BOUND {
                        stamps.clear();
                    }
                }
                observed(&scratch, 0)
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_thrash_lru, bench_thrash_full_clear);
criterion_main!(benches);
