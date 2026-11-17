# ADR-0104 — M100 scope: the secure tier

Milestone: **M100** (this cut). Parent contract: the IME contract's
request half (§10.3, ADR-7) and the a11y sink's M6 mapping standing
beside the fact the field has declared since M4 and the tree could
never say: the secret entry — the constructor word the platform's IME
already honors — AccessKit's `PasswordInput` role, the word every
platform AT announces ("secure text entry"), ARIA's `type="password"`.

## The ask

`TextInput::secure()` has named `ImePurpose::Secure` since M4: the
platform suppresses preedit echo and suggestion bars, the way a
password field behaves. The a11y tree read a plain text input — and
carried the field's value as the *raw secret*: the payload no trail
carries (the clipboard trail's rule, ADR-0045) riding the one surface
every assistive technology reads. The AT's user heard a bare field
where the sighted user saw dots; the tree held the secret the pixels
never drew. The declared-but-unreported gap, the M94 pattern at the
secure tier — the M95 rule (the word the boundary maps, the widget
answers) cashed at the secure tier.

## The shape

- **The trait door** (`Widget::secure_entry`): the provided method
  beside `numeric_value()` — `false` is the honest absence, and a
  widget that declared nothing is no secure field, never a guessed
  flag (the ADR-0087 presence rule at the secure tier). A non-input
  role answers nothing. `TextInput` overrides with the constructor's
  own word (`self.purpose == ImePurpose::Secure`); every other role
  inherits the default.
- **The snapshot rides twice** (`TreeNode::secure_entry` and the
  masked value): pass 9 resolves the flag beside the value it already
  resolved — and for a secured field the value the tree carries is the
  *mask*: one masking character per code point of the committed value,
  never the secret. AccessKit's own contract names the rule ("within a
  `PasswordInput`, this property must only contain masking
  characters"); the tree satisfies it instead of violating it, and the
  clipboard trail's payload rule (ADR-0045) lands at the tree tier.
  The mask's length tracks the value — the shape the AT reads, the
  secret itself riding nothing (the m100 suite pins both the word and
  the mask's tracking).
- **The sink maps** (`AccessKitSink::access_node`): a secured input
  becomes AccessKit's `PasswordInput` role — the word every platform
  AT announces. The tier maps no other role: a secure checkbox is not
  a word the vocabulary declares, and a non-input widget's answer is
  the honest absence. The value the node ships is the mask pass 9
  resolved; the sink ships what the tree hands it, never a secret.
- **The declaration is the constructor's** (the M98 rule at the
  secure tier): there is no `set_secure` door, the way the IME purpose
  itself is constructor-immutable — a re-render that repeats the
  `secure` word queues nothing. The write door stays open: a password
  manager fills the field through the AT's own `SetValue`, the same
  funnel every editable answers — the value lands, the mask answers
  the read.
- **No patch tier growth**: the word is constructor-immutable and the
  mask is derived — the `secure` word the text-input element has
  parsed since the M0 registry builds the secured constructor, so the
  DSL's fields declare through the same door with no new prop class to
  pay for.
- **The bench** (`secure/snapshot_1k_secured`): the 1,000-node secured
  input arena against the same input arena bare
  (`placeholder/snapshot_1k_bare`, the M96 bare case) — the M95
  pricing rule (the tier's cost measured against the same corpus, one
  fact richer). The corpus grows 111 → 112 by the coordinated rule,
  the `secure_snapshot_overhead` ratio budget (≤ 1.25×) holding the
  word's cost at the measured class: the bool and the mask the node
  builds beside the value's own copy — the same-length allocation
  either way. The m-suite pins re-agree at 112.
- `examples/secure.rs` drives the beats from the consumer side — the
  secured field beside the bare one, the *Fill* write landing the mask
  the tree carries, and the status line mirroring the word the tree
  says.

## Re-open rules (pre-registered)

- a consumer needs the *revealed* word (a "show password" toggle) →
  AccessKit's own contract names the mapping ("when the password is
  revealed, use `Role::TextInput` instead"): the flag grows the
  revealed state beside the constructor's, the sink's mapping one
  branch beside the one it has — the m100 suite's read doors
  re-pinning beside it.
- a consumer needs the word on other editables (a secure note field, a
  custom secret entry) → the trait door already answers them
  (`secure_entry()` is not input-shaped beyond the role gate), the
  producers joining without a seam change.
- a consumer needs the AT's echo of the masked length (a strength
  meter reading the mask) → the tree already carries the mask's shape
  beside the flag; the composition is the consumer's story, not the
  vocabulary's.
- a consumer needs the *automatic fill* tier (one-time codes, autofill
  hints) → AccessKit's platform autos fill contract is the platform
  adapter's story; the seam maps no platform-specific property until a
  consumer prices it (ADR-0046's kill rule).
