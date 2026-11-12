//! Keymap-dispatch benchmarks (§10.3, ADR-0041): one thousand key events
//! through the installed seam against a table of one hundred bindings,
//! alternating a miss (the unbound chord every unbound key carries) with
//! a hit (the mid-table binding the shortcut fires). The published
//! `keymap/dispatch_100_mixed_1k` baseline keeps the seam honest as the
//! table grows: the walk is a linear scan over owned bindings — a chord
//! compare per row, one `Rc` call per hit — never a re-materialization,
//! and the funnel's consult rides the same thread-local read every other
//! seam answers through.

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::keymap::{Chord, Keymap, install};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::rc::Rc;

const BINDINGS: usize = 100;
const EVENTS: usize = 1_000;

/// One table chord, by index — the single source both the table builder
/// and the hit event read, so the hit is always exactly binding
/// `BINDINGS / 2`'s shape. The letter advances once per modifier group
/// (`index / 4`), so the first `26 × 4` indices are all distinct
/// `(letter, modifier)` chords — the table carries `BINDINGS` bindings,
/// no rebind collapsing any. The letters ride a byte lookup — no cast
/// on the index path, the seat the saturating helpers take in `num`.
fn binding_chord(index: usize) -> Chord {
    const LETTER_BYTES: [u8; 26] = *b"abcdefghijklmnopqrstuvwxyz";
    let letter = char::from(LETTER_BYTES[(index / 4) % LETTER_BYTES.len()]);
    match index % 4 {
        0 => Chord::new(Key::Char(letter)),
        1 => Chord::new(Key::Char(letter)).control(),
        2 => Chord::new(Key::Char(letter)).alt(),
        _ => Chord::new(Key::Char(letter)).shift(),
    }
}

/// The bench's own table: `BINDINGS` chords over the lowercase letters
/// cycled with the three accelerator modifiers — every chord distinct,
/// the table order stable, the actions empty (the dispatch walk is the
/// cost under test, not the work behind it).
fn bench_table() -> Keymap {
    let mut keymap = Keymap::new();
    for index in 0..BINDINGS {
        keymap.bind(binding_chord(index), "bench", || {});
    }
    keymap
}

/// The mixed event script: odd events miss (the `!` chord with Meta held
/// matches nothing), even events hit the mid-table binding — the miss/hit
/// alternation the funnel answers on a window where shortcuts and typing
/// interleave.
fn mixed_events() -> Vec<KeyEvent> {
    let hit = KeyEvent {
        key: *binding_chord(BINDINGS / 2).key(),
        state: KeyState::Pressed,
        modifiers: binding_chord(BINDINGS / 2).modifiers(),
        text: None,
    };
    let miss = KeyEvent {
        key: Key::Char('!'),
        state: KeyState::Pressed,
        modifiers: Modifiers {
            meta: true,
            ..Modifiers::default()
        },
        text: None,
    };
    (0..EVENTS)
        .map(|index| {
            if index % 2 == 0 {
                miss.clone()
            } else {
                hit.clone()
            }
        })
        .collect()
}

fn bench_keymap_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("keymap");
    group.throughput(Throughput::Elements(EVENTS as u64));

    let keymap = bench_table();
    assert_eq!(keymap.len(), BINDINGS, "the bench table is whole");
    install(Rc::new(keymap));

    let events = mixed_events();
    group.bench_function("dispatch_100_mixed_1k", |b| {
        b.iter(|| {
            let mut consumed = 0_u32;
            for event in &events {
                // The funnel's own route: the total read the dispatch
                // contract consults (no seam installed answers false;
                // this thread's seam answers the script). The answers
                // are consumed, not discarded — the walk cannot be
                // optimized out from under the measurement.
                if black_box(alacer_core::keymap::dispatch_installed(black_box(event))) {
                    consumed += 1;
                }
            }
            black_box(consumed);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_keymap_dispatch);
criterion_main!(benches);
