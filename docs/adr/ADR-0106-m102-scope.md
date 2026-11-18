# ADR-0106 — M102 scope: the reveal tier

Milestone: **M102** (this cut). Parent contract: the secure tier's own
re-open rule, pre-registered verbatim in ADR-0104 — a consumer needs
the *revealed* word (a "show password" toggle) → AccessKit's own
contract names the mapping ("when the password is revealed, use
`Role::TextInput` instead"): the flag grows the revealed state beside
the constructor's, the sink's mapping one branch beside the one it
has.

## The ask

Since M100 a secured field's tree answer is `PasswordInput` and a
masked value, and the constructor word was the whole story — a "show
password" toggle had no word. The consumer's toggle flipped the
pixels and the tree kept the M100 answer: the AT read `PasswordInput`
and the mask while the user read the secret on screen — the declared
state the tree could not follow, the M94 pattern at the reveal tier.

## The shape

- **The trait door** (`Widget::secure_revealed`): the provided method
  beside `secure_entry()` — a compound word, meaningful only where the
  secure word holds: a field that answers `secure_entry()` `false`
  answers `false` here too, whatever its internal toggle carries (the
  plain field's value was always the tree's read, never a secret to
  reveal). The honest absence, never a munged half (the ADR-0087
  presence rule at the reveal tier).
- **The producer** (`TextInput`): the flag beside `purpose` — the
  constructor word `revealed()` names the initial state (a field
  mounted revealed reads as the ordinary editable from the first
  frame), and `set_revealed` is the app's live door: the toggle moved
  through the mutate seam, the caller's `PAINT | A11Y` request paying
  the re-announcement (the app-door shape every app-side write rides —
  the M56 door's own shape, priced by the caller that knows the frame).
- **The snapshot carries it** (`TreeNode::secure_revealed`): pass 9
  resolves the flag beside the secure word it already resolved — and
  the value the tree resolves follows it: the mask the secure word
  prices (one masking character per code point, ADR-0104's rule)
  yields to the revealed state, the field's ordinary value riding the
  tree again. The pixels and the tree tell one truth — AccessKit's
  own contract names the role half ("when the password is revealed,
  use `Role::TextInput` instead"); the mask half is the same contract
  at the value tier (the masking requirement is `PasswordInput`'s, and
  the field no longer is one).
- **The sink maps** (`AccessKitSink::access_node`): the
  `PasswordInput` role only while the field is secured *and not
  revealed* — the one branch beside the one it has, exactly the
  mapping ADR-0104 pre-registered. A revealed field reads as the
  ordinary editable word; a field that revealed nothing answers
  exactly the word M100 mapped.
- **No patch-tier growth**: the revealed state is runtime state the
  app's door moves, not a declarable word — the DSL's `secure` word
  (parsed since the M0 registry) builds the same constructor, and the
  `revealed()` builder names the initial state through the same
  element. The way M100 priced the tier: the declaration is the
  constructor's; the toggle is the app's.
- **The bench** (`secure_revealed/snapshot_1k_revealed`): the
  1,000-node revealed field arena against the same 1,000-node secured
  arena masked (`secure/snapshot_1k_secured`, the M100 case) — the M95
  pricing rule (the tier's cost measured against the same corpus, one
  fact richer). The corpus grows 113 → 114 by the coordinated rule,
  the `secure_revealed_snapshot_overhead` ratio budget (≤ 1.25×)
  holding the word's cost at the measured class: the flag and the
  role branch the node answers beside the M100 masked copy's own. The
  m-suite pins re-agree at 114.
- `examples/revealed.rs` drives the beats from the consumer side —
  the masked field beside the revealed one, the hide restoring the
  M100 answer, the reveal yielding the mask to the value, the status
  line mirroring the words the tree says.

## Re-open rules (pre-registered)

- a consumer needs the toggle on other editables (a secure note field,
  a custom secret entry) → the trait door already answers them (the
  compound word is not input-shaped beyond the secure gate), the
  producers joining without a seam change — ADR-0104's second re-open
  rule at the reveal tier.
- a consumer needs the AT to *drive* the toggle (an `AT action`
  flipping the revealed state) → the action vocabulary prices itself
  when a screen reader names the ask (the ADR-0046 kill rule); today
  the AT's set-value writes the value, and no platform AT names a
  reveal ask.
- a consumer needs the *session-scoped* reveal (the field reverting on
  focus loss, the DOM's autocomplete shape) → the reversion policy is
  the app's story (the door answers, the app decides when to call it),
  never a framework guess about the field's purpose.
- a consumer needs the mask's strength meter reading the revealed
  value → the composition is the consumer's story; the tree carries
  the value the field holds, ADR-0104's third re-open rule stands
  unchanged.
