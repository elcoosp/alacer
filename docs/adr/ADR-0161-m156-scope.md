# ADR-0161: M156 scope — the action tier: the tree says the asks each door answers

- **Status:** Accepted (2026-11-28)
- **Implements:** `docs/architecture.md` §18, row M156 — per-action capability advertisement
- **Cashes the deferral of:** ADR-0016 §3, ADR-0017 §3, ADR-0023 §3 ("advertising is a widget-canon sweep once more actions exist")
- **Companion suite:** `crates/alacer-testing/tests/m156_exit_criteria.rs`
- **Companion example:** `crates/alacer/examples/a11y_actions.rs`

## Decision

Three scope ADRs carried the same deferral in their limits: *no per-action
capability advertisement* — "refusal is honest and observable today;
advertising is a widget-canon sweep once more actions exist" (ADR-0016 §3,
ADR-0017 §3), and ADR-0023 §3 listed `supports_*` among the words that
"ride M18+ with its own corpus cycle". The trigger had long been met: by
M155 fourteen widgets answered ten kinds of ask through
`Widget::on_a11y_action`, and the platform boundary mapped each of them.
What the tree never said was *which* asks a given control answers.

The gap was not cosmetic. AccessKit's consumer (0.39.2) derives
`is_clickable`, `supports_increment` and `supports_decrement` from the
actions a provider declares explicitly and "not assume anything beyond
what the provider has explicitly told us" — a deliberate departure from
Chromium's role-based guess. Alacer's sink declared none, so on every
platform adapter each node read as un-actionable: the door answered, the
word never said it would, and an AT had no reason to ask.

M156 ships the word, in the house shape (the M94 / M121 pattern):

- **The vocabulary** (`crates/alacer-core/src/widget.rs`): `A11yActions`, a
  ten-kind bitset — `DEFAULT_PRESS`, `INCREMENT`, `DECREMENT`, the four
  `SCROLL_*` travel asks, `SET_VALUE`, `REPLACE_SELECTED_TEXT`,
  `SET_TEXT_SELECTION` — one bit per kind the way `A11yAction` carries one
  variant per kind; `A11yAction::advertises()` maps an ask onto its kind
  (the word names the kind, never the payload). Core never learns
  "accesskit" (ADR-9, `scripts/check_dag.py`).
- **The seam**: `Widget::a11y_actions()`, defaulted to the empty word — a
  widget that overrides nothing reads exactly as it did before the tier.
  Pass 9 fills `TreeNode.actions` from it, so the word rides the same
  node-only delta as every other property (§11.1 item 4).
- **The sink** (`crates/alacer-a11y/src/lib.rs`): `advertised_actions`
  maps each kind onto its AccessKit `Action`, and `access_node` calls
  `add_action` per kind. The table is the exact inverse of the platform's
  `map_action_request` — pinned both ways in the platform crate's tests.
- **The canon sweep** (eleven widgets): the seven press controls say
  `DEFAULT_PRESS`; `Slider` and `SpinButton` the value tier's three;
  `TextInput` the text tier's three; `List` the scroll tier's six.
  `Progress`, `Meter`, `Canvas` and the plain labels inherit the absence.

The word is **state-aware**, and the invariant is the suite's first test:
a kind is advertised if and only if `on_a11y_action` answers `true` for a
well-formed ask of that kind. An unavailable control advertises nothing (a
closed door is no capability — the M22 rule reaching the word); a read-only
field advertises the selection ask alone (the M105 rule reaching the word:
the two writes are refused on the declared word, the selection still
answers). The availability and read-only doors move the word live through
the mutate seam, the `PAINT | A11Y` request paying the re-announcement.

**The price.** The word prices the word and nothing else: the doors answer
exactly as they always did. Its ceiling is published in the corpus —
`a11y/full_rebuild_1k_actions`, the 1,000-node tree with *every* node
advertising all ten asks (a real tree mixes words), against
`a11y/full_rebuild_1k`: 1.15× on the recorded host, gated by the
`a11y_actions_overhead` ratio budget ≤ 1.25× (the corpus growing
207 → 208 by the coordinated rule, ADR-0011).

## Consequences

- An AT now reads each Alacer control as actionable for exactly the asks
  its door answers, and for none it will refuse.
- A new ask kind is a coordinated change by construction: the bitset, the
  `advertises` arm, the sink table and the platform mapping each fail to
  compile or fail a test without the others.
- A widget whose door answers an ask *must* override `a11y_actions`; the
  ask-for-ask probe in the m156 suite is the pattern to extend, not a
  lint — a widget that forgets reads as un-actionable exactly as before.
- **Not done here.** No new action kinds; no payload in the word (the
  scroll unit and the value ride the request); no inspector wire field for
  the word (the M121 precedent: the wire mirrors structure, not tier
  words); `List`'s `no_scrollable_content` refusal still rides the scroll
  log rather than the word (M50, ADR-0056 — the scroll asks answer `true`
  at the door and are priced by the scroll funnel).
- **UNVERIFIED at strict platform adapters** (no AT was run for this cut):
  the consumer's derivation is pinned by reading its source and by the
  sink's `supports_action` tests, not by a live screen reader session.
