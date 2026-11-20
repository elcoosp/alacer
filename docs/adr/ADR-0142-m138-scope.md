# ADR-0142: M138 scope — the image tier: the named picture the app presents, said by the tree

- **Status:** Accepted (2026-11-20)
- **Implements:** the word the picture vocabulary speaks. Through
  M137 the snapshot said the menu tier (ADR-0141), the tooltip tier
  (ADR-0140), the tree tier (ADR-0139), the dialog tier (ADR-0138),
  the meter tier (ADR-0137) — and the *picture* had no word of its
  own: an app wiring a chart, a logo, a diagram reached for a bare
  [`Label`](crates/alacer-widgets/src/label.rs) wearing no picture
  word (the picture the AT cannot name is a picture it cannot offer on
  demand — the ADR-5 rule the named roles carry) or for a
  [`Canvas`](crates/alacer-widgets/src/canvas.rs) wearing the wrong
  half (the canvas tier, M82, names a *drawing surface the app paints
  into* — a paint seam with a required name; a picture is *named
  content the app presents*, the pixels already painted or embedded —
  the wrong word wearing the wrong half). [`Role::Image`]
  (crates/alacer-core/src/widget.rs) carries the picture word
  AccessKit's own `Image` role spells — the named-container shape
  (ADR-0128) grown to the content tier, the word that walls nothing.
- **Companion:** ADR-0128 (the member-and-container discipline the
  word grows), ADR-0133/0135/0138/0139/0140/0141 (the M129/M131/M134/
  M135/M136/M137 named-container precedents the tier follows — the
  strip, the pane, the window, the collection, the bubble, the menu),
  ADR-0099 (the M95 description tier — the alt text the word carries
  beside the name), ADR-0086 (the M82 canvas tier — the paint seam the
  word never replaces), ADR-5 (the required-name rule the word joins),
  ADR-2 (the one-currency discipline the change never leaves), ADR-9
  (the DAG the change never touches), architecture §7.2 pass 9 (the
  snapshot the word rides), §17 (the register), §18 (the M138 roadmap
  row)

## Decision

- **One word, the picture's own.** [`Role::Image`]
  (crates/alacer-core/src/widget.rs) carries the "reads as a picture"
  fact — AccessKit's own `Image` role spells it. The role joins the
  required-name set (an unnamed picture reads as a bare blank — the
  picture the AT cannot name is a picture it cannot offer on demand,
  the ADR-5 rule the named roles carry) and never joins the focusable
  set (a picture is not a control: it holds no focus and answers no
  activation — the M126/M129 container rule, at the picture tier; the
  controls the app mounts around it hold the focus). The role is *not*
  the canvas's word worn twice: the M82 canvas is the drawing surface
  the app paints *into* (the paint seam's producer, its own role), the
  image is *named content the app presents* — the pixels already
  painted or embedded, the word naming what they are. And it is *not*
  the label's word worn down: a label is static text; a picture is
  content the AT announces *as a picture* — the fact the vocabulary
  spells. **Pre-registered re-open rule:** a consumer needing the
  picture to answer the AT's actions (a clickable image button) grows
  the control beside the word, priced by a real control — the image
  button's word is the button's, never a second walk, and never by
  mutating what this word means.
- **The word never paints: the pixel story is the app's own.**
  [`Image`](crates/alacer-widgets/src/image.rs) prices the word and
  nothing else — the paint seam the canvas tier published carries
  whatever the app paints, the embedded raster rides whatever the app
  embeds, and the widget's own paint borrows the surface role so the
  word's slot reads as itself in the frame (the borrowed-roles
  discipline every canon widget paints by, ADR-0079). No door behind
  the word changes — the tier prices the word and nothing else.
- **The alt text rides the description word the tier already
  published.** The M95 description tier (ADR-0099) is the help text
  beside a name — the alt text's natural carrier: the picture's
  summary, source, or refresh cadence the name is too short to carry.
  [`Image::description`](crates/alacer-widgets/src/image.rs) the
  builder word and `set_description` the one write reporting the move
  (the write that lands where the text already is queues nothing — the
  M21 idle-frame property, at the description tier); `set_name` beside
  it, view reconciliation. No new property class: the alt text is the
  tree's truth the pixels never carry, riding the word M95 maps.
- **The living documents re-agree at M138 and 0.17.0** (the README's
  status block, register line, widget vocabulary, and example counts;
  the catalog; the register; the roadmap; the m-suite pins moving
  M136 → M138 across the chain; the CHANGELOG). The corpus pin
  re-agrees at 144 — the tier grows no snapshot route (the word is
  the picture's own, the shape the M124/M127/M129/M131/M134/M135/M136/
  M137 named containers rode), and its pricing door is the widgets
  crate's own bench
  ([`benches/image.rs`](crates/alacer-widgets/benches/image.rs),
  `image/name_write_256` against `image/idle_write_256`), outside the
  published corpus by ADR-0131's rule.

## Consequences

- **The published corpus stays at 144 — the pricing door is the
  widget's own.** The word's cost is priced in
  [`crates/alacer-widgets/benches/image.rs`]
  (crates/alacer-widgets/benches/image.rs): `image/name_write_256` —
  the moving rename (the `String`'s write the picture's word takes)
  against `image/idle_write_256`, the write that lands where the word
  already is (the honest floor of the same write at the widget seam,
  the M21 idle path priced). The pair rides outside the published
  baseline: the coordinated rule grows the corpus at the next
  published cut, never at a widget bench's. The m-suite pins re-agree
  at 144, the m138 suite's pin documenting the same shape.
- **The AT announces the picture, at last.** A screen reader
  encountering a mounted image announces the picture by name — the
  chart, the logo, the diagram the vocabulary could not say before —
  and the alt text rides beside it, the help the name alone never
  carried. The word walls nothing: the controls around the picture
  answer exactly as they always did, and the focus the app's controls
  need stays the walk the framework already publishes.
- **Consumers get the whole vocabulary for free.** The
  `examples/image.rs` beat is the shape: the named picture beside the
  canvas that paints it (the pixel story the app's own), the alt text
  the app's wiring moves, the reads reading back the way the app
  wired them — the neighbours untouched.
