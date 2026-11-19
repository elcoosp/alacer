# ADR-0125: M121 scope — the shortcut tier: the chord the keymap rides, said by the tree

- **Status:** Accepted (2026-11-19)
- **Implements:** the word every menu-and-toolbar vocabulary speaks
  about *how a control is invoked from the keyboard* — "Save —
  Control+S" — and the tree never carried for the control the AT walks
  to. Through M120 the snapshot said the declared live region
  (ADR-0124), the collection's place and span (ADR-0123), the
  routing outcome the harness answered as data (ADR-0122), the
  subtree's updating fact (ADR-0121), what a control is read as
  (ADR-0120), what a control opens (ADR-0119), the form facts, the
  collection facts — and the chord the keymap's own dispatch table
  rides (M35, ADR-0041; the consult's name leg, M37, ADR-0043) was a
  fact the trail knew and the tree did not: the keymap named its
  bindings to the causal log, the platform menus carried their
  accelerators, and the control the screen reader announced said its
  name and never the way its keymap answers — the declared-but-
  unreported gap (the M94 pattern), cashed at the shortcut tier: "a
  consumer needs the AT to announce the chord beside the name → the
  tree carries AccessKit's `KeyboardShortcut` property (ARIA's
  `aria-keyshortcuts`), the string the app declared, the doors
  answering unchanged beside the word — the word prices the word and
  nothing else"
- **Companion:** ADR-0041/ADR-0043 (the keymap seam the word mirrors —
  the dispatch table and the consult's name leg are the framework's
  own chord machinery; the word says the chord the app named, it never
  binds a key: the keymap's dispatch is the dispatch it always was,
  and a word that swallowed a key would price a contract the tree does
  not own), ADR-0099 (the string-tier precedent the word rides — the
  property is a string, no kind enum to mirror, one branch with no
  translation), ADR-0087 (the presence rule at the tier: `None` the
  honest absence — a control that declared no chord announces none,
  never a guessed binding), ADR-2 (the one-currency discipline the
  word joins — the word rides the `A11Y` mask the snapshot pass
  already pays), ADR-9 (the DAG the change never touches),
  architecture §11 (the a11y contract), §17 (the register), §18 (the
  M121 roadmap row)

## Decision

- **One word, `None` the honest absence.**
  [`Widget::keyboard_shortcut`](crates/alacer-core/src/widget.rs)
  answers `Option<&str>` — the chord the control is invoked with, the
  spellings the app's own vocabulary ("Control+S", "Mod+P"; the
  property is a string and the tier translates nothing), `None` the
  honest absence (the ADR-0087 presence rule at the tier: a control
  that declared no chord announces none, never a guessed binding). The
  word prices the word and nothing else: a worded control's doors
  answer exactly as they always did — the chord names a fact, it walls
  nothing, the control's press and focus routes the widget's own, and
  the keymap seam's dispatch is the dispatch it always was (the word
  and the keymap are separate contracts — the trail leg names the
  binding a press *fired*, the tree leg says the chord the control
  *declared*; neither silences the other).
- **The producer is the actionable vocabulary — `Button`, the M114
  precedent's own shape.**
  [`Button::keyboard_shortcut`](crates/alacer-widgets/src/button.rs)
  — the builder naming the chord from the first frame (the toolbar
  save's "Control+S", the print dialog's "Control+Shift+P") — beside
  [`Button::set_keyboard_shortcut`](crates/alacer-widgets/src/button.rs),
  the app's live door through the mutate seam (`None` restoring the
  honest absence — the chord withdrawn; the write that lands where the
  word already is queues nothing, the M21 idle-frame property). The
  word is an actionable word: the plain widgets answer nothing (a
  label reports its text, a list its scroll — the chord is the
  control's own), so no other producer joins at the cut; other
  actionable producers join without a seam change (the M107 rule) the
  day the vocabulary needs them.
- **The snapshot carries it; the sink maps it.** `TreeNode::
  keyboard_shortcut` rides the pass-9 fill beside the words already
  there (`slot.widget().keyboard_shortcut()` the same walk every other
  property rides), and the sink's mapping answers with AccessKit's
  `KeyboardShortcut` property — the `aria-keyshortcuts` word every
  platform AT announces beside the name. No patch-tier growth (the
  word is runtime state the app's door moves, never a declarable DSL
  word), no second channel: the property rides the same node-only
  delta as every other property — a shortcut write re-announces
  exactly that node.
- **The living documents re-agree at M121 and 0.17.0.** The ADR count
  (a hundred and twenty-six), the architecture register (ADR-0125
  scopes M121), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `shortcut` row) all name the
  tier; the corpus grew 140 → 141 by the coordinated rule below, and
  the m-suite corpus pins re-agree at 141.

## Consequences

- **The corpus grew 140 → 141 by the coordinated rule — the M120
  precedent's own shape:** `shortcut/snapshot_1k_buttons_shortcut` —
  1,000 buttons, the worded arena's snapshot price (the chord cycling
  the platform spellings down the stream, the way a toolbar's buttons
  alternate) — against the current route's published bare arena
  (`current/snapshot_1k_buttons_bare`, the M114 case the tier reuses
  by the M111 precedent; the M95 pricing rule — the word is a button
  word, so the honest comparison is buttons against buttons, the
  M108/M120 denominator shape). The `shortcut_snapshot_overhead` ratio
  budget ≤ 1.25× (the word rides the snapshot the button rides, the
  tier's price the one `Option` the node carries beside it), measured
  1.22× at the cut. The m-suite pins re-agree at 141.
- **The AT announces what the keymap answers.** The word is the
  mapping platform ATs turn into "Save — Control+S" beside the name —
  beside the keymap seam the framework owns, a screen reader walking a
  toolbar reads the chord before it asks the app for the dispatch, and
  a control the keymap never names says nothing. The absence is the
  contract's other half: a control that claims no chord announces
  none, never a guessed binding, so a plain button does not start
  carrying a chord the app did not declare.
- **Consumers get the whole vocabulary for free.** The
  `examples/shortcut.rs` beat is the shape: the worded save and print
  under the word, the plain close and the plain label the honest
  absence between them, every door answering as it always did, the
  shortcut door moving the word — the plain export declaring its own
  chord, the save chord withdrawn, clear — the honest absence restored.
