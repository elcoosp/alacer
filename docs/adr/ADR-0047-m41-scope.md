# ADR-0047: M41 scope — the clipboard engine names its platform-side facts

- **Status:** Accepted (2026-11-12)
- **Implements:** ADR-0045's own re-open rule — "a consumer needs the
  clipboard's *platform-side* facts the widget's round trips cannot name
  (the engine's own refusal diagnostics, the OS's change notifications)
  → grow the seam's engine contract beside `ClipboardSeam`, priced by a
  real engine — never a guess the OS did not answer (ADR-0020's rule
  stands)" — cashed as the vocabulary growth the additive rule names:
  "grow the event vocabulary beside the wire's kinds, priced by a real
  reader — never a second log"
- **Companion:** ADR-0020 (the clipboard seam whose engine contract this
  grows), ADR-0045 (the clipboard trail whose record carries the
  engine's species), ADR-0043 (the provided-method discipline this door
  rides — the provided table answers, custom implementations default),
  ADR-0046 (the value trail that shipped beside this milestone), ADR-0010
  (the protocol's additive versioning — the new field is compatible at
  version 1), architecture §12 (the platform integration), §18 (the M41
  roadmap row)

## Decision

- **One provided door, beside the `bool` contract of record.**
  `alacer-core/src/clipboard.rs`: `ClipboardSeam::write_reported` — a
  *provided* method answering [`ClipboardWriteFact`] (the M37
  provided-method discipline: the provided default maps the `bool`
  door — an accepted write answers `accepted`, a refusal the `bool`
  cannot name answers [`ClipboardWriteRefusal::Unnamed`], the absence
  honest — and a real engine overrides the door and names its species).
  The `write_text` door stays the contract of record byte-for-byte (the
  M35 `dispatch_installed` rule): a write through either door is the
  same write, and a refusal through either deletes nothing (ADR-0020's
  rule stands). The widget's copy and cut funnels ride the reported
  door; the paste's read side never asked a write question.
- **The species are the engine's own answers.**
  [`ClipboardWriteRefusal`]: `unnamed` (the provided default's honest
  absence — the engine answered only its `bool`), `unsupported` (no
  clipboard subsystem at all: a headless box, a sandboxed session),
  `occupied` (another party held the clipboard while the write ran),
  `conversion` (the payload could not be converted to the platform's
  format), `os_error` (the platform answered with an error the engine
  cannot classify — its description traced engine-side, never guessed
  onto the wire). The real engine rides arboard's error kinds: the
  kinds the OS names map onto the species; the `#[non_exhaustive]`
  remainder stays `os_error` — the engine never classifies past the
  answer it got (ADR-0020's "never a guess the OS did not answer").
- **The record carries the fact beside the verdict.**
  [`crate::inspect::ClipboardRecord`] grows one additive field:
  `engine: Option<ClipboardWriteRefusal>` — `Some(species)` when the
  round trip's refusal was a refused write, `None` on a delivery and on
  every other refusal (a paste reads, a no-selection copy never asked
  the engine). The round trip says *what* moved; the engine fact says
  *which platform answered*. The trail's shape is untouched — the same
  256-bounded queue, the same tick's start drain, one branch per chord
  while nobody listens.
- **The wire grows one field, additively at version 1.**
  `alacer-inspect/src/wire.rs`: the `clipboard` event grows `engine`
  (the species' `snake_case` tag, `null` where the fact is absent) —
  the additive field the versioning rule admits at version 1, clients
  that ignore unknown fields unaffected. The recorder's ring carries
  the record verbatim (no new ring — the same 512 slots), the embedded
  client renders the species beside the refusal, and
  `docs/inspector-protocol.md` documents the field.
- **The corpus stands at 63.** The door is dev-mode and bounded — no
  new hot loop (the coordinated rule's no-growth arm, the
  M31/M33/M36/M37/M39/M40 rule); the published `editing/route_100_clipboard_ops_1k`
  route measures the same round trips it always measured, and the
  disabled trail costs one branch per chord (the A7 idle contract,
  pinned by the exit suite's idle test).
- **The living documents re-agree at M41.** The README's status block,
  the ADR count (forty-eight), the quickstart, and the example table
  (34 rows); the catalog; the register; the roadmap; the m-suite pins
  moving M40 → M41 and 47 → 48 across the chain; the protocol
  document's `engine` field; the CHANGELOG.

## Consequences

- The workspace stays at one milestone per minor cut: the next release
  (0.18.0) carries M41 beside M40.
- The corpus is 63 cases; the m-suite's corpus pin stands (the
  no-growth arm).
- A status line, an inspector client, or a test answers "which
  clipboard failed it" from the log: the kiosk's `unsupported`, the
  busy desktop's `occupied`, the unclassifiable `os_error` — the
  platform-side diagnostics a devtools surface can route on, never a
  second log, never a guess.
- A custom engine that overrides nothing keeps compiling and answering:
  the provided door maps its `bool`, the record honest about the
  absence — the additive rule's whole point (ADR-0010).

## Kill criteria (re-open rules)

- A consumer needs the OS's *change notifications* (the clipboard
  changing under the app — another writer) → grow the engine contract
  beside the write door (a listen tier, priced by a real engine's
  notification protocol) — never a poll guessed onto a devtools
  surface (ADR-0020's rule stands).
- A consumer needs the unclassifiable `os_error` split (the OS's own
  error taxonomy on the wire) → grow the species additively
  (ADR-0010's rule), priced by a reader — never a description string
  guessed past the engine's answer.
- A consumer needs the read side's facts (a refused read's species —
  the `None` that collapsed an empty clipboard, a non-text payload,
  and an unopenable clipboard) → grow the read door beside the write
  door (the same provided-method discipline), priced by a real reader.
