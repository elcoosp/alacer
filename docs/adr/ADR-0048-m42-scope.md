# ADR-0048: M42 scope — the clipboard read names its facts

- **Status:** Accepted (2026-11-12)
- **Implements:** ADR-0047's third re-open rule — "a consumer needs the
  read side's facts (a refused read's species — the `None` that
  collapsed an empty clipboard, a non-text payload, and an unopenable
  clipboard) → grow the read door beside the write door (the same
  provided-method discipline), priced by a real reader" — cashed as the
  vocabulary growth the additive rule names: "grow the event vocabulary
  beside the wire's kinds, priced by a real reader — never a second log"
- **Companion:** ADR-0020 (the clipboard seam whose read contract this
  grows), ADR-0045 (the clipboard trail whose record carries the
  read's species), ADR-0047 (the engine's write facts this door mirrors
  on the read side), ADR-0049 (the value attribution that shipped
  beside this milestone), ADR-0037 (the provided-method discipline's
  origin at the keymap tier), ADR-0010 (the protocol's additive
  versioning — the new field is compatible at version 1), architecture
  §12 (the platform integration), §18 (the M42 roadmap row)

## Decision

- **One provided door, beside the `Option` contract of record.**
  `alacer-core/src/clipboard.rs`: `ClipboardSeam::read_reported` — a
  *provided* method answering [`ClipboardReadFact`] (the M37
  provided-method discipline: the provided default maps the `Option`
  door — a payload answers `Delivered`, an absence the `Option` cannot
  name answers [`ClipboardReadRefusal::Unnamed`], the absence honest —
  and a real engine overrides the door and names its species). The
  `read_text` door stays the contract of record (the M35
  `dispatch_installed` rule): a read through either door is the same
  read, and a refusal through either inserts nothing (ADR-0020's rule
  stands). The widget's paste funnel rides the reported door; the
  copy's and the cut's write-side round trips never asked a read
  question.
- **The species are the engine's own answers.**
  [`ClipboardReadRefusal`]: `unnamed` (the provided default's honest
  absence — the engine answered only its `Option`), `unsupported` (no
  clipboard subsystem at all, or one this process cannot open: a
  headless box, a sandboxed session), `empty` (the clipboard carries
  nothing the OS counts as content), `non_text` (the clipboard carries
  content, but not text — an image, a custom format), `os_error` (the
  platform answered with an error the engine cannot classify — its
  description traced engine-side, never guessed onto the wire). The
  real engine rides arboard's read answers — and where arboard's own
  answer collapses two facts (`ContentNotAvailable` covers the empty
  clipboard *and* the non-text payload), the engine asks the OS its
  second question: an image read that lands names the `non_text`
  payload, a second `ContentNotAvailable` is the honest `empty`, any
  other answer stays `os_error`. Each species is the OS's own answer
  to a question the engine asked — never a guess (ADR-0020's rule
  stands), and the second question runs only when the first refused: a
  delivered text read never pays for it.
- **The memory engine names its own emptiness.** The harness's
  in-memory store is a real engine whose absence is *known* emptiness —
  the store knows the payload it holds — so it overrides the door and
  names `Empty` instead of riding the provided default's honest
  `unnamed` absence. Headless CI's paste refusals carry the species
  too, and the m42 suite's field-refused beat (a payload that
  delivered but sanitized to nothing) rides the read's absence — the
  two halves of the M39 collapse, named apart on the record.
- **The record carries the fact beside the verdict.**
  [`crate::inspect::ClipboardRecord`] grows one additive field:
  `read: Option<ClipboardReadRefusal>` — `Some(species)` when the
  paste refused for the *read*, `None` on a delivery, on the field's
  own refusal (a payload that sanitized to nothing — the read
  delivered), and on every write-side round trip. The verdict says
  *what the field did*; the read fact says *what the clipboard
  answered*. The trail's shape is untouched — the same 256-bounded
  queue, the same tick's start drain, one branch per chord while
  nobody listens.
- **The wire grows one field, additively at version 1.**
  `alacer-inspect/src/wire.rs`: the `clipboard` event grows `read`
  (the species' `snake_case` tag, `null` where the fact is absent) —
  the additive field the versioning rule admits at version 1, clients
  that ignore unknown fields unaffected. The recorder's ring carries
  the record verbatim (no new ring — the same 512 slots), the embedded
  client renders the species beside the paste's refusal, and
  `docs/inspector-protocol.md` documents the field.
- **The corpus stands at 63.** The door is a chord-path read and a
  dev-mode record — no new hot loop (the coordinated rule's no-growth
  arm, the M31/M33/M36/M37/M39/M40/M41 rule); the published
  `editing/route_100_clipboard_ops_1k` route measures the same round
  trips it always measured, and the disabled trail costs one branch
  per chord (the A7 idle contract, pinned by the exit suite's idle
  test).
- **The living documents re-agree at the cut.** The README's status
  block, the ADR count (fifty), the quickstart, and the example table;
  the catalog; the register; the roadmap; the m-suite pins moving
  M41 → M43 and 48 → 50 across the chain; the protocol document's
  `read` field; the CHANGELOG.

## Consequences

- The workspace stays at one milestone per minor cut: the next release
  (0.18.0) carries M42 beside M43.
- The corpus is 63 cases; the m-suite's corpus pin stands (the
  no-growth arm).
- A status line, an inspector client, or a test answers "did the paste
  refuse for the read, or for the field" from the log: the empty
  clipboard's `empty`, the screenshot's `non_text`, the sandbox's
  `unsupported`, the bool-only engine's honest `unnamed` — the
  read-side diagnostics a devtools surface can route on, never a
  second log, never a guess.
- A custom engine that overrides nothing keeps compiling and
  answering: the provided door maps its `Option`, the record honest
  about the absence — the additive rule's whole point (ADR-0010).

## Kill criteria (re-open rules)

- A consumer needs the read side's *payload* on the log → grow it as
  an explicit, opt-in consumer-side capture beside the seam — never a
  default on a devtools surface a secret can reach (the
  count-not-payload rule stands).
- A consumer needs the OS's *change notifications* (the clipboard
  changing under the app — another writer) → grow the engine contract
  beside the write door (a listen tier, priced by a real engine's
  notification protocol) — never a poll guessed onto a devtools
  surface (ADR-0020's rule stands).
- A consumer needs the unclassifiable `os_error` split (the OS's own
  error taxonomy on the wire) → grow the species additively
  (ADR-0010's rule), priced by a reader — never a description string
  guessed past the engine's answer.
