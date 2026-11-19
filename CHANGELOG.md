# Changelog

All notable changes to **Alacer** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
generated from Conventional Commits by [git-cliff](https://git-cliff.org).

## [Unreleased]
### M129 — the toolbar tier (ADR-0133)

- The container every editing-and-formatting vocabulary speaks as "this strip holds the tools" said by the tree at last. Through M128 the snapshot said the bounded number field the AT walks by arrows (ADR-0132), the page selector and its strip (ADR-0131), the section title the document outline walks (ADR-0130), the hyperlink and its open verb (ADR-0129), the one-of-N control and its group (ADR-0128), the set words, the form facts, the collection facts — and the tool strip had no word: an app wiring one reached for a bare row (a container wearing no name — the set the AT cannot name is a set it cannot walk, the ADR-5 rule the named roles carry) or for a tab list wearing the wrong word (a tab strip *selects pages*; a toolbar *offers tools*, and the AT's vocabulary names the difference). The tier ships one role beside one machinery: [`Role::Toolbar`](crates/alacer-core/src/widget.rs) — the strip word AccessKit's own `Toolbar` role spells, the tab-list/radio-group shape (the named container, ADR-0128's member-and-container discipline) at the tool tier. The role joins the required-name set (an unnamed strip a bare box — the ADR-5 rule) and never the focusable set (a toolbar is not a control: it holds no focus and answers no activation — the tools hold the focus, the word walls nothing, the M126 heading's own rule at the container tier).
- The producer ships without a seam change: [`Toolbar`](crates/alacer-widgets/src/toolbar.rs) — the strip the tools' layout, never a second channel to them: the children ride the arena exactly as a `Row`'s do, each tool keeping its role, its label, and its doors (the button presses, the input edits, the strip naming the set they read as). No door behind the word changes — the tier prices the word and nothing else; the members' behaviour is the behaviour it always was. The word's one write is the programmatic [`Toolbar::set_name`](crates/alacer-widgets/src/toolbar.rs) (view reconciliation) — the word is the app's declaration, never a user door.
- The snapshot carries it (`Role::Toolbar` is the fact the pass-9 fill already walks), and the sink maps AccessKit's `Toolbar` — one branch beside the tab-list/radio-group pair's, no patch-tier growth. The m129 suite pins the strip against the focusable set: the tool rides it, the strip does not.
- The published corpus keeps its count at the M128 cut: the toolbar tier grows no snapshot route (the word is the container's own, the shape the M124/M127 named containers rode), and its pricing door is the widgets crate's own bench — [`benches/toolbar.rs`](crates/alacer-widgets/benches/toolbar.rs), `toolbar/name_write_256` (the moving rename: the `String`'s write the strip's word takes) against `toolbar/idle_write_256` (the write that lands where the word already is — the honest floor) — riding outside the published baseline, the coordinated rule growing the corpus at the next published cut, never at a widget bench's. The m-suite pins re-agree at 144.
- [`examples/toolbar.rs`](crates/alacer/examples/toolbar.rs) drives the beats from the consumer side: the named strip under the word, the tools the members' own controls, the presses reading back the way the app wired them — the neighbours untouched.

### M128 — the spin-button tier (ADR-0132)

- The control every quantity-and-count vocabulary speaks as "one bounded number, discretely written" said by the tree at last. Through M127 the snapshot said the page selector and its strip (ADR-0131), the section title the document outline walks (ADR-0130), the hyperlink and its open verb (ADR-0129), the set words, the form facts, the collection facts, the declared live region, the chord, the capacity, the language — and the number field had no tier: an app wiring one reached for a slider wearing text (the wrong shape — a slider's write is the *continuous drag* the pointer speaks, a spin button's write is the *discrete step* the arrows speak) or for a bare text input carrying no value words at all. The tier ships one role beside one machinery: [`Role::SpinButton`](crates/alacer-core/src/widget.rs) — the "writes one bounded number" fact AccessKit's own `SpinButton` role spells, the numbers riding the value tier's own vocabulary (the M3 value word, the M97 range word with the step beside the walls, the M99 numeric word beside the spoken one — never a second value language). The role joins the required-name set (an unnamed number field reads as a bare number — the ADR-5 rule).
- The producer ships without a seam change: [`SpinButton`](crates/alacer-widgets/src/spin_button.rs) — four doors, one funnel: the arrows step it (the slider's own key vocabulary), Home/End jump to the range's ends (the M47 named jump species, never a munged `SetValue`), a stepper click steps it from the pointer (the release over the stacked pair steps by its sign — no drag, no gesture log: a click that never was a gesture streams nothing), and the AccessKit value tier answers `SetValue` (parsed, then snapped like every other write), `Increment`, and `Decrement`. Every write clamps to the range, snaps to the step, and requests the same masks — `PAINT | A11Y` — and a write that lands where the value already is queues no masks and fires no handler (the M21 idle frame). `DefaultPress` is refused, never munged into a write (the M11 rule — a value control has no press); the availability tier guards every door (M22, ADR-0028): an unavailable field refuses, never munges. The value log's discipline is the slider's own (M40/M44/M45/M47/M49): every AT write and every keyboard step streams one [`ValueRecord`](crates/alacer-core/src/value.rs) — the ask beside the landing, the wall a pinned landing names — while the stepper clicks stay off the value log (the pointer's facts ride the dispatch funnel's own record).
- The snapshot carries it (`TreeNode::value`/`value_range`/`numeric_value` are the words M3/M97/M99 published), and the sink maps `Role::SpinButton` onto AccessKit's `SpinButton` — one branch beside the tab pair's, the value properties riding the M97/M99 mappings, no patch-tier growth. The field's frame joins the snapshot corpus (`spin_button.png`, blessed by the m128 suite).
- The published corpus keeps its count at the M127 cut: the spin-button tier grows no snapshot route (its words are the value tier's, published since M3/M97/M99), and its pricing door is the widgets crate's own bench — [`benches/spin_button.rs`](crates/alacer-widgets/benches/spin_button.rs), `spin_button/step_route_256` (the moving step: the state write, the mask request, the handler call, the record the M45 rule streams) against `spin_button/redundant_ask_256` (the M21 idle path priced — the arrow the wall held) — riding outside the published baseline, the coordinated rule growing the corpus at the next published cut, never at a widget bench's. The m-suite pins re-agree at 144.
- [`examples/spin_button.rs`](crates/alacer/examples/spin_button.rs) drives the beats from the consumer side: the named field under the word, the value mounted from the signal, every write — arrow, stepper, AT act — landing through the funnel the constructor mounted, the status line reading the value back the way the tree carries it — the neighbours untouched.

### M127 — the tab tier (ADR-0131)

- The control every preferences-and-mail vocabulary speaks as "one page of many, shown" said by the tree at last. Through M126 the snapshot said the section title the document outline walks (ADR-0130), the hyperlink and its open verb (ADR-0129), the set words, the form facts, the collection facts, the declared live region, the chord, the capacity, the language — and the tab strip had no tier: an app wiring one reached for a row of buttons (the wrong shape — a button's activation is a *press* with no after-state, a tab's activation *selects the page*) or for bare labels wearing no selection fact at all. The tier ships two roles beside one machinery: [`Role::Tab`](crates/alacer-core/src/widget.rs) — the "selects a page" fact AccessKit's own `Tab` role spells, the selection riding the selected word the tree already carries (the M108 contract, ADR-0112, never a second selection vocabulary) — and [`Role::TabList`](crates/alacer-core/src/widget.rs), the named strip the AT announces before it walks the pages, the radio tier's own member-and-container shape (ADR-0128) re-proved at the page tier. Both roles join the required-name set (an unnamed page a bare mark, an unnamed strip a bare box — the ADR-5 rule).
- The producer ships without a seam change: [`Tab::select`](crates/alacer-widgets/src/tab.rs) — the one select path the click, the Space key, and the AccessKit default action all land in (the radio's three-doors-one-funnel shape) — beside [`set_selected`], the one deselect door (programmatic state, never a user door). A selected page asked again is the M21 idle frame: the selection does not move, the handler does not fire, and the widget's own doors request no masks — the a11y door answers `true` (the ask is the tab's own tier) and the re-announcement it costs rides the dispatch funnel's coarse `consumed` contract, the one coarse corner named at the module docs. The exclusivity is the app's wiring: the framework never rewrites widgets it does not own (A1), the [`on_select`] handler and the view's programmatic door where the sibling clears. The keyboard's activation names itself (M70, ADR-0076) — answered only when the selection moved, the redundant ask streaming nothing (one fact per trail, the M39 rule). The availability tier guards every door (M22, ADR-0028): an unavailable tab refuses, never munges (the M11 rule).
- The snapshot carries it (`TreeNode::selected` already announces the selection, the M108 word the tier rides), and the sink maps `Role::Tab` onto AccessKit's `Tab` and `Role::TabList` onto AccessKit's `TabList` — two branches beside the radio pair's, no patch-tier growth.
- The published corpus keeps its count at the M126 cut: the tab tier grows no snapshot route (its word is the selected tier's, published since M108), and its pricing door is the widgets crate's own bench — [`benches/tab.rs`](crates/alacer-widgets/benches/tab.rs), `tab/select_route_256` (the moving select: the state write, the mask request, the handler call) against `tab/redundant_ask_256` (the M21 idle path priced) — riding outside the published baseline, the coordinated rule growing the corpus at the next published cut, never at a widget bench's. The m-suite pins re-agree at 144.
- [`examples/tab.rs`](crates/alacer/examples/tab.rs) drives the beats from the consumer side: the named strip under the word, the pages mounting selected and unselected from the signal, the select handler writing the one write the exclusivity rides, the status line reading the selection back the way the tree carries it — the neighbours untouched.

### M126 — the heading tier (ADR-0130)

- The word every document vocabulary speaks about its structure said by the tree at last. Through M125 the snapshot said the hyperlink and its open verb (ADR-0129), the one-of-N control and the group it reads as (ADR-0128), the set words, the form facts, the collection facts, the declared live region, the chord, the capacity, the language — and the section title had no role: an app wiring a document outline reached for a plain label, and the AT walked the page as one flat run of text — the rotor's quick-nav, the outline every reader skims by, having nothing to land on. The role ships: [`Role::Heading`](crates/alacer-core/src/widget.rs) — the "names a section" fact AccessKit's own `Heading` role spells (not a bold label wearing bigger glyphs), the depth riding the level word the M113 tier published ([`Widget::level`], AccessKit's `Level` property, zero-based beside ARIA's one-based `aria-level` — the heading's natural carrier, never a second depth vocabulary, the sink's existing mapping serving the tier with no new branch). The role joins the required-name set (an unnamed heading reads as bare styled text — the outline the AT cannot walk; the ADR-5 rule).
- The producer ships without a seam change: [`Heading`](crates/alacer-widgets/src/heading.rs) — the text *is* the name — with [`set_level`] beside [`set_name`] the app's live doors through the mutate seam (`None` restoring the honest absence — the flat title, never a guessed level; the write that lands where the depth already is queues nothing, the M21 idle-frame property). A heading is not a control: it holds no focus, answers no activation, and the AT's presses land on the widgets the section contains — the word names structure, it walls nothing, the section's content reading as it always did. The glyphs are the app's styling (the `font_size` override the label tier answers); the depth is a fact and never a font size.
- The snapshot carries it (`TreeNode::level` is the word M113 published), and the sink maps AccessKit's `Heading` role — one branch beside the link's, no patch-tier growth.
- The published corpus keeps its count at the M125 cut: the heading tier grows no snapshot route (the role is the fact), and its pricing door is the widgets crate's own bench — [`benches/heading.rs`](crates/alacer-widgets/benches/heading.rs), `heading/level_write_route_256` (the moving write: the compare, the state write, the moved answer) against `heading/idle_write_256` (the M21 idle path priced) — riding outside the published baseline, the coordinated rule growing the corpus at the next published cut, never at a widget bench's. The m-suite pins re-agree at 144.
- [`examples/heading.rs`](crates/alacer/examples/heading.rs) drives the beats from the consumer side: the outline under the word — the root title the honest absence beside, the nested sections riding their zero-based depths, the deepening write the one mask the outline costs, the status line reading the moved depth back the way the tree carries it — the neighbours untouched.

### M125 — the link tier (ADR-0129)

- The control every document vocabulary speaks as "this goes somewhere" said by the tree at last. Through M124 the snapshot said the one-of-N control and the group it reads as (ADR-0128), the set words, the form facts, the collection facts, the declared live region, the chord, the capacity, the language — and the hyperlink had no control: an app wiring one reached for a button (the wrong verb — a button's activation is a *press*, a link's is an *open*, and the AT's vocabulary names the difference) or for styled text wearing no activation fact at all. The control ships: [`Role::Link`](crates/alacer-core/src/widget.rs) — the "opens something" fact AccessKit's own `Link` role spells (not a button wearing styled text), the destination the app's own (the tree says "opens", never where — the name and the description, ADR-0099, carry the words the app declared), the role joining the required-name set (an unnamed link reads as bare styled text — the ADR-5 rule).
- The producer ships without a seam change: [`Link::on_open`](crates/alacer-widgets/src/link.rs) — the open act the app's navigation rides — beside [`set_on_open`], the app's live door through the mutate seam. The activation verb is the link's own: the click opens on release (the GTK rule — an auto-repeat hold cannot machine-gun the navigation), the AT's default action is the open ask the door answers, and the keyboard's key the platform convention names — **Enter** opens, Space is handed on (the scroller's key, the framework's to take; the M10 parity at the link tier — the activation keys are the control's own, not every key's, the radio's Space select the checkbox vocabulary's own verb and the link's is not). The Enter-open streams one `ActionRecord` with `Keyboard` beside it (M70, ADR-0076); the availability tier guards every door (M22, ADR-0028): an unavailable link refuses, never munges (the M11 rule).
- The snapshot carries it (`Role::Link` is the fact the pass-9 fill already walks), and the sink maps AccessKit's `Link` role — one branch beside the radio pair's, no patch-tier growth. The control prices the control and nothing else: a link names a destination-shaped fact, it walls nothing, the activation and focus routes the widgets' own.
- The published corpus keeps its count at the M123 cut: the link tier grows no snapshot route (the role is the fact), and its pricing door is the widgets crate's own bench — [`benches/link.rs`](crates/alacer-widgets/benches/link.rs), `link/open_route_256` (the activation: the handler call, the mask request) against `link/refusal_wall_256` (the M22 refusal priced) — riding outside the published baseline, the coordinated rule growing the corpus at the next published cut, never at a widget bench's. The m-suite pins re-agree at 144.
- [`examples/link.rs`](crates/alacer/examples/link.rs) drives the beats from the consumer side: the described link under the word, the plain link the honest absence beside it, every activation routing through the same door — the status line reading the destination back the way the app named it, the neighbours untouched.

### M124 — the radio tier (ADR-0128)

- The control every settings vocabulary speaks as "one of these, not many" said by the tree at last. Through M123 the snapshot said which member a collection picks (ADR-0112), the set's span and the member's place (ADR-0123), the form facts, the collection facts, the declared live region, the chord, the capacity, the language — and exclusive choice had no control: an app wiring one-of-N reached for a checkbox that toggles itself off (the wrong verb — a radio selects, it never deselects itself) or for a bare button wearing no selection fact at all. The tier ships two roles beside one machinery: [`Role::Radio`](crates/alacer-core/src/widget.rs) — the selection the M6 `toggled` contract announces exactly as the checkbox's, `On` or `Off`, never `Mixed` (ADR-0098's rule restated: the summary is the select-all parent's word, a radio has no children to summarize) — and [`Role::RadioGroup`](crates/alacer-core/src/widget.rs), the named container the set reads as, the set tier's own span word (`Widget::size_of_set`, ADR-0123) its natural carrier: no second set vocabulary, the member's place ([`Radio::position_in_set`]) riding the words published since M119. Both roles join the required-name set (an unnamed member a bare mark, an unnamed group a bare box — the ADR-5 rule).
- The producer ships without a seam change: [`Radio::select`](crates/alacer-widgets/src/radio.rs) — the one select path the click, the Space key, and the AccessKit default action all land in, exactly the checkbox's three-doors-one-funnel shape — beside [`set_checked`], the one deselect door (programmatic state, never a user door — the checkbox's own rule). A checked member asked again is the M21 idle frame: the selection does not move, the handler does not fire, and the widget's own doors request no masks — the a11y door answers `true` (the ask is the radio's own tier) and the re-announcement it costs rides the dispatch funnel's coarse `consumed` contract, the one coarse corner named at the module docs. The exclusivity is the app's wiring: the framework never rewrites widgets it does not own (A1), the [`on_select`] handler and the view's programmatic door where the sibling clears. The keyboard's activation names itself (M70, ADR-0076) — answered only when the selection moved, the redundant ask streaming nothing (one fact per trail, the M39 rule).
- The snapshot carries it (`TreeNode::toggled` already announces the selection, the set words already say the place and the span), and the sink maps `Role::Radio` onto AccessKit's `RadioButton` (the `Checked` state the toggle tier already carries) and `Role::RadioGroup` onto AccessKit's `RadioGroup` — two branches beside the checkbox's, no patch-tier growth.
- The published corpus keeps its count at the M123 cut: the radio tier grows no snapshot route (its words are the set tier's), and its pricing door is the widgets crate's own bench — [`benches/radio.rs`](crates/alacer-widgets/benches/radio.rs), `radio/select_route_256` (the moving select: the state write, the mask request, the handler call) against `radio/redundant_ask_256` (the M21 idle path priced) — riding outside the published baseline, the coordinated rule growing the corpus at the next published cut, never at a widget bench's. The m-suite pins re-agree at 144.
- [`examples/radio.rs`](crates/alacer/examples/radio.rs) drives the beats from the consumer side: the named group under the word, the members mounting checked and cleared from the signal, the select handler writing the one write the exclusivity rides, the status line reading the selection back the way the tree carries it — the neighbours untouched.

### M123 — the language tier (ADR-0127)

- The word every multilingual vocabulary speaks about what language the text speaks — "this label is French, that one Japanese" — said by the tree at last. Through M122 the snapshot said what a control *does* (ADR-0099), what an empty field hints (ADR-0100), the field's line capacity (ADR-0126), the form facts, the collection facts, the declared live region, the chord the control is invoked with — and the language the label's text, the button's name, the field's value speak was a fact the app knew and the tree did not: the AT announced the text in whatever voice the platform guessed, and a declared tag had no tier. The producer ships: [`Widget::language`](crates/alacer-core/src/widget.rs) — `None` the honest absence (a widget that declared no language announces none, never a guessed tag — the ADR-0087 presence rule at the tier), the string the app declared (the BCP 47 tag the AT voices the text with — no kind enum, AccessKit's `Language` property a string, the ADR-0099 description precedent at the string tier), the word mirroring the text it never translates: the word says the text's language, it never changes the text.
- The producer joins without a seam change: [`Label::language`](crates/alacer-widgets/src/label.rs) — the constructor word naming the tag from the first frame (the localized greeting, the foreign-language quote) — beside [`set_language`], the app's live door through the mutate seam (`None` the honest absence restored — the tag withdrawn; the write that lands where the word already is queues nothing, the M21 idle-frame property). The word is a text word: the plain widgets answer nothing (a button reports its press, a list its scroll — the language is the text's own), so no other producer joins at the cut; other text producers join without a seam change (the M107 rule) the day the vocabulary needs them.
- The snapshot carries it (`TreeNode::language`), and the sink maps AccessKit's `Language` property (the BCP 47 tag the platform AT voices the text with) — one branch beside the shortcut word's, no patch-tier growth. The word prices the word and nothing else: a tag names a fact, it walls nothing, the text's read routes the widget's own.
- The corpus grew 142 → 144 by the coordinated rule: `language/snapshot_1k_labels_language` — the 1,000-node label arena carrying the constructor's tag (the BCP 47 spellings cycling down the stream, the way a localized surface alternates) — against `language/snapshot_1k_labels_bare`, the same arena bare (the tier's own honest pair, the M119 shape; the M95 pricing rule — the word is a label word, so the honest comparison is labels against labels) — the `language_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the snapshot the label rides (measured 1.099×). The m-suite pins re-agree at 144.
- [`examples/language.rs`](crates/alacer/examples/language.rs) drives the beats from the consumer side: the worded Bienvenue under the word, the plain Welcome and the plain button the honest absence between them, every door answering as it always did — the text the text it always was — the language door moving the word: the plain Hello declaring its own tag, the Bienvenue tag withdrawn, clear.

### M122 — the multiline tier (ADR-0126)

- The word every form-and-composer vocabulary speaks about how much the field takes — "this field wraps onto many lines" — said by the tree at last. Through M121 the snapshot said what an empty field hints (ADR-0100), the form's ask and the form's rejection (ADR-0110/ADR-0111), the field's suggestion contract (ADR-0115), the secret entry and its reveal (ADR-0104/ADR-0106), the wall the read-only field holds (ADR-0109) — and the capacity the composer's notes field, the feedback box, the log viewer take was a fact the field's own constructor knew and the tree did not: the field took every line the user typed, and the AT announced the hint, the ask, the rejection — and never the capacity. The producer ships: [`Widget::multiline`](crates/alacer-core/src/widget.rs) — `false` the honest absence (a field that declared no capacity announces none, never a guessed "multiline" — the ADR-0113 flag shape at the tier), the word mirroring the field it never changes: the word says the capacity, it never changes what the field takes — the typing, caret, selection and submit routes the widget's own.
- The producer joins without a seam change: [`TextInput::multiline`](crates/alacer-widgets/src/text_input.rs) — the constructor word naming the capacity from the first frame (the composer's notes field, the feedback box) — beside [`set_multiline`], the app's live door through the mutate seam (`false` the honest absence restored — the capacity withdrawn; the write that lands where the word already is queues nothing, the M21 idle-frame property). The word is a field word: the plain widgets answer nothing (a button reports its press, a list its scroll — the capacity is the field's own), so no other producer joins at the cut; other editable producers join without a seam change (the M107 rule) the day the vocabulary needs them.
- The snapshot carries it (`TreeNode::multiline`), and the sink resolves AccessKit's `MultilineTextInput` role (ARIA's `aria-multiline` beside the `textbox` role) — the capacity word AccessKit's own vocabulary carries, there being no separate property to mirror — in the role branch the secure tier opened (M100): the secured field's `PasswordInput` word wins the branch it has carried, the capacity word riding the snapshot beside the role it never displaces; a field that declared no capacity reads as the ordinary `TextInput` role it always was. A capacity write re-announces exactly that node, the same node-only delta as every other property.
- The corpus grew 141 → 142 by the coordinated rule: `multiline/snapshot_1k_fields_multiline` — the 1,000-node input arena carrying the constructor's capacity word (the way a composer mounts its notes field) — against the placeholder route's published bare arena (`placeholder/snapshot_1k_bare`, the M96 case the M111 precedent reuses; the M95 pricing rule — the word is a field word, so the honest comparison is fields against fields, the M108/M121 denominator shape) — the `multiline_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the snapshot the field rides (measured 0.877×). The m-suite pins re-agree at 142.
- [`examples/multiline.rs`](crates/alacer/examples/multiline.rs) drives the beats from the consumer side: the worded notes field under the word, the plain title field and the plain label the honest absence between them, every door answering as it always did — the AT's write door landing where it always landed — the multiline door moving the word: the plain summary field declaring its own capacity, the notes capacity withdrawn, clear.

### M121 — the shortcut tier (ADR-0125)

- The word every menu-and-toolbar vocabulary speaks about how a control is invoked from the keyboard — "Save — Control+S" — said by the tree at last. Through M120 the snapshot said the declared live region (ADR-0124), the collection's place and span (ADR-0123), what a control is read as (ADR-0120), the subtree's updating fact (ADR-0121), what a control opens, the set's current item, the form facts — and the chord the keymap's own dispatch table rides (M35, ADR-0041; the consult's name leg, M37, ADR-0043) was a fact the trail knew and the tree did not: the keymap named its bindings to the causal log and the platform menus carried their accelerators, while the control the screen reader announced said its name and never the way its keymap answers. The producer ships: [`Widget::keyboard_shortcut`](crates/alacer-core/src/widget.rs) — `None` the honest absence (a control that declared no chord announces none, never a guessed binding), the string the app declared (the spellings the app's own vocabulary — no kind enum, AccessKit's `KeyboardShortcut` property a string, the ADR-0099 description precedent at the string tier), the word mirroring the keymap seam it never replaces: the word says the chord the app named, it never binds a key — the keymap's dispatch is the dispatch it always was (the trail leg names the binding a press fired, the tree leg says the chord the control declared, neither silencing the other).
- The producer joins without a seam change: [`Button::keyboard_shortcut`](crates/alacer-widgets/src/button.rs) — the constructor word naming the chord from the first frame — beside [`set_keyboard_shortcut`], the app's live door through the mutate seam (`None` the honest absence restored — the chord withdrawn; the write that lands where the word already is queues nothing, the M21 idle-frame property). The word is an actionable word: the plain widgets answer nothing (a label reports its text, a list its scroll — the chord is the control's own), so no other producer joins at the cut; other actionable producers join without a seam change (the M107 rule) the day the vocabulary needs them.
- The snapshot carries it (`TreeNode::keyboard_shortcut`), and the sink maps AccessKit's `KeyboardShortcut` property (ARIA's `aria-keyshortcuts`) — one branch beside the set word's, no patch-tier growth. The word prices the word and nothing else: a chord names a fact, it walls nothing, the control's press and focus routes the widgets' own.
- The corpus grew 140 → 141 by the coordinated rule: `shortcut/snapshot_1k_buttons_shortcut` — the 1,000-node button arena carrying the constructor's chord (the platform spellings cycling down the stream, the way a toolbar's buttons alternate) — against the current route's published bare arena (`current/snapshot_1k_buttons_bare`, the M114 case the M111 precedent reuses; the M95 pricing rule — buttons against buttons) — the `shortcut_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the snapshot the button rides (measured 1.22×). The m-suite pins re-agree at 141.
- [`examples/shortcut.rs`](crates/alacer/examples/shortcut.rs) drives the beats from the consumer side: the worded save and print under the word, the plain close and the plain label the honest absence between them, every door answering as it always did, the shortcut door moving the word — the plain export declaring its own chord, the save chord withdrawn, clear.

### M120 — the live tier (ADR-0124)

- The word every streaming vocabulary speaks about the region whose changes the AT speaks — "this region is live, announce what lands here" — said by the tree at last. Through M119 the snapshot said the collection's span and the member's place beside the form facts and the updating fact — and the pipeline's own reserved region (§11.6, ADR-0024) carried the imperative half alone: the framework reserved the node, the app pushed announcements through it, and a region the *app* wanted the AT to watch had no word — the AT spoke what the framework's channel pushed and never what the app declared. The producer ships: [`Widget::live`](crates/alacer-core/src/widget.rs) — `None` the honest absence (a container that declared nothing announces no region, never a guessed politeness), the politeness the announcement seam's own vocabulary carries (ADR-0022's `Politeness` pair — `polite` the AT waits its turn, `assertive` it interrupts; the tier reuses the seam's enum, never a second politeness type), the declared counterpart of the reserved region: the app says *which* region the AT watches, the imperative channel the channel it always was.
- The producers join without a seam change: [`List::live`](crates/alacer-widgets/src/list.rs) — the constructor word naming the declared region, the feed the AT watches — beside [`set_live`], the app's live door through the mutate seam (`None` the honest absence restored — the region withdrawn; the write that lands where the word already is queues nothing), and [`Table::live`](crates/alacer-widgets/src/table.rs) joining by the M107 rule: the streaming ledger joins without a seam change. The word is a container word: the members and plain widgets answer nothing (a row reports its depth, a button its press — the watched region is the container's own), so no producer joins there.
- The snapshot grows the field it already carried for the pipeline's own region to the app's containers (one field, two riders — the reserved region's politeness and the declared regions' riding the same `TreeNode::live`), and the sink's existing `Live` mapping serves both: AccessKit's `Live` property (ARIA's `aria-live`) mapped in the same snapshot every other property rides. One contract re-states where the pairing lives: the Status/Alert role pairing is the pipeline's own region's (the role its announcement contract names) — a container that *declares* the word keeps the role it already was; the word names the region the AT watches, it re-roles nothing.
- The corpus grew 139 → 140 by the coordinated rule: `live/snapshot_1k_lists_live` — the 1,000-node list arena carrying the declared region's word (the politeness cycling down the stream) — against the busy route's published bare arena (`busy/snapshot_1k_lists_bare`, the M117 case the M111 precedent reuses; the M95 pricing rule — lists against lists) — the `live_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the snapshot the container rides (measured 0.945×). The m-suite pins re-agree at 140.
- [`examples/live.rs`](crates/alacer/examples/live.rs) drives the beats from the consumer side: the polite feed and the assertive ledger under the word, the settled archive and the plain scroller the honest absence between them, every door answering as it always did, the live door moving the word — the feed's region withdrawn, the archive declaring its own, clear.

### M119 — the set tier (ADR-0123)

- The two numbers every collection vocabulary speaks about its members and its span — "row 3 of 500" — said by the tree at last. Through M118 the snapshot said what a control is read as (ADR-0120), the subtree's updating fact (ADR-0121), the routing outcome the harness answered as data (ADR-0122), what a control opens, the set's current item, the nested member's depth, the sorted column's order, the field's suggestion contract, the member's disclosure, the container's capacity, the member's pick, the form's ask and rejection — and the place a member stands in the collection's own span, and the span itself, were the two collection facts the tree carried nothing for: the AT announced the grid position the M6 slot carried (a zero-based row and column, a table's geometry), the depth, the name — and never the place or the size. The producers ship: [`Widget::position_in_set`](crates/alacer-core/src/widget.rs) — `None` the honest absence (a member that declared no place announces none, never a guessed `0`), the zero-based count AccessKit's own base (the contract naming it verbatim beside ARIA's one-based `aria-posinset` — one branch with no translation) — and [`Widget::size_of_set`](crates/alacer-core/src/widget.rs) — `None` the honest absence (a collection that declared no span announces none, never a guessed one), the size on the *container* per AccessKit's own contract ("`aria-setsize` is set on each item, while this property is set on the container" — the one difference from ARIA the documentation carries).
- The producers join without a seam change: [`TableRow::position_in_set`](crates/alacer-widgets/src/table.rs) — the constructor word naming the member's place — beside [`set_position_in_set`], the app's live door through the mutate seam (`None` the honest absence restored — the row left the set; the write that lands where the word already is queues nothing), and [`Table::size_of_set`](crates/alacer-widgets/src/table.rs) — the constructor word naming the collection's span — beside [`set_size_of_set`], the container shape the M109 tier rode. The words are collection words: the plain widgets answer nothing (a button reports its press, a field its value — the place and the span are the collection's own), so no other producer joins there.
- The snapshot carries them (`TreeNode::position_in_set` / `TreeNode::size_of_set`), and the sink maps AccessKit's `PositionInSet` and `SizeOfSet` properties (ARIA's `aria-posinset` / `aria-setsize`) — one branch pair beside the level word's, no patch-tier growth. The words price the words and nothing else: a place and a span name facts, they wall nothing, the rows' and the table's press and focus routes the widgets' own.
- The corpus grew 137 → 139 by the coordinated rule: `set/snapshot_1k_rows_set` — the 1,000-node row arena carrying the place word under a table declaring the span — against the same shapes bare (`set/snapshot_1k_rows_bare`, the tier's own honest pair, the M117 denominator shape) — the `set_snapshot_overhead` ratio budget (≤ 1.25×) holding the words' cost at the snapshot the nodes ride (measured 1.011×). The m-suite pins re-agree at 139. The cut also re-publishes the baseline and dashboard with the pair, and re-states two contracts the published data could no longer hold: the range and numeric snapshot budgets (M97/M99's ≤ 1.25×) measured 1.31–1.33× the minimal arena across two hosts — the gate re-states at 1.35×, the measured ratio with headroom, the honest re-measurement the coordinated rule's own work (ADR-0101/ADR-0103's cites re-agreed).
- [`examples/set.rs`](crates/alacer/examples/set.rs) drives the beats from the consumer side: the sized ledger and its placed rows beside the unsized archive and the plain widgets, every door answering as it always did, the live doors moving the words — the ledger's third row taking its place, the archive starting its own count, clear — the honest absences restored.

### M118 — the routing-outcome doors (ADR-0122)

- The harness's keyboard and IME doors answer what the pipeline answered, as data. Through M117 the doors dispatched and settled — and dropped the verdict on the floor (`let _ = self.pipeline.dispatch_key(event)`): a test that wanted to know whether the field took the key, the wall refused it, or the funnel fell through had to smuggle the answer out through effects — the value that moved, the trail that streamed — the one routing question the harness could not ask directly. The pipeline always answered as data (the M27 rule at the dispatch seam): [`EventResponse`](crates/alacer-core/src/widget.rs) — `Handled` the widget (or the keymap or a framework fallback) consumed the key, `Ignored` it bubbled on. The doors carry that answer to the test: [`TestHarness::key`](crates/alacer-testing/src/lib.rs), [`TestHarness::press_key`] and [`TestHarness::ime`] answer the funnel's own verdict; [`TestHarness::type_text`] answers one verdict per dispatched character, in order — the field that took the characters, the wall that consumed its refusals (M105, ADR-0109), and the widget that takes no text each name themselves.
- The doors price nothing but the answer: the dispatch they rode is byte-for-byte the dispatch they always ran (the settle still follows, the `# Panics` contract stands) — the verdict is the return value the funnel already computed, never a second route. The corpus keeps its count at the M117 cut: no case joins and no case leaves, the door prices no frame work. The replay driver records steps, not verdicts (the recording tier's own contract), so its door calls read unchanged.
- [`examples/key_routing.rs`](crates/alacer/examples/key_routing.rs) drives the beats from the consumer side: the stray key the empty funnel falls through, the Tab the cycling takes, the characters a focused field lands, the characters a plain button takes no interest in, the read-only wall that consumes its refusals while landing nothing, and the IME route's verdict beside them.

### M117 — the busy tier (ADR-0121)

- The word every bulk-update vocabulary speaks about the region under construction — "this list is still loading, hold what you would announce" — said by the tree at last. Through M116 the snapshot said what a control is read as (ADR-0120), what it opens (ADR-0119), the set's current item, the nested member's depth, the sorted column's order, the field's suggestion contract, the member's disclosure, the container's capacity, the member's pick, the form's ask and rejection, the field's secret entry — and the fact the AT defers its speech on while the stream writes the subtree was the one updating fact the tree carried nothing for: the AT announced every half-settled intermediate the stream wrote in. The producer ships: [`Widget::busy`](crates/alacer-core/src/widget.rs) — `false` the honest absence (a container that declared nothing announces nothing, never a guessed "updating"), the word riding beside the announcement machinery it paces (§11.6, ADR-0024's reserved region — the declared counterpart: the app says *which* region is settling, the AT holds what it would announce while it is).
- The producers join without a seam change: [`List::busy`](crates/alacer-widgets/src/list.rs) — the constructor word naming the initial updating fact, the scroller the feed fills — beside [`set_busy`], the app's live door through the mutate seam (`false` the honest absence restored; the write that lands where the word already is queues nothing), and [`Table::busy`](crates/alacer-widgets/src/table.rs) joining by the M107 rule: the streaming ledger joins without a seam change. The word is a container word: the members and plain widgets answer nothing (a row reports its depth, a button its press — the updating fact is the region's own), so no producer joins there.
- The snapshot carries it (`TreeNode::busy`), and the sink maps AccessKit's `Busy` property (ARIA's `aria-busy`) — one branch beside the multiselectable word's, no patch-tier growth. The word prices the word and nothing else: the updating fact is not a wall, the children's press and focus routes the widgets' own — the AT defers its own speech, it is never the tree's to silence.
- The corpus grew 135 → 137 by the coordinated rule: `busy/snapshot_1k_lists_busy` — the 1,000-node list arena carrying the updating word — against the same arena bare (`busy/snapshot_1k_lists_bare`, the tier's own honest pair; the M95 pricing rule — the word is a container word, so the honest comparison is lists against lists, the M108/M109 denominator shape) — the `busy_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the snapshot it rides. The m-suite pins re-agree at 137.
- [`examples/busy.rs`](crates/alacer/examples/busy.rs) drives the beats from the consumer side: the loading feed and the streaming ledger under the word, the settled archive and the plain scroller the honest absence between them, every door answering as it always did, the live door moving the word — the feed finishing, the archive starting its own refresh, clear.

### M116 — the role-description tier (ADR-0120)

- The word every custom-control vocabulary speaks about what a widget is *read as* — "this slider is a star rating, that button is a wizard page" — said by the tree at last. Through M115 the snapshot said what a control *does* (the description, ADR-0099), what it opens (the has-popup tier, ADR-0119), the span a value rides, the form's ask and rejection, the collection's pick, capacity and disclosure, the field's suggestion contract, the sorted column's order, the nested member's depth, the set's current item — and the fact the AT turns into "star rating" while it walks the form was the one specialization fact the tree carried nothing for: the AT announced the generic role — "slider", "button" — and never the read-as one. The producer ships: [`Widget::role_description`](crates/alacer-core/src/widget.rs) — `None` the honest absence (a control that claims no specialization is read as the generic role it already was, never a guessed one), the word riding beside the role the snapshot's `Role` already spells. No kind enum mirrors this tier — AccessKit's `RoleDescription` is a string property (the description precedent, ADR-0099, at the string tier), so the mapping is one branch with no translation.
- The producers join without a seam change: [`Button::role_description`](crates/alacer-widgets/src/button.rs) — the constructor word naming the initial read-as role — beside [`set_role_description`], the app's live door through the mutate seam (`None` the honest absence restored; the write that lands where the word already is queues nothing), and [`Slider::role_description`](crates/alacer-widgets/src/slider.rs) — the constructor word the canonical specialization rides ("star rating"), the M107 join rule: the slider joins without a seam change, its value funnel the doors it always was. The word is a custom-control word: the containers and collection members answer nothing (the read-as role is the specialized control's own fact), so no producer joins there.
- The snapshot carries it (`TreeNode::role_description`), and the sink maps AccessKit's `RoleDescription` property (ARIA's `aria-roledescription`) — one branch beside the description word's, no patch-tier growth. The word prices the word and nothing else: the read-as role is not a wall, the control's press and focus routes the widget's own.
- The corpus grew 134 → 135 by the coordinated rule: `role_description/snapshot_1k_role_descriptions` — the 1,000-node button arena carrying the read-as word — against the same arena bare (`current/snapshot_1k_buttons_bare`, the M114 case the M111 precedent reuses; the M95 pricing rule — the word is a button word, so the honest comparison is buttons against buttons, the M108/M110 denominator shape) — the `role_description_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the snapshot it rides. The m-suite pins re-agree at 135.
- [`examples/role_description.rs`](crates/alacer/examples/role_description.rs) drives the beats from the consumer side: the star-rating slider beside the plain slider, the wizard-page button beside the plain button, the AT's press landing on the plain button untouched (the word names a read-as role, it walls nothing), the live door moving the word — specializing, rewording, clear — the honest absence restored.

### M115 — the has-popup tier (ADR-0119)

- The word every menu-and-picker vocabulary speaks about what a control opens — "this one opens a menu, that one a dialog" — said by the tree at last. Through M114 the snapshot said what a control *does* (the description, ADR-0099), what an empty field *hints* (the placeholder, ADR-0100), the span a value rides (the range tier, ADR-0101), the form's ask and the form's rejection (ADR-0110/ADR-0111), the collection's pick, capacity and disclosure (ADR-0112/ADR-0113/ADR-0114), the field's suggestion contract (ADR-0115), the sorted column's order (ADR-0116), the nested member's depth (ADR-0117), the set's current item (ADR-0118) — and the fact the AT turns into "opens a menu" while it walks the toolbar was the one opener fact the tree carried nothing for. The producer ships: [`Widget::has_popup`](crates/alacer-core/src/widget.rs) — `None` the honest absence (the attribute's unset state the `Option` spells, never a guessed target), the kind ([`HasPopupKind`](crates/alacer-core/src/widget.rs)) riding beside the word: `Menu` / `Listbox` / `Tree` / `Grid` / `Dialog`, AccessKit's own five enum values mirrored so the mapping is one branch with no translation.
- The producer joins without a seam change: [`Button::has_popup`](crates/alacer-widgets/src/button.rs) — the constructor word naming the initial target — beside [`set_has_popup`], the app's live door through the mutate seam (`None` the honest absence restored; the write that lands where the word already is queues nothing). The word is an opener word: the containers and collection members answer nothing (a container reports its members' words, a row reports its depth — the opener's target is the opener's own fact), so no producer joins there.
- The snapshot carries it (`TreeNode::has_popup`), and the sink maps AccessKit's `HasPopup` property (ARIA's `aria-haspopup`) — one branch beside the current word's, no patch-tier growth. The word prices the word and nothing else: the target is not a wall, the opener's press and focus routes the widget's own.
- The corpus grew 133 → 134 by the coordinated rule: `has_popup/snapshot_1k_has_popups` — the 1,000-node button arena carrying the target word — against the same arena bare (`current/snapshot_1k_buttons_bare`, the M114 case the M111 precedent reuses; the M95 pricing rule — the word is a button word, so the honest comparison is buttons against buttons, the M108/M110 denominator shape) — the `has_popup_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the snapshot it rides. The m-suite pins re-agree at 134.
- [`examples/has_popup.rs`](crates/alacer/examples/has_popup.rs) drives the beats from the consumer side: the menu button beside the combobox button, the tree toggle, the grid picker and the dialog opener, the plain button between them, the AT's press landing on the plain button untouched (the word names a target, it walls nothing), the live door moving the word — opening menus, switching to dialogs, clear — the honest absence restored.

### M114 — the current tier (ADR-0118)

- The word every navigation and wizard vocabulary speaks about which item the reader stands in — "this one is the page we're on, that step is where we are" — said by the tree at last. Through M113 the snapshot said where a member sits in its grid (the M6 table slot), what a control *does* (the description, ADR-0099), what an empty field *hints* (the placeholder, ADR-0100), the span a value rides (the range tier, ADR-0101), the form's ask and the form's rejection (ADR-0110/ADR-0111), the collection's pick, capacity and disclosure (ADR-0112/ADR-0113/ADR-0114), the field's suggestion contract (ADR-0115), the sorted column's order (ADR-0116), the nested member's depth (ADR-0117) — and the fact the AT turns into "current page" while it walks the nav was the one navigation fact the tree carried nothing for. The producer ships: [`Widget::current`](crates/alacer-core/src/widget.rs) — `None` the honest absence (ARIA's `false`, the attribute's absent default, the absence the `Option` spells, never a guessed flavor), the kind ([`CurrentKind`](crates/alacer-core/src/widget.rs)) riding beside the word: `True` / `Page` / `Step` / `Location` / `Date` / `Time`, AccessKit's own six enum values mirrored so the mapping is one branch with no translation.
- The producer joins without a seam change: [`Button::current`](crates/alacer-widgets/src/button.rs) — the constructor word naming the initial currency — beside [`set_current`], the app's live door through the mutate seam (`None` the honest absence restored; the write that lands where the word already is queues nothing). The word is an item word: the containers and collection members answer nothing (a container reports its members' words, a row reports its depth — the current item's currency is the item's own fact), so no producer joins there.
- The snapshot carries it (`TreeNode::current`), and the sink maps AccessKit's `AriaCurrent` property (ARIA's `aria-current`) — one branch beside the level word's, no patch-tier growth. The word prices the word and nothing else: the currency is not a wall, the item's press and focus routes the widget's own.
- The corpus grew 131 → 133 by the coordinated rule: `current/snapshot_1k_currents` — the 1,000-node button arena carrying the currency word — against the same arena bare (`current/snapshot_1k_buttons_bare`, the tier's own honest pair; the M95 pricing rule — the word is a button word, so the honest comparison is buttons against buttons, the M108/M110 denominator shape) — the `current_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the snapshot it rides. The m-suite pins re-agree at 133.
- [`examples/current.rs`](crates/alacer/examples/current.rs) drives the beats from the consumer side: the nav's page beside the wizard's step and the calendar's date, the plain button between them, the AT's press landing on the plain button untouched (the word names a currency, it walls nothing), the live door moving the word — landing, stepping, clear — the honest absence restored.

### M113 — the level tier (ADR-0117)

- The word every nested-collection vocabulary speaks about how deep a member stands — "this one is a level-2 row under the level-1 one" — said by the tree at last. Through M112 the snapshot said where a member sits in its grid (the M6 table slot), what a control *does* (the description, ADR-0099), what an empty field *hints* (the placeholder, ADR-0100), the span a value rides (the range tier, ADR-0101), the form's ask and the form's rejection (ADR-0110/ADR-0111), the collection's pick, capacity and disclosure (ADR-0112/ADR-0113/ADR-0114), the field's suggestion contract (ADR-0115), the sorted column's order (ADR-0116) — and the fact the AT turns into "level 2" while it walks the rows was the one nested-collection fact the tree carried nothing for. The producer ships: [`Widget::level`](crates/alacer-core/src/widget.rs) — `None` the honest absence (the flat row the absence keeps, never a guessed `0`), the count riding the word, zero-based — AccessKit's own base, the contract naming it verbatim beside ARIA's one-based `aria-level`, so the mapping is one branch with no translation and the base difference is the documentation's, never a silent cast.
- The producer joins without a seam change: [`TableRow::level`](crates/alacer-widgets/src/table.rs) — the constructor word naming the initial depth — beside [`set_level`], the app's live door through the mutate seam (`None` the honest absence restored, the flat row; the write that lands where the word already is queues nothing). The word is a member word: the headers beside it answer nothing (a header reports the column's order, not the row's depth), so no producer joins there.
- The snapshot carries it (`TreeNode::level`), and the sink maps AccessKit's `Level` property — one branch beside the sort word's, no patch-tier growth. The word prices the word and nothing else: the depth is not a wall, the member's press and focus routes the widget's own.
- The corpus grew 130 → 131 by the coordinated rule: `level/snapshot_1k_level` — the 1,000-node row arena carrying the depth word — against the same arena bare (`expanded/snapshot_1k_rows_bare`, the M110 case the M111 precedent reuses; the M95 pricing rule — the word is a row word, so the honest comparison is rows against rows, the M108/M110 denominator shape) — the `level_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the snapshot it rides. The m-suite pins re-agree at 131.
- [`examples/level.rs`](crates/alacer/examples/level.rs) drives the beats from the consumer side: the root-level row beside the level-1 row and the level-2 row, the flat row between them, the AT's press landing on the button beside the nested rows untouched (the word names a depth, it walls nothing), the live door moving the word — deeper, higher, clear — the honest absence restored.

### M112 — the sort direction tier (ADR-0116)

- The word every sorted-column vocabulary speaks about the rows it orders — "this one sorts ascending, that one descending" — said by the tree at last. Through M111 the snapshot said what a control *does* (the description, ADR-0099), what an empty field *hints* (the placeholder, ADR-0100), the span a value rides (the range tier, ADR-0101), the form's ask and the form's rejection (ADR-0110/ADR-0111), the collection's pick, capacity and disclosure (ADR-0112/ADR-0113/ADR-0114), the field's suggestion contract (ADR-0115) — and the fact the AT turns into "sorted ascending" while it walks the rows was the one sorted-column fact the tree carried nothing for. The producer ships: [`Widget::sort_direction`](crates/alacer-core/src/widget.rs) — `None` the honest absence (ARIA's `none` the absence the `Option` spells, never a guessed direction), the kind ([`SortDirection`](crates/alacer-core/src/widget.rs)) riding beside the word: `Ascending` / `Descending` / `Other`, AccessKit's own three enum values mirrored so the mapping is one branch with no translation.
- The producer joins without a seam change: [`ColumnHeader::sort`](crates/alacer-widgets/src/table.rs) — the constructor word naming the initial order — beside [`set_sort`], the app's live door through the mutate seam (`None` the honest absence restored; the write that lands where the word already is queues nothing). The word is a header word: the member and value roles answer nothing (a row reports its position, not the column's order), so no producer joins there.
- The snapshot carries it (`TreeNode::sort_direction`), and the sink maps AccessKit's `SortDirection` property (ARIA's `aria-sort`) — one branch beside the auto-complete word's, no patch-tier growth. The word prices the word and nothing else: the order is not a wall, the column's press and focus routes the widget's own.
- The corpus grew 128 → 130 by the coordinated rule: `sort/snapshot_1k_sort` — the 1,000-node header arena carrying the sort word — against the same arena bare (`sort/snapshot_1k_headers_bare`, the tier's own honest pair; the M95 pricing rule — the word is a header word, so the honest comparison is headers against headers, the M108/M110 denominator shape) — the `sort_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the snapshot it rides. The m-suite pins re-agree at 130.
- [`examples/sort.rs`](crates/alacer/examples/sort.rs) drives the beats from the consumer side: the ascending column beside the descending column and the order ARIA names no word for, the plain header between them, the AT's press landing on the button beside the sorted columns untouched (the word names an order, it walls nothing), the live door moving the word — descending, ascending, clear — the honest absence restored.


### M111 — the auto-complete tier (ADR-0115)

- The word every field vocabulary speaks about the suggestions it offers — "this one completes inline, that one lists them" — said by the tree at last. Through M110 the snapshot said what a control *does* (the description, ADR-0099), what an empty field *hints* (the placeholder, ADR-0100), the span a value rides (the range tier, ADR-0101), the form's ask (the required tier, ADR-0110), the form's rejection (the invalid tier, ADR-0111), the collection's pick and capacity (ADR-0112/ADR-0113), the member's disclosure (the expanded tier, ADR-0114) — and the fact the AT turns into "completions available" while the user types was the one field fact the tree carried nothing for. The producer ships: [`Widget::auto_complete`](crates/alacer-core/src/widget.rs) — `None` the honest absence (ARIA's `none` the absence the `Option` spells, never a guessed mode), the kind ([`AutoCompleteKind`](crates/alacer-core/src/widget.rs)) riding beside the word: `Inline` / `List` / `Both`, AccessKit's own three enum values mirrored so the mapping is one branch with no translation.
- The producer joins without a seam change: [`TextInput::auto_complete`](crates/alacer-widgets/src/text_input.rs) — the constructor word naming the initial contract — beside [`set_auto_complete`], the app's live door through the mutate seam (`None` the honest absence restored; the write that lands where the word already is queues nothing). The word is a field word: the value tier's non-editable controls answer nothing (a slider reports its value, not its suggestions), so no producer joins there.
- The snapshot carries it (`TreeNode::auto_complete`), and the sink maps AccessKit's `AutoComplete` property (ARIA's `aria-autocomplete`) — one branch beside the expanded word's, no patch-tier growth. The word prices the word and nothing else: the suggestion contract is not a wall, the field stays editable, the AT's write landing untouched (a tree that refused on the word's behalf would price a contract it does not own).
- The corpus grew 127 → 128 by the coordinated rule: `auto_complete/snapshot_1k_auto_complete` — the 1,000-node field arena carrying the suggestion contract — against the same arena bare (`placeholder/snapshot_1k_bare`, the M96 case; the M95 pricing rule — the word is a field word, so the honest comparison is fields against fields, the M106/M107 denominator) — the `auto_complete_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the snapshot it rides. The m-suite pins re-agree at 128.
- [`examples/auto_complete.rs`](crates/alacer/examples/auto_complete.rs) drives the beats from the consumer side: the field that lists its completions beside the field that completes inline and the plain field between them, the AT's write landing on the auto-completed field untouched (the word names a contract, it walls nothing), the live door moving the word — list, inline, clear — the honest absence restored.

### M110 — the expanded tier (ADR-0114)

- The word every collection vocabulary speaks about the member that opens and closes — "this one is expanded, that one collapsed" — said by the tree at last. Through M109 the snapshot said which member the collection picked (the selected tier, ADR-0112) beside the capacity the multiselectable tier named (ADR-0113), the position the M6 slot carried, the form's ask and the form's rejection beside them — and the fact the AT turns into "expanded" / "collapsed" while the user walks the rows was the one collection fact the tree carried nothing for. The producer ships: [`Widget::expanded`](crates/alacer-core/src/widget.rs) — the honest trio: `None` the member that opens and closes not (the absence ARIA spells `undefined`, never a guessed `false`), `Some(false)` the member standing collapsed, `Some(true)` the member standing open. AccessKit's own contract names the mapping verbatim ("setting this to `false` means the node is collapsed; omitting it means this state isn't applicable"), so the mapping is one branch with no translation.
- The producer joins without a seam change: [`TableRow::expanded`](crates/alacer-widgets/src/table.rs) — the constructor word naming the initial disclosure — beside [`set_expanded`], the app's live door through the mutate seam (`None` the honest absence restored; the write that lands where the word already is queues nothing). The word is a member word, never an inherited one: the cells beside it answer nothing (the disclosure is the row's), and a `Table` — the container — carries no disclosure to say, so no producer joins there.
- The snapshot carries it (`TreeNode::expanded`), and the sink maps AccessKit's `Expanded` property (ARIA's `aria-expanded`) — one branch beside the multiselectable word's, no patch-tier growth. The word prices the word and nothing else: expanded is not a wall, the word names state, the open and close routes the widget's own (a tree that refused on the word's behalf would price a contract it does not own).
- The corpus grew 125 → 127 by the coordinated rule — the tier's own honest pair: `expanded/snapshot_1k_expanded` (the 1,000-node row arena carrying the disclosure word) against the same arena bare (`expanded/snapshot_1k_rows_bare`) — the `expanded_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the snapshot it rides. The m-suite pins re-agree at 127, and the campaign re-agreed the stale fragments earlier cuts trailed: the README's tier-vocabulary table (four rows the M105–M109 campaigns left behind — read-only, required, selected, multiselectable) and its tier paragraph now read the M110 cut's truth.
- [`examples/expanded.rs`](crates/alacer/examples/expanded.rs) drives the beats from the consumer side: the row standing open beside the row standing collapsed and the wordless row, the AT's press landing on the button beside the expanded table untouched (the word names state, it walls nothing), the live door moving the word — open, collapse, clear — the honest absence restored.

### M109 — the multiselectable tier (ADR-0113)

- The word every collection vocabulary speaks about the container — "this one allows more than one pick" — said by the tree at last. Through M108 the snapshot said which member the collection picked (the selected tier, ADR-0112) beside the position the M6 slot carried, the form's ask and the form's rejection beside them — and the fact the AT turns into "you can pick more than one" before the user ever reaches the rows was the one collection fact the tree carried nothing for. The producer ships: [`Widget::multiselectable`](crates/alacer-core/src/widget.rs) — `false` the honest absence (the ADR-0087 presence rule at the tier, the read-only shape: a container word with no third state to protect, the flag the whole truth).
- The producer joins without a seam change: [`Table::multiselectable`](crates/alacer-widgets/src/table.rs) — the constructor word naming the capacity — beside [`set_multiselectable`], the app's live door through the mutate seam (`false` the honest absence restored; the write that lands where the word already is queues nothing). The word is a container word, never a member or widget word: the rows and cells beside it answer nothing (the selected tier's trio is theirs), and a `List` — the tree says `ScrollArea` for it — carries no collection role to size, so no producer joins there.
- The snapshot carries it (`TreeNode::multiselectable`), and the sink maps AccessKit's `Multiselectable` property (ARIA's `aria-multiselectable`) — one branch beside the selected word's, no patch-tier growth. The word prices the word and nothing else: multiselectable is not a wall, the word names capacity, the members' routes the widgets' own (a tree that refused on the word's behalf would price a contract it does not own).
- The corpus grew 123 → 125 by the coordinated rule — the tier's own honest pair: `multiselectable/snapshot_1k_multiselectable` (the 1,000-node table arena carrying the capacity word) against the same arena bare (`multiselectable/snapshot_1k_tables_bare`) — the `multiselectable_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the snapshot it rides. The m-suite pins re-agree at 125.
- [`examples/multiselectable.rs`](crates/alacer/examples/multiselectable.rs) drives the beats from the consumer side: the multi-pick table (its first row the pick, the two tiers' words beside each other in one snapshot) beside the single-pick table, the AT's press landing on the button beside the multiselectable table untouched, the live door moving the word — declare, clear — the honest absence restored.

### M108 — the selected tier (ADR-0112)

- The word every collection vocabulary speaks about the member — "this one is the pick" — said by the tree at last. Through M107 the snapshot said where a member sits (the M6 table slot), what a control *does* (the description, ADR-0099), what an empty field *hints* (the placeholder, ADR-0100), the span a value rides (the range tier, ADR-0101), the form's ask (the required tier, ADR-0110), the form's rejection (the invalid tier, ADR-0111) — and the fact the AT turns into "selected" while the user walks the rows was the one collection fact the tree carried nothing for. The producer ships: [`Widget::selected`](crates/alacer-core/src/widget.rs) — the honest trio: `None` the member of no selection collection (the absence ARIA spells `undefined`, never a guessed `false`), `Some(false)` the member standing unpicked beside its picked siblings, `Some(true)` the pick.
- The producers join without a seam change: [`TableRow::selected`](crates/alacer-widgets/src/table.rs) — the constructor word naming the initial pick — beside [`set_selected`], the app's live door through the mutate seam (`None` the honest absence restored; the write that lands where the word already is queues nothing), with [`TableCell::selected`] and [`TableCell::set_selected`] riding beside it (the cell tier beside the row tier, the trait door already answering both). The word is a member word, never an inherited one: a cell that declared no word of its own announces nothing, the row's word never bleeds into it.
- The snapshot carries it (`TreeNode::selected`), and the sink maps AccessKit's `Selected` property (ARIA's `aria-selected`) — one branch beside the invalid word's, no patch-tier growth. The word prices the word and nothing else: selected is not a wall, the word names state, the pick's press and focus routes the widget's own (a tree that refused on the word's behalf would price a contract it does not own).
- The corpus grew 121 → 123 by the coordinated rule — the tier's own honest pair: `selected/snapshot_1k_selected` (the 1,000-node row arena carrying the pick word) against the same arena bare (`selected/snapshot_1k_rows_bare`) — the `selected_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the snapshot it rides. The m-suite pins re-agree at 123, and the campaign re-agreed the stale fragments earlier cuts trailed: m9's corpus doc comment (read "48 criterion cases" since M10) and the "94-case corpus" panic messages nine suites carried now read the M108 cut's truth.
- [`examples/selected.rs`](crates/alacer/examples/selected.rs) drives the beats from the consumer side: the picked row beside the row standing unpicked and the wordless row, the AT's press landing on the button beside the selected table untouched, the live door moving the word — pick, unpick, clear — the honest absence restored.

### M107 — the invalid tier (ADR-0111)

- The word every form vocabulary speaks when the value lands wrong — "this one is not valid as given" — said by the tree at last. Through M106 the snapshot said what a control *does* (the description, ADR-0099), what an empty field *hints* (the placeholder, ADR-0100), the span a value rides (the range tier, ADR-0101), the form's ask (the required tier, ADR-0110) — and the fact the AT turns into "invalid" while the user edits was the one form fact the tree carried nothing for. The producer ships: [`Widget::invalid`](crates/alacer-core/src/widget.rs) — `None` the honest absence (the ADR-0087 presence rule at the tier), the kind riding beside the word.
- The kind is the neutral payload type the tree carries: [`InvalidKind`](crates/alacer-core/src/widget.rs) — `True` the bare rejection (`aria-invalid="true"`), `Grammar` and `Spelling` the prose-error kinds the AT reads natively — ARIA's own three values, mirrored so the mapping is one branch with no translation.
- The producers join without a seam change: [`TextInput::invalid`](crates/alacer-widgets/src/text_input.rs) — the constructor word naming the initial word and kind — beside [`set_invalid`], the app's live door through the mutate seam (`None` the honest absence restored; the write that lands where the word already is queues nothing), with [`Slider::invalid`] and [`Checkbox::invalid`] overriding beside it (the trait door already answers them). The snapshot carries it (`TreeNode::invalid`), and the sink maps AccessKit's `Invalid` property (ARIA's `aria-invalid`) — one branch beside the required flag's, no patch-tier growth.
- The word prices the word and nothing else: an invalid control's doors answer exactly as they always did — invalid is not a wall, the value stays editable, the re-submit the app's own contract (the AT reads the rejection *while* the user edits, which is the whole point of the word). The tier grows no refusal vocabulary and prices no behavior beyond the word itself.
- The corpus grew 120 → 121 by the coordinated rule: `invalid/snapshot_1k_invalid` — the 1,000-node invalid field arena against the same arena bare (`placeholder/snapshot_1k_bare`, the M96 case; the M95 pricing rule) — the `invalid_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the snapshot it rides. The m-suite pins re-agree at 121, and the M107 campaign re-agreed the pins earlier cuts trailed: the m-suite corpus pins and the README fragments the M102–M106 campaigns left behind ("All 118 criterion cases", "M0 through M102", "a hundred and seven decision records", m20's gates-len pin) now read the M107 cut's truth.
- [`examples/invalid.rs`](crates/alacer/examples/invalid.rs) drives the beats from the consumer side: the misspelled card field (`spelling`), the out-of-contract amount slider (`true`), the mismatched agree checkbox (`grammar`) beside the valid note, the AT's write landing on the invalid field untouched, the live door moving the word and its kind.

### M106 — the required tier (ADR-0110)

- The word every form vocabulary speaks before the submit fires — "this one must be filled" — said by the tree at last. Through M105 the snapshot said what a control *does* (the description, ADR-0099), what an empty field *hints* (the placeholder, ADR-0100), the span a value rides (the range tier, ADR-0101), whether the value is the app's (the read-only tier, ADR-0109) — and the ask the AT turns into "required" on every platform was the one form fact the tree carried nothing for. The producer ships: [`Widget::required`](crates/alacer-core/src/widget.rs) — `false` the honest absence (the ADR-0087 presence rule at the tier).
- The producers join without a seam change: [`TextInput::required`](crates/alacer-widgets/src/text_input.rs) — the constructor word — beside [`set_required`], the app's live door through the mutate seam, with [`Slider::required`] and [`Checkbox::required`] overriding beside it (the trait door already answers them). The snapshot carries it (`TreeNode::required`), and the sink maps AccessKit's `Required` flag (ARIA's `aria-required`) — one branch beside the read-only flag's, no patch-tier growth.
- The word prices the word and nothing else: a required control's doors answer exactly as they always did — required is not a wall, the field stays editable, the empty submit the app's own contract (the AT reads the declaration *before* it asks, which is the whole point of the word). The tier grows no refusal vocabulary and prices no behavior beyond the flag itself.
- The corpus grew 119 → 120 by the coordinated rule: `required/snapshot_1k_required` — the 1,000-node required field arena against the same arena bare (`placeholder/snapshot_1k_bare`, the M96 case; the M95 pricing rule) — the `required_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the snapshot it rides. The m-suite pins re-agree at 120.
- [`examples/required.rs`](crates/alacer/examples/required.rs) drives the beats from the consumer side: the required card field, amount slider and agree checkbox beside the optional note, the AT's write landing on the required field untouched, the live door moving the word.

### M105 — the read-only tier (ADR-0109)

- The declared-but-unreported word the write walls priced, said by the tree at last. Since M22 (ADR-0028) the availability tier named the control that refuses every door including focus; since M55 (ADR-0061) the AT's write door streamed its refusals; since M62 (ADR-0068) the typing door priced its walls as named species — and the field every word-processing app ships (the license key, the computed total, the log the app owns) had no word: it reads, selects, copies and walks its caret like any other field while refusing only the writes. The producer ships: [`Widget::read_only`](crates/alacer-core/src/widget.rs) — `false` the honest absence (the ADR-0087 presence rule at the tier).
- The producer joins: [`TextInput::read_only`](crates/alacer-widgets/src/text_input.rs) — the constructor word naming the initial state — beside [`set_read_only`], the app's live door through the mutate seam (the caller's `PAINT | A11Y` request pays the re-announcement; the write that lands where the word already is queues nothing). The snapshot carries it (`TreeNode::read_only`), and the sink maps AccessKit's `ReadOnly` flag (ARIA's `aria-readonly`) — one branch beside the disabled flag's, no patch-tier growth (the word is runtime state the app's door moves, never a declarable DSL word).
- The walls are the doors' own vocabulary: the typing door's classifier routes the edit asks it already names (typing, deletes, the word chords, paste, cut) to [`KeyEditRefusal::ReadOnly`](crates/alacer-core/src/inspect.rs) — the ask the open door would have answered streaming behind its record, the shape priced at nothing; the IME's commit streams the same species beside the IME driver; the AT's `SetValue` / `ReplaceSelectedText` refuse before arithmetic with [`TextWriteRefusal::ReadOnly`], the selection door answering beside the wall. The journal's chords (undo, redo) are value writes the wire vocabulary did not name: [`KeyEditAction::Undo`] and [`KeyEditAction::Redo`] join the action set (the additive wire rule ADR-0010 names), so the wall prices them like every other ask. The submit's Enter consumes and moves nothing (the M62 rule: the wall prices edit asks, and the submit's Enter asks nothing).
- The availability tier stays the outer one: a field both disabled and read-only never reaches the read-only wall — the unavailable control holds no focus, the key route's boundary answers first, and the read-only word rides the tree beside the availability's. The app's own `set_value` never asked the field's permission: the tier walls user doors, not the app's.
- The corpus grew 118 → 119 by the coordinated rule: `read_only/snapshot_1k_read_only` — the 1,000-node read-only field arena against the same arena bare (`placeholder/snapshot_1k_bare`, the M96 case; the M95 pricing rule) — the `read_only_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the snapshot it rides. The m-suite pins re-agree at 119, and the published baseline re-agrees with its own pins: the M103/M104 cases the baseline file trailed (`click_chain/seamed_chain_1k`, `click_chain/manual_chain_1k`, `pass_overhead/timed_idle_10k`, `pass_overhead/seamed_timed_idle_10k`) join beside the tier's own, measured medians from fresh criterion data — no ungated case, and no unpublished gate.
- [`examples/read_only.rs`](crates/alacer/examples/read_only.rs) drives the beats from the consumer side: the locked license field beside the editable draft, the unlock door moving the word live, the status line mirroring the words the tree says.

### M104 — the pipeline's frame timing rides the time seam (ADR-0108)

- ADR-0107's seam cashed at the last hardcoded read: the M1 gate instrumentation's nine `Instant::now` sites — the ledger the benches gate against, the overhead test asserts, and the frame report carries — answered wall noise no test could assert exactly and no demo could print twice. The producer ships: [`PassPipeline::set_clock`](crates/alacer-core/src/passes.rs) — the door beside `set_timings` — installing one [`TimeSeam`] every timing read answers; with no clock installed the fallback anchor (the pipeline's own construction instant) still prices real time, byte-for-byte the pre-seam tick.
- The timed frame is exact data: a stepping seam clock prices every pass a whole step and two idle ticks price identically (`a_seam_clock_prices_two_ticks_identically` — the ledgers equal, as data); a frozen [`ManualClock`] prices the whole frame zero; `set_clock` replaces (the newest install wins); and without a seam the fallback still measures (`without_a_seam_the_tick_still_measures_wall_time`). The read-count structure is kept, not invented — the due-gated passes read only when due (a skipped pass prices zero and pays no read), and the tests assert whole steps and same-class determinism, never a fixed read count.
- No patch-tier growth and no trail tier: the clock is instrumentation state the door swaps (the M98/M99 rule at the timing tier), and a clock read prices what the frame already did — no side effect, no record (the ADR-0107 rule at the pipeline tier).
- The corpus grew 116 → 118 by the coordinated rule: `pass_overhead/timed_idle_10k` — the timed idle tick on the 10,000-node static tree, the shipped fallback's own price — beside `pass_overhead/seamed_timed_idle_10k`, the same tick with the deterministic clock behind the door — the `timed_tick_seam_overhead` ratio budget (≤ 1.25×) holding the seam's indirection at the fallback's own class. The m-suite pins re-agree at 118.
- [`examples/frame_clock.rs`](crates/alacer/examples/frame_clock.rs) drives the beats from the consumer side: the fallback's two unequal totals (the wall clock's own honesty), the stepping clock's byte-identical ledger printed as a table, and the frozen clock's deterministic zero.

### M103 — the time seam (ADR-0107)

- The one external effect with no seam cashed: the clock. Two contracts read a clock and both hardcoded `Instant::now` — the platform's click chain, whose own M13 test block carried the pre-registered gap since ADR-0019 ("the time bound rides `Instant` and is not sleep-tested; the distance bound is"), the double-click window the one input contract CI never priced. The producer ships: [`TimeSeam`](crates/alacer-core/src/time.rs) — one required read, `now_nanos` (monotonic nanos since the clock's own anchor, never backwards), durations the caller's arithmetic (differences of two reads, the shape both consumers already ride) — no sleep, no deadline, no wall-calendar read.
- The provided implementations ride the provided-implementation rule: [`MonotonicClock`] — the canon, real time off `Instant`, anchored at construction, the default build's shape — beside [`ManualClock`] — the deterministic seam, a shared `Arc<AtomicU64>` reading: a clone shares the one reading, the test advances through its handle while the consumer reads through theirs. A frozen clock prices every duration exactly zero; an advanced one prices exactly what the advance wrote.
- The click chain rides the seam: [`ClickCounter`](crates/alacer-platform/src/lib.rs) holds `Box<dyn TimeSeam>` and prices `now − then ≤ DOUBLE_CLICK_NANOS` (the window's own width in the seam's unit — one constant, never a second clock-shaped fact), constructing with the canon clock, byte-for-byte today's behavior. The M13 gap closes the honest way: four tests install a manual clock and price the window exactly — 200 ms chains, 500 ms starts fresh, exactly 400 ms chains (the ≤ bound), the triple counts before a stale chain resets — no sleeps, the distance bound's tests standing unchanged beside them.
- The seam is public vocabulary: `TimeSeam`, `MonotonicClock`, `ManualClock` in core's prelude (the `KeymapSeam`/`SettingSeam` placement) and the facade's re-exports — app code that needs elapsed time holds one seam and never calls `Instant::now`, the chain's own doc's axiom ("the widget never owns the clock") extended to the app's own code. No trail tier: a clock read prices what the frame already did — it writes nothing, queues nothing, carries no payload a devtools surface needs.
- The corpus grew 114 → 116 by the coordinated rule: `click_chain/seamed_chain_1k` — 1,000 chained presses through the canon clock, the shipped default's per-press price (the clock read is the chain's own cost) — beside `click_chain/manual_chain_1k`, the same storm through the deterministic clock, the seam indirection's floor. The pair's ratio is the vDSO read's own price, not the seam's, so no ratio budget rides it: both cases publish regression gates, the per-case medians the perf contracts hold. The m-suite pins re-agree at 116.
- [`examples/time_seam.rs`](crates/alacer/examples/time_seam.rs) drives the beats from the consumer side: the app's only time read is one seam read — inject a `ManualClock`, advance exactly 1.5 s, and the tree says `1.500 s elapsed` in every CI run — with the canon clock answering the same code and never going backwards.
- The inspector's attach loop names its spawn refusal (the ADR-0029 discipline at the accept loop's own tier): a failed thread spawn now warns at `alacer::inspect` with the error — the dead panel's one clue — instead of swallowing it into a silent `None`.

### M102 — the reveal tier (ADR-0106)

- ADR-0104's first pre-registered re-open rule cashed: the "show password" toggle said by the tree. Since M100 a secured field's tree answer was `PasswordInput` and a masked value, and the constructor word was the whole story — the consumer's toggle flipped the pixels while the tree kept the masked answer: the AT read `PasswordInput` and the mask while the user read the secret on screen. The flag grows the revealed state beside the constructor's: [`Widget::secure_revealed`](crates/alacer-core/src/widget.rs) — a compound word, meaningful only where the secure word holds, a plain field answering `false` whatever its toggle carries (the honest absence, never a munged half).
- The producer ships: `TextInput`'s `revealed()` builder naming the initial state, and [`set_revealed`](crates/alacer-widgets/src/text_input.rs) — the app's live door through the mutate seam, the caller's `PAINT | A11Y` request paying the re-announcement. The snapshot carries it (`TreeNode::secure_revealed`), and the value the tree resolves follows it: the mask yields to the revealed state, the field's ordinary value riding the tree again — the pixels and the tree telling one truth.
- The sink's mapping one branch beside the one it has, exactly the mapping ADR-0104 pre-registered: `PasswordInput` only while the field is secured *and not revealed* — AccessKit's own contract ("when the password is revealed, use `Role::TextInput` instead"); the mask half is the same contract at the value tier (the masking requirement is `PasswordInput`'s, and the field no longer is one).
- No patch-tier growth: the revealed state is runtime state the app's door moves, not a declarable word — the DSL's `secure` word builds the same constructor, the `revealed()` builder names the initial state through the same element.
- The corpus grew 113 → 114 by the coordinated rule: `secure_revealed/snapshot_1k_revealed` — the 1,000-node revealed field arena against the same secured arena masked (`secure/snapshot_1k_secured`, the M100 case; the M95 pricing rule) — the `secure_revealed_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the measured class. The m-suite pins re-agree at 114.
- [`examples/revealed.rs`](crates/alacer/examples/revealed.rs) drives the beats from the consumer side: the masked field beside the revealed one, the hide restoring the M100 answer, the reveal yielding the mask to the value, the status line mirroring the words the tree says.

### M101 — the scroll position tier (ADR-0105)

- The declared-but-unreported fact cashed: where a scroll container *is*. Since M50 the scroll doors land offsets and clamp them against declared spans (ADR-0056), since M89 the container keeps both axes' stories (ADR-0093), since M98 the tree says which axis the scrolls ride (ADR-0102) — and the tree never said the position and extent the doors resolve: the AT read the track but not the carriage on it. The producer ships: [`Widget::scroll_position`](crates/alacer-core/src/widget.rs) beside the axis word the tree already said — each axis riding its own honesty (the ADR-0087 presence rule at the scroll position tier): an empty span (the content fits the viewport, or the width story was never declared) carries no position — the same wall every scroll door answers `no_scrollable_content` with — never a guessed zero, and a container with no live axis at all answers `None`.
- The snapshot carries it (`TreeNode::scroll_position` — the offset beside the wall it clamps to, one type per axis pair: `ScrollPosition`, `ScrollAxisPosition`), and the sink maps AccessKit's `ScrollY` / `ScrollYMin` / `ScrollYMax` properties — the `ScrollX` halves beside them on a two-axis container — the arithmetic every platform AT turns into "scrolled 200 of 0..600". The floor the funnel clamps to is carried, not guessed: the clamp arithmetic's own zero, the range tier's wall discipline (ADR-0101) at the scroll tier.
- The tree's currency rule is the wheel's own: the doors that pay the a11y currency (the wheel, the AT's asks, bring-into-view) merge `PAINT | A11Y` on the container, so the position re-announces the frame the offset moved; the app's own handle door rides the next paid pass — the M90 trail rule, stated for the tree half too. No patch-tier growth: the position is runtime state the doors resolve, never a declarable word (the M98/M99 rule).
- The corpus grew 112 → 113 by the coordinated rule: `scroll_position/snapshot_1k_scrolled` — the 1,000-node scrolled list arena against the same arena's axis-word case (`orientation/snapshot_1k_vertical`, the M98 case; the M95 pricing rule) — the `scroll_position_snapshot_overhead` ratio budget (≤ 1.25×) holding the position's cost at the measured class: the two axis options the node builds beside the orientation word's own. The m-suite pins re-agree at 113.
- [`List`](crates/alacer-widgets/src/list.rs) produces the fact beside the doors it already answers by; `examples/scroll_position.rs` drives the beats from the consumer side: the scrolled list beside the bare one, the AT's own page ask landing the position the status line reads back, and the wheel paying the currency natively.

### M100 — the secure tier (ADR-0104)

- The declared-but-unreported fact cashed: the field's secret entry — `TextInput::secure()` naming the IME contract's `ImePurpose::Secure` since M4, the platform suppressing preedit echo and suggestions — rode the construction site while the a11y tree read a plain text input whose value was the raw secret itself, a payload no tree carries (the clipboard trail's rule, ADR-0045, at the tree tier). The producer ships: [`Widget::secure_entry`](crates/alacer-core/src/widget.rs) beside the constructor that already told the truth — `false` the honest absence, a non-input role answers nothing, never a guessed flag — `TextInput` overriding, the snapshot carrying it (`TreeNode::secure_entry`) *and* masking the value the tree resolves for a secured field (one masking character per code point of the committed value, never the secret), and the sink mapping it onto AccessKit's `PasswordInput` role — the word every platform AT announces, whose own contract ("a `PasswordInput` node's value must only contain masking characters") the tree now satisfies instead of violating.
- The declaration is the constructor's: there is no `set_secure` door, the way the IME purpose itself is constructor-immutable — a re-render that repeats the `secure` word queues nothing. The AT's write door stays open (a password manager fills the field); the read the tree resolves is the mask, its length tracking the committed value's code points (the m100 suite pins both).
- The DSL's `secure` word (parsed since the M0 registry) declares through the same door — no new prop class to pay for.
- The corpus grew 111 → 112 by the coordinated rule: `secure/snapshot_1k_secured` — the 1,000-node secured input arena against the same input arena bare (`placeholder/snapshot_1k_bare`, the M96 bare case; the M95 pricing rule) — the `secure_snapshot_overhead` ratio budget (≤ 1.25×) holding the word's cost at the measured class: the bool and the mask the node builds beside the value's own copy, the same-length allocation either way. The m-suite pins re-agree at 112.
- [`examples/secure.rs`](crates/alacer/examples/secure.rs) drives the beats from the consumer side: the secured field beside the bare one, the *Fill* write landing the mask the tree carries, and the status line mirroring the word the tree says.

### M99 — the numeric tier (ADR-0103)

- The declared-but-unreported fact cashed: the number behind the value's spoken word — every value-tier widget has reported its scalar through the M3 `value()` contract as *text*, and since M97 the tree carries the span the writes funnel through, but the AT read "40" (`aria-valuetext`) and never the number 40 (`aria-valuenow`): the automation and voice-control tier that reads range widgets as numbers read prose. The producer ships: [`Widget::numeric_value`](crates/alacer-core/src/widget.rs) beside the range it answers — `None` the honest absence (the ADR-0087 presence rule at the numeric tier: a text field's "16 digits on the front" is not 16, a valueless role announces nothing, never a parsed guess) — `Slider` and `Progress` overriding, the snapshot carrying it (`TreeNode::numeric_value`), and the sink mapping it onto AccessKit's `NumericValue` property (ARIA's `aria-valuenow`) beside the walls and step the M97 mapping already sets — one setter, exactly the shape ADR-0101's re-open rule pre-registered.
- The text rides exactly as it always has: the number is *beside* the spoken word, never instead of it — the pair landing together through the funnel the drag, the arrows and the AT's `SetValue` ride, and the write that lands where the value already is queues nothing (the M20 idle-frame property, re-pinned by the m99 suite).
- No patch tier growth: the number is derived, not declared — the scalars move through the doors the widgets have answered since M20 and M21, and there is no `set_numeric_value` door. A re-render that repeats the same `value` word queues nothing.
- The corpus grew 110 → 111 by the coordinated rule: `numeric/snapshot_1k_valued` — the 1,000-node value arena whose every node answers the number against the no-fact 1,000-node toggle arena the M94 binary case already carries (the M95 pricing rule) — the `numeric_snapshot_overhead` ratio budget (≤ 1.25×) holding the number's cost at the measured class: the `f64` option the node builds beside the label's own. The m-suite pins re-agree at 111.
- [`examples/numeric.rs`](crates/alacer/examples/numeric.rs) drives the beats from the consumer side: the slider and the report whose numbers ride the funnel beside the text, and the *Snap to max* write that moves both shapes of the answer together.

### M98 — the orientation tier (ADR-0102)

- The declared-but-unreported fact cashed: the axis every scroll door has answered on since M50 — `List::new(height)` naming the y axis the viewport's only scrollable story, `List::two_axis(width, height)` declaring both (ADR-0093), the x doors answering `no_scrollable_content` against a vertical-only list's empty span, the AT's own `ScrollLeft`/`ScrollRight` traveling the same unit contract — rode the construction site while the a11y tree read "scroll area" and not the axis: the AT's user heard a scrollable region where the sighted user saw a track with a direction. The producer ships: [`Widget::scroll_orientation`](crates/alacer-core/src/widget.rs) beside the constructors that already told the truth — `None` the honest absence (the ADR-0087 presence rule at the orientation tier), the two-axis answer the word ARIA spells `undefined`, never a guessed half — `List` overriding, the snapshot carrying it (`TreeNode::scroll_orientation`), and the sink mapping it onto AccessKit's `Orientation` property (ARIA's `aria-orientation`).
- The declaration is the constructor's: the viewport half the list's shared state carries is written by no door — the handle's doors move the offsets and the content spans, never the viewport the widget was built with — so the answer is constructor-fixed. A scroll moves the offset and moves nothing the tree says about the axis (the m98 suite pins the declaration surviving the scroll).
- No patch-tier growth: the declaration is constructor-immutable — there is no `set_orientation` door — and the `height` word the list element has parsed since the M0 registry builds the vertical-only constructor, so the DSL's lists declare through the same door with no new prop class to pay for.
- The corpus grew 109 → 110 by the coordinated rule: `orientation/snapshot_1k_vertical` — the 1,000-node vertical list arena against the no-fact 1,000-node toggle arena the M94 binary case already carries (the M95 pricing rule) — the `orientation_snapshot_overhead` ratio budget (≤ 1.25×) holding the declaration's cost at the measured class: the enum's option the node builds beside the label's own. The m-suite pins re-agree at 110.
- [`examples/orientation.rs`](crates/alacer/examples/orientation.rs) drives the beats from the consumer side: the vertical list's declared axis beside the two-axis list's honest absence, and the scrolls that move the offsets while the axis words stay.

### M97 — the range tier (ADR-0101)

- The declared-but-unreported fact cashed: the span every value-tier write has funneled through since M20 — `Slider::new(name, min, max)` and `Progress::new(name, min, max)` refuse a degenerate range outright, and every door (the drag, the arrows, Home/End, the AT's own `SetValue`/`Increment`/`Decrement`) lands where the clamp and the snap put it — rode the construction site while the a11y tree read the value ("42") and not the span: the AT's user heard a bare number, not "42 of 0..100, step 5". The producer ships: [`Widget::value_range`](crates/alacer-core/src/widget.rs) beside the clamp that already told the truth — `None` the honest absence (the ADR-0087 presence rule at the range tier), the step `Option` because the report half declares where its value lives, not how far one act moves it — `Slider` and `Progress` overriding, the snapshot carrying it (`TreeNode::value_range`), and the sink mapping it onto AccessKit's `MinNumericValue`/`MaxNumericValue` beside the `NumericValueStep` the slider declares (ARIA's `aria-valuemin`/`aria-valuemax`/the step).
- No patch-tier growth: the range is constructor-immutable — `Slider::new` owns the span's invariant, there is no `set_range` door — and the `min`/`max`/`step` words the slider and progress elements have parsed since M32 land where every other declared fact lands: the tree, at mount. A re-render that repeats the same words queues nothing (an idle frame; the m97 suite pins the counters' absence).
- The corpus grew 108 → 109 by the coordinated rule: `range/snapshot_1k_ranged` — the ranged 1,000-node slider arena against the no-range 1,000-node toggle arena the M94 binary case already carries (the M95 pricing rule) — the `range_snapshot_overhead` ratio budget (≤ 1.25×) holding the span's cost at the measured class: the copy the node builds beside the label's own. The m-suite pins re-agree at 109.
- [`examples/ranges.rs`](crates/alacer/examples/ranges.rs) drives the beats from the consumer side: the ranged slider beside the honest-absence report, and the *Snap to max* write landing on the wall the tree carries.

### M96 — the placeholder tier (ADR-0100)

- The declared-but-unreported fact cashed: the hint an empty field paints has ridden the canvas since M6, and the a11y tree read an empty value where the sighted user read the hint. The producer ships: [`Widget::placeholder`](crates/alacer-core/src/widget.rs) beside the paint that already told the truth — `None` the honest absence, an empty placeholder the absence, never an empty hint — `TextInput` overriding, the snapshot carrying it (`TreeNode::placeholder`), and the sink mapping it onto AccessKit's `Placeholder` property. The property stays mounted once the value lands: the value is what the AT reads first, the painter stops drawing the hint when the field is non-empty.
- The write is honest: `TextInput::set_placeholder` grows the moved answer — the write that lands where the hint already is queues nothing (the M21 idle-frame property, at the placeholder tier), the registry patcher's `visual` riding the widget's own comparison, never an unconditional write.
- The view's patch tier grows the **a11y-*also* prop class** beside M95's a11y-only one: the hint the pixels *do* carry keeps the paint masks beside the re-announcement the a11y class adds — the tree's truth and the painted hint move together, each tier paying its own mask, the re-render that lands where the hint already is an idle frame.
- The corpus grew 104 → 108 by the coordinated rule: `placeholder/snapshot_1k_placeholdered` beside `placeholder/snapshot_1k_bare` — the hinted 1,000-node input arena against the same arena bare — the `placeholder_snapshot_overhead` ratio budget (≤ 1.25×) holding the hint's cost at the measured class: the second bounded copy the node builds beside the label's own. Beside the pair, the full-corpus re-publish repaired the two orphans bench code had carried unpublished (`canvas/paint_100_canvases` and `inspect/tick_with_recorder_attached` — the M5 rule's own gap, gated now). The m-suite pins re-agree at 108.
- [`examples/placeholders.rs`](crates/alacer/examples/placeholders.rs) drives the beats from the consumer side: the hinted field beside the honest absence, and the *Swap hint* patch paying both masks the tier costs.

### M95 — the description tier (ADR-0099)

- The vocabulary's declared-but-unproduced property cashed: AccessKit's `Description` — ARIA's `aria-description`, the help text a screen reader reads when the name alone does not answer "what does this control do?" — declared at the boundary and produced by no widget. The producer ships: [`Widget::description`](crates/alacer-core/src/widget.rs) beside the name it explains — `None` the honest absence (the ADR-0087 presence rule at the description tier) — the snapshot carrying it (`TreeNode::description`), and the sink mapping it.
- The producers: `Button::description`, `Checkbox::description`, and `TextInput::description` mount the help text; each carries the one programmatic write (`set_description(Option<String>)`) answering whether it moved — the write that lands where the text already is queues nothing (the M21 idle-frame property, at the description tier).
- The view DSL grows the `description:` word, and the patch tier grows the **a11y-only prop class**: a changed `description` word queues exactly `Change::A11Y` — the re-announcement without the repaint — and counts in the patch report (the M72 rule: one fact per patch that moved something). The pixels never pay for a fact the painter did not write.
- The corpus grew 103 → 104 by the coordinated rule: `describe/snapshot_1k_described` — the same 1,000-node checkbox arena the toggle route's binary case carries, every node described — the `describe_snapshot_overhead` ratio budget (≤ 1.25×) holding the description's cost at the measured class: the second `String` the node builds beside the label's own. The m-suite pins re-agree at 108 (the M96 pair and the two repaired orphans joining the cut).
- [`examples/descriptions.rs`](crates/alacer/examples/descriptions.rs) drives the beats from the consumer side: the described canon beside the honest absences, and the description-only patch the pixels never pay for.

### M94 — the checkbox grows the mixed state (ADR-0098)

- The vocabulary's declared-but-unproduced state cashed: `ToggleState::Mixed` — "partially checked (tri-state parents)" — has ridden the a11y sink's M6 mapping since the tree could announce it, and no widget has ever produced one; the select-all parent was a story the tree could carry and the widget set could not tell. The producer ships: [`Checkbox::state`](crates/alacer-widgets/src/checkbox.rs) mounts the tri-state beside the binary `checked` door it keeps, `toggle_state()` and `is_mixed()` read it, and `toggled()` answers the full vocabulary the tree announces.
- The resolution rule is the cut's own decision: the mixed state is a *summary*, never a third bit — every user door (the pointer's click, the keyboard's Space, the AT's default press) resolves a mixed summary **fully checked**, the handler receiving the resolved boolean and the second press toggling the resolved bit like any checkbox. The programmatic door ([`Checkbox::set_state`](crates/alacer-widgets/src/checkbox.rs)) writes the summary verbatim — the view-reconciliation path the patch tier drives, firing no handler, the write tier queueing its own masks (the m22 discipline).
- The painter tells the tri-state truth: the mixed box fills with the accent and draws the horizontal dash — deterministic quads beside the check mark's strokes, the same font-free discipline the snapshot corpus ships by, the look telling the truth the tree carries (the M22 rule at the paint tier).
- The view DSL grows the `state:` word (`"on"` / `"off"` / `"mixed"`): the checkbox element's create and patch parse the vocabulary, unknown words refused by absence (the patcher applies only the props it understands), the binary `checked` flag keeping its contract beside the word.
- The corpus grew 101 → 103 by the coordinated rule: `toggle/snapshot_1k_tri_state` beside `toggle/snapshot_1k_binary` — the same roles, the same names, the same 1,000-node arenas through the real pipeline's a11y snapshot pass — the `toggle_tri_state_overhead` ratio budget (≤ 1.10×) holding the summary's cost at the route's noise floor. The m-suite pins re-agree at 103.
- [`examples/tri_state.rs`](crates/alacer/examples/tri_state.rs) drives the beats from the consumer side: the parent's summary recomputing from the tasks on every re-render, the resolution cascading in one write batch, and the patch channel answering with exactly one scoped re-render (the todo canon's own shape).

### M93 — the layout seam's call ledger (ADR-0097)

- The seam's own probe has carried one number since M1 (`measure_calls`, the translation-only probe every scroll milestone priced itself by); every other fact of the call was invisible. The alloc ledger's own precedent (A7, M84) names the shape: `alacer_layout::recording::RecordingLayout` wraps the engine the assembly site chose, and one `LayoutCallRecord` lands per call — the root asked, the dirty set's size, the viewport the clamp priced against, the outcome (the error's own message riding a refusal, never a bare flag).
- The ledger is a bounded window: records beyond the cap evict their oldest and count the eviction on the handle — the A7 window discipline the trail ring answers by (a ledger nobody drains never leaks, it forgets and says so). The reading handle clones beside the install, so the test reads after the harness has taken the engine's ownership.
- The harness door beside the constructors: `TestHarness::with_layout(width, height, engine)` — the trail door's own shape (`trail`/`trail_with_cap`), the caller's engine under the same passes the production host drives, the seams installing first exactly as `new` does.
- The counters delegate: `measure_calls` answers through the wrapper, the reset reaches the engine — the probe the m-suite already asserts through keeps its meaning under the instrument. The instrument is installed by the binary that wants it, never by the framework (the A7 rule the alloc ledger shipped by).
- The corpus grew 99 → 101 by the coordinated rule: `recording/wrapper_1k_dirty_10pct` beside `recording/bare_wrapper_1k_dirty_10pct` — the wrapper against the bare engine on the same incremental route (a 1,000-node arena, ten percent dirty per frame) — the `recording_wrapper_overhead` ratio budget (≤ 1.15×) holding the instrument's cost at the route's noise floor, the measured ratio 1.003×. The m-suite pins re-agree at 101.
- `examples/layout_probe.rs` drives the beats from the consumer side — the layout seam's first consumer example (the a11y sink's and the canvas's own precedent): one frame's story read back through the handle, the probe keeping its meaning, and a consumer's own broken engine refusing with its message landing in the ledger.

### M92 — the `#[derive(Widget)]` form (ADR-0096)

- Architecture §19 Q5's own option cashed: the required pair — the role, the name, the `Any` upcast — is the exact boilerplate the option existed to retire, and the derive grows into `alacer-macro` beside the `rsx!` compiler it already hosts. The expansion is the required pair and nothing else: `role()`, `label()`, and `as_any_mut()` in its documented `{ self }` shape — every other method rides the trait's own default, the derived widget exactly as honest as the hand-written one beside it.
- The attribute speaks the vocabulary the whole framework prints: `#[widget(role = "…")]` takes `Role::name`'s words (`button`, `container`, `text_input`, …), unknown words refused with the vocabulary named, and `live_region` refused with its reason — the pipeline builds the region itself (ADR-0024), a widget claiming the reserved role would fight the pipeline's own node.
- The name is required in one of two honest forms: a string riding `LabelSource::text`, or `#[widget(label = from_children)]` riding the container aggregation contract Column and Table declare themselves — silence is never the derive's default (an unnamed widget is the a11y walker's offender, §11.1). Duplicates, non-strings, enums, and generics are refused with the limitation named (the ADR-0025 diagnostic species).
- The facade re-exports the derive beside the trait it implements — the serde pattern, one name two namespaces: `use alacer::prelude::*` imports both, and `#[derive(Widget)]` resolves in the macro namespace while the trait resolves in the type namespace. Generated code references `::alacer_core` paths at the call site (the crate's own dioxus-rsx pattern), so the derive cannot drift from the trait it implements.
- The corpus grew 97 → 99 by the coordinated rule: `derive/snapshot_1k_derived` beside `derive/snapshot_1k_handwritten` — the same roles, the same names, the same 1,000-node trees through the real pipeline — the `derive_zero_overhead` ratio budget (≤ 1.10×) the host-independent contract class prices, the measured ratio 0.96×. The m-suite pins re-agree at 101.
- `examples/derive_widget.rs` drives the beats from the consumer side: the pair landing in the a11y snapshot, the two name forms, the defaults riding the trait, the `Any` hatch downcasting, and the AT's press meeting the honest refusal.

### M91 — the AT's horizontal vocabulary (ADR-0095)

- The action enum grew the pair beside the unit scrolls: `A11yAction::ScrollLeft(ScrollUnit)`/`A11yAction::ScrollRight(ScrollUnit)` — ADR-0093's second re-open rule cashed, the same unit contract the vertical pair travels by (an item is one line, a page is the viewport's *width*). Every variant gained its consuming widget in the same cut (the enum's own rule).
- The scroll area consumes the pair by the same clamp arithmetic on the declared width span: the record's species names the ask (`ScrollAction::ScrollLeft`/`ScrollRight`, the wire tags `scroll_left`/`scroll_right` flowing additively), the horizontal fact carries the story beside the vertical identity, and the unit rides the record exactly as the vertical pair's asks ride theirs.
- The boundary's mapping carries AccessKit's pair through (`Action::ScrollLeft`/`Action::ScrollRight`, the request's unit carried through) — the vocabulary the AT's reader always spoke, never dropped unmapped again.
- The value tier refuses the pair in the same cut: the slider, the progress bar, and the text field refuse the horizontal ask — a question outside the tier is not a tier fact (the ADR-0046 rule), the refusal arms growing with the enum.
- The vertical-only container refuses the horizontal ask honestly (`no_scrollable_content`), the vertical vocabulary beside it landing untouched — the pair grew beside the vertical vocabulary, never over it (the M89 contract re-pinned at this cut).
- The corpus grew 96 → 97 by the coordinated rule: `a11y_action/route_100_horizontal_actions_1k` — the horizontal pair routed through pass 10 on the same 1,000-node arena at the same worst-case target, the gate holding it in the mixed route's cost class. The m-suite pins re-agree at 97.
- `examples/at_horizontal.rs` drives the beats from the consumer side: the walk by its unit, the record's species, the vertical-only refusal, and the value tier's refusal.

### M90 — the handle grows the x half (ADR-0094)

- The app's own two-axis handle door: `ListHandle::scroll_to_x` beside `scroll_to` — ADR-0093's third re-open rule cashed, the x ask priced by the same clamp arithmetic every door answers by (the declared width span, the named walls, the honest refusals).
- The record is the same named-wall record: the x story rides its horizontal half (ADR-0056's axis rule) beside the vertical fields riding the identity answer — the axis unasked holds the offset it had, delivered, no ask beside it (the M89 content door's own convention). The stamps ride their honest absence: the app door knows no node id (the M43 rule at the x half).
- The visible motion composes at the mutate-guard `request` seam (the M23 shape, ADR-0093's call-site composition rule): `arena.mutate(node)` + `request(PAINT | A11Y)` — the seam every programmatic scroll surfaces through; the bare handle door's record half is the M50 suite's own shape.
- A vertical-only list answers `no_scrollable_content` against its undeclared span — the honest wall met at the handle's own door; a non-finite ask refuses before the arithmetic (`non_finite_ask`).
- The corpus grew 95 → 96 by the coordinated rule: `scroll/route_100_handle_scroll_x_1k` — the app's own x door driven the way apps drive it, one hundred wall-to-wall asks, the gate holding the handle door in the offset-priced class. The m-suite pins re-agree at 96.
- `examples/handle_scroll_x.rs` drives the beats from the consumer side: the landing composed at the seam, the record's horizontal half, the wall pinning a past run, and the vertical-only refusal.

### M89 — the two-axis door (ADR-0093)

- The bring-into-view door grew the axis beside the ask: `BringIntoViewAsk` carries both halves, `Widget::bring_into_view` answers each axis by its own declared span, and `ScrollLanding` reports the horizontal axis's own answer beside the vertical fields.
- The scroll record grew the axis beside the action (ADR-0056's axis rule): one additive `horizontal` fact — outcome, landed offset, ask, wall — beside the vertical story; `None` on the stories with no horizontal half. The wire's scroll event carries the same additive object.
- `List::two_axis(width, height)` declares the viewport's two axes; the handle's `set_content_width` and `offset_x` carry the horizontal story; the wheel's x run reaches the same clamp arithmetic (positive x scrolls right).
- The vertical-only container refuses the x half honestly: `no_scrollable_content` against the undeclared span, the ask the geometry priced riding beside the refusal.
- The corpus grew 94 → 95 by the coordinated rule: `scroll/route_100_horizontal_bring_into_view_1k` — the wide rows re-pin the x half on every landing, the gate holding the two-axis ask in the offset-priced class. The m-suite pins re-agree at 95.
- `examples/horizontal_scroll.rs` drives the beats from the consumer side: the straddling row landed on both axes, the wheel's x run, and the vertical-only refusal — every silence named.


### Added

- **M88 — the bring-into-view door: the scroll ask priced in nodes (ADR-0092)**:
  the complement the scroll tier's own door set always named — the
  wheel, the AT's unit scrolls and value steps, and the app's
  raw-offset handle all answer asks priced in *offsets*, while the ask
  consumers actually hold is priced in *nodes* ("this item, this row,
  this focused control"). `PassPipeline::scroll_into_view` reads the
  bounds ledger it already owns and prices the minimal offset that
  lands the aimed node fully inside the nearest scrollable ancestor's
  viewport — the content-space rect the bounds' own difference,
  invariant under every offset the bounds reflect (container and
  descendants ride the same compose translation); a rect taller than
  the viewport pins its top edge — and hands the ask to the
  container's own door (`Widget::bring_into_view`, the clamp
  arithmetic the wheel already rides: the span walls, the named
  refusals, never the pipeline guessing at a widget-private span).
  The answer is data at every landing: `Landed` beside the container
  that moved, `Visible` the honest no-op (no masks, no record, the
  idle frame the content clamp's own rule rides), the refusals named
  `unknown_node`/`not_laid_out`/`no_scrollable_ancestor`/
  `no_scroll_door` — typed answers alone, never tier records. A
  landed ask streams one `bring_into_view` record with the
  container's stamp beside the additive `aimed` field (the node the
  ask was for, the M30 rule at the scroll tier), and merges
  `PAINT | A11Y` on the container (the wheel's own currency, compose
  translating without a measure call). The corpus grows 93 → 94 by
  the coordinated rule (`scroll/route_100_bring_into_view_1k`), and
  [`examples/scroll_into_view.rs`](crates/alacer/examples/scroll_into_view.rs)
  the consumer demonstration.
- **M87 — the input-frame allocation budget: the tier the warm tick does not cover (ADR-0091)**:
  ADR-0088's own pre-registered re-open rule cashed — through M86 the
  ledger's budget had one tier (the warm tick pinned clean to the
  byte, the m84 suite) and an input-driven frame was priced by
  nothing — the instrument measured first: a pointer-move frame
  churns a small, balanced burst (6 blocks / ~752 bytes, allocated
  and freed inside the same tick), the dispatch queue's one-time
  block (~192 bytes) lands on the first tracked move frame and is
  held ever after (the capacity, never re-paid), the key and click
  frames balance beside it, and the warm idle tick answers clean in
  the same window. The budget the m87 suite pins is the honest shape —
  the input frame's line is **balanced**, net zero to the byte, the
  one-time block named (never a zero the facts refute): the m86-era
  folklore the instrument answered, the corpus growing 92 → 93 by the
  coordinated rule (`alloc/input_frame_budget`), and
  [`examples/input_budget.rs`](crates/alacer/examples/input_budget.rs)
  the consumer demonstration.
- **M86 — the shaped composition lock: the lock with shape (ADR-0090)**:
  ADR-0089's own pre-registered re-open rule cashed — through M85 a
  palette the OS's facts could not move followed nothing at all (the
  kiosk lock answers one palette for every fact permutation, an app's
  own composition follows every fact by how it is written) — so the
  composition tier grows the lock with shape:
  [`ShapedComposer::over`](crates/alacer-core/src/settings.rs) with
  [`FactShape`](crates/alacer-core/src/settings.rs), the second
  provided composer beside the kiosk one: the shape holding the facts
  it pins (const-built one pin at a time from `FOLLOW_ALL`, the honest
  absence pinned beside the color), the base composition (the app's
  own fact → palette mapping) answering the rest — the pinned fact
  substituted before the base resolves, the lock's flip the equality
  check's idle answer wherever the shape holds (one `SettingRecord`
  streams, the theme tier silent on the pinned fact), one
  `ThemeRecord` tagged `shaped` riding the followed fact's swap, no
  pipeline plumbing (the shape is a composition the door already
  resolves), the m86 suite pinning the story in pixels and records
  (the two-probe shape), the corpus growing 91 → 92 by the coordinated
  rule (`settings/shaped_flip_pinned_no_op`), and
  [`examples/shaped_lock.rs`](crates/alacer/examples/shaped_lock.rs)
  the consumer demonstration.
- **M85 — the kiosk lock: the composition that follows nothing (ADR-0089)**:
  ADR-0081's rule cashed a second time beside the
  provided-implementation rule (§13.3) — through M84 a palette the
  OS's facts could not move had two half-doors (the manual
  `set_theme` hold that loses on the next flip, the canon lock that
  locks only the canon palette) — so the composition tier grows the
  lock over any palette:
  [`KioskComposer::locked`](crates/alacer-core/src/settings.rs)
  (const, one `Palette` for every fact permutation, all sixteen
  bit-flips pinned by the settings tier's unit tests), the lock's
  flip the equality check's idle answer (the flip lands, one
  `SettingRecord` streams, the theme tier streams none — the composed
  answer *is* the palette already installed), the manual `set_theme`
  ask winning while it holds and the next flip winning it back (the
  `ThemeRecord` tagged `kiosk`, the reach beside it), the m85 suite
  pinning the story in pixels and records, the corpus growing
  90 → 91 by the coordinated rule (`settings/kiosk_flip_no_op`), and
  [`examples/kiosk.rs`](crates/alacer/examples/kiosk.rs) the consumer
  demonstration.
- **M84 — the allocation ledger: the warm-frame budget enforced (ADR-0088)**:
  architecture A7's consequence shipped as the instrument it always
  named — through M83 the workspace named dhat in its
  adopted-dependencies list and no crate adopted it, and the
  instrument found the truth: a static UI's warm tick churned 2
  blocks / 288 bytes (the tick's ledger buffer dying with every
  `FrameReport`). The hook is adopted
  ([`dhat::Alloc`](crates/alacer-testing/src/alloc.rs), A2 — steal
  proven cores; own only the composition) and the budget's
  vocabulary composed beside it
  ([`AllocLedger`](crates/alacer-testing/src/alloc.rs) the armed
  window, [`AllocStats`](crates/alacer-testing/src/alloc.rs) the
  snapshots, [`AllocDelta`](crates/alacer-testing/src/alloc.rs) the
  answers); the pass ledger's buffer rides back into the pipeline
  and the warm frame's report carries the canonical idle ledger by
  `Arc` handle
  ([`FrameReport::passes`](crates/alacer-core/src/passes.rs)) — the
  warm tick now answers **zero heap allocations**, to the byte,
  pinned by
  [`tests/m84_exit_criteria.rs`](crates/alacer-testing/tests/m84_exit_criteria.rs)
  and the corpus growing 86 → 90 by the coordinated rule
  (`alloc/warm_frame_zero_alloc` beside the ledger's own priced
  costs),
  [`examples/alloc_budget.rs`](crates/alacer/examples/alloc_budget.rs)
  the consumer demonstration.
- **M83 — the inspector at the assembly site: `attach_inspector` (ADR-0010)**:
  the recorder into the observer slot in one call —
  [`attach_inspector`](crates/alacer-inspect/src/attach.rs) binds the
  server before the pipeline installs it (a refused attach changes
  nothing), the reader handle answers the causal facts in-process,
  and [`examples/inspector_attach.rs`](crates/alacer/examples/inspector_attach.rs)
  the one-call demonstration (the URL printed, the accept loop
  serving on its own thread).
- **M82 — the canvas: the app's own paint closure (ADR-0008's identity rule)**:
  the paint seam grows the app-facing painter:
  [`Canvas`](crates/alacer-widgets/src/canvas.rs) mounts a hoisted
  painter closure, [`PaintCtx`](crates/alacer-core/src/scene.rs)
  handing the resolved facts beside the clip stack, the painter's
  primitives appending through the paint pass's own commit, and the
  flux bridge's `bind_value` queueing the canvas's `PAINT` mask —
  the repaint currency every widget shares, at the app's own
  surface; the identity rule (ADR-8) prices the hoisted painter as
  an idle frame (a re-render that keeps the painter closure
  re-records nothing), and
  [`examples/canvas.rs`](crates/alacer/examples/canvas.rs) drives the
  route from the consumer side.
- **M81 — the setting seam grows the accent fact, the composition follows it (ADR-0087)**:
  ADR-0085's own pre-registered re-open rule cashed a third time, grown
  by ADR-0086's own rule beside it — through M80 a widget that wanted
  the OS's accent preference guessed or polled, and a composition that
  wanted to tint by it reached for the platform's adapter, the reach
  ADR-0081 forbids — so the seam grows the fact beside the three:
  [`SettingSeam::accent_color`](crates/alacer-core/src/settings.rs)
  answers `Option<Color>` (required, one seam one door never a second
  seam per fact, the fact riding its presence — `None` the honest
  absence, never a guessed color), the door's flip check comparing all
  four facts (the accent-only flip the M79 check swallowed lands now),
  the fact resolving once per paint pass into
  [`PaintCtx::accent_color`](crates/alacer-core/src/scene.rs),
  [`SettingRecord`](crates/alacer-core/src/inspect.rs) growing the
  field beside the facts, the wire growing the additive `accent`
  field at version 1 (the documented `#RRGGBBAA` hex, `null` the
  absence, the embedded client rendering it), the composition
  resolving by it beside the three
  ([`SettingComposer::palette`](crates/alacer-core/src/settings.rs)
  growing the fact by ADR-0086's own shape,
  [`CanonComposer`](crates/alacer-core/src/settings.rs) grown in
  signature unchanged in behavior), the winit adapter answering the
  honest absence (ADR-0083's adapter rule), and
  [`examples/accent_follow.rs`](crates/alacer/examples/accent_follow.rs)
  the consumer demonstration (the accent-follow composition tinting
  its accent roles from the fact); the corpus grows 85 → 86 by the
  coordinated rule (`settings/door_flip_accent`).
- **M80 — the composition follows the facts (ADR-0086)**:
  ADR-0083's second pre-registered re-open rule, grown by ADR-0085's
  own cut — through M79 the composition resolved against the dark fact
  alone (a palette that wanted to calm or raise by the facts could not
  answer without reaching into the platform's adapter, the reach
  ADR-0081 forbids) — so the composer's signature grows the facts
  beside `dark`:
  [`SettingComposer::palette`](crates/alacer-core/src/settings.rs)
  answers `palette(&self, dark, reduced_motion, high_contrast)` (the
  pipeline handing the composition what it asks for, never the composer
  reaching into an adapter), every landed flip re-resolving the
  composition against the facts that landed (a motion-only flip
  composing the calm palette riding the flip's own broadcast — one
  flip, one count, the ADR-0084 rule unchanged; a flip whose facts
  compose the palette already installed still re-swapping nothing —
  the theme tier's idle rule guarding the swap half), the install
  resolving against the facts that are,
  [`CanonComposer`](crates/alacer-core/src/settings.rs) grown in
  signature unchanged in behavior (the composition that follows
  nothing), the wire carrying nothing new (the tag names the
  composition, not the palette — the M75 *duet* rule), and
  [`examples/follow_os.rs`](crates/alacer/examples/follow_os.rs)'s
  *Duet* composition growing its calm answer (the motion-only flip
  composing the *calm* duet dark — the palette that calms with the
  fact).
- **M79 — the setting seam grows the contrast fact (ADR-0085)**:
  ADR-0083's first pre-registered re-open rule cashed a second time —
  through M78 a widget that wanted the OS's high-contrast preference
  guessed or polled, the fact the vocabulary consults when contrast
  itself is the question (a hairline's weight, a focus ring's width)
  existing nowhere in the pipeline — so the fact rides the seam the
  dark and motion facts already have:
  [`SettingSeam::high_contrast`](crates/alacer-core/src/settings.rs)
  beside the two (required — one seam, one door, never a second seam
  per fact), [`CanonSettings`](crates/alacer-core/src/settings.rs)
  answering the canon absence (light, motion allowed, standard
  contrast), the platform's
  [`WinitSettings`](crates/alacer-platform/src/lib.rs) answering the
  honest absence where winit reports no read (a platform that reports
  no fact answers the absence, never a guess),
  [`PassPipeline::set_settings`](crates/alacer-core/src/passes.rs)'s
  equality check comparing all three facts (the M77 check would have
  swallowed a contrast-only flip — the fact moved, the door answered
  `false`, the vocabulary never saw it), a landed flip broadcasting
  `Change::PAINT` on the tree and resolving the fact once per paint
  pass into [`PaintCtx`](crates/alacer-core/src/scene.rs) beside the
  palette and the facts it sits beside (one dynamic call per pass,
  never per node), the record growing the fact beside the facts it
  already names (`SettingRecord { dark, reduced_motion, high_contrast,
  repainted }` — one record per flip), the wire growing additively at
  version 1 (`setting` — the fact beside the facts), the embedded
  client rendering the fact in the row's headline and detail line, the
  corpus growing 84 → 85 by the coordinated rule
  (`settings/door_flip_contrast` — the contrast-only flip landing
  where the other facts are still, the real reader naming the contrast
  fact), and
  [`examples/high_contrast.rs`](crates/alacer/examples/high_contrast.rs)
  driving the fact from the consumer side.
- **M78 — the swap names its repaint count (ADR-0084)**: ADR-0082's
  second pre-registered re-open rule reaches the theme tier — through
  M77 a swap's [`ThemeRecord`](crates/alacer-core/src/inspect.rs)
  named the tag alone while the flip's `SettingRecord` beside it
  carried its count (the asymmetry the two doors' rows show side by
  side on one log) — so the theme record grows the broadcast's node
  count beside the tag (`repainted` — the lengths rule at the theme
  tier: the count computed where the swap's broadcast resolves,
  `PassPipeline::set_theme`'s own pre-order walk, the ids never
  riding), the composed swap naming the same reach the flip's
  broadcast resolved (one flip, one count, however many palettes the
  composition resolved; the install path walking its own broadcast and
  naming that walk's length), the no-op swap streaming no record
  (there is no count to name, the idle-frame property unchanged), the
  wire growing additively at version 1 (`theme` — the tag beside its
  reach), the embedded client rendering the count in the row's detail
  line, the corpus growing 83 → 84 by the coordinated rule
  (`theme/door_swap_counted` — the counted swap read end to end, the
  real reader the lengths rule priced), and
  [`examples/theme_swap.rs`](crates/alacer/examples/theme_swap.rs)'s
  exit read-back names the reach beside the tag.
- **M77 — the setting seam grows the motion fact (ADR-0083)**:
  ADR-0080's first pre-registered re-open rule cashes — through M76 a
  widget that wanted the OS's reduced-motion fact guessed or polled,
  the fact the vocabulary consults when motion itself is the question
  (a spinner's spin, a thumbnail's autoplay) existing nowhere in the
  pipeline — so the fact rides the seam the dark fact already has:
  [`SettingSeam::reduced_motion`](crates/alacer-core/src/settings.rs)
  beside `dark_mode` (required — one seam, one door, never a second
  seam per fact), [`CanonSettings`](crates/alacer-core/src/settings.rs)
  answering the canon absence (light, motion allowed), the platform's
  [`WinitSettings`](crates/alacer-platform/src/lib.rs) answering the
  honest absence where winit reports no read (a platform that reports
  no fact answers the absence, never a guess),
  [`PassPipeline::set_settings`](crates/alacer-core/src/passes.rs)'s
  equality check comparing both facts (the M74 check would have
  swallowed a motion-only flip — the fact moved, the door answered
  `false`, the vocabulary never saw it), a landed flip broadcasting
  `Change::PAINT` on the tree and resolving the fact once per paint
  pass into [`PaintCtx`](crates/alacer-core/src/scene.rs) beside the
  palette and dark (one dynamic call per pass, never per node), the
  record growing the fact beside the facts it already names
  (`SettingRecord { dark, reduced_motion, repainted }` — one record
  per flip), the wire growing additively at version 1 (`setting` — the
  fact beside the facts), the embedded client rendering the fact in
  the row's headline and detail line, the corpus growing 82 → 83 by
  the coordinated rule (`settings/door_flip_motion` — the motion-only
  flip landing where the dark fact is still, the real reader naming
  the motion fact), and
  [`examples/reduced_motion.rs`](crates/alacer/examples/reduced_motion.rs)
  drives the fact from the consumer side (the canon build, the
  motion-only landing, the honest no-op, the fact restored beside a
  dark flip).
- **M76 — the flip names its repaint count (ADR-0082)**: ADR-0080's
  second pre-registered re-open rule cashes — a consumer asking "how
  much did the OS flip repaint?" had to re-derive it, the record
  naming the fact alone — so
  [`SettingRecord`](crates/alacer-core/src/inspect.rs) grows the
  broadcast's node count beside the fact (`repainted` — the lengths
  rule: the count computed where the broadcast resolves,
  `PassPipeline::set_settings`'s own pre-order walk, the walk's length
  *is* the flip's reach, never re-counted at the reader, the ids never
  riding — a count is not an identity), the count being the fact of the
  flip's own moment (a tree that grows later never rewrites a queued
  record), the no-op flip streaming no record (there is no count to
  name, the idle-frame property unchanged), the wire growing additively
  at version 1 (`setting` — the fact beside its reach), the embedded
  client rendering the count in the row's detail line, the composition
  door's swaps riding the grown record unchanged (one flip, one count,
  however many palettes the composition resolved), the corpus growing
  81 → 82 by the coordinated rule (`settings/door_flip_counted` — the
  counted flip read end to end, the real reader the lengths rule
  priced), and
  [`examples/dark_mode.rs`](crates/alacer/examples/dark_mode.rs)'s exit
  read-back names the reach beside the fact.
- **M75 — the composition door (ADR-0081)**: ADR-0080's pre-registered
  re-open rule cashes — through M74 an app that wanted palette-following
  subscribed to its own OS events and called `set_theme` beside every flip
  it watched — so the palette the OS's fact resolves into installs as one
  [`SettingComposer`](crates/alacer-core/src/settings.rs)
  (`PassPipeline::set_composer`), a pure function reading no platform
  state — the pipeline hands it the fact, it answers a palette and the
  tag the theme trail carries — the install resolving against the fact
  that *is* (the effective-palette equality, one
  [`ThemeRecord`](crates/alacer-core/src/inspect.rs) tagged with the
  composer's name, `Change::PAINT` on the tree), every landed flip
  re-resolving the composition on the flip's own broadcast (no second
  walk over the arena, no second currency), the composed swap streaming
  behind the flip's own `Setting` record (the M74 drain ordering — the
  fact first, the palette that fact painted into second), the manual
  `set_theme` door winning while the fact is still (the composition
  owning the palette only when the OS moves),
  [`CanonComposer`](crates/alacer-core/src/settings.rs) the provided
  composition that follows nothing (the canon palette for either fact,
  installable by name so a consumer that composed away can compose back
  through the same door it composed in), a flip that composes the palette
  already installed re-swapping nothing (the swap half the no-op, the
  idle rule at the theme tier), the corpus growing 79 → 81 by the
  coordinated rule (`settings/composer_install_no_op`,
  `settings/door_flip_composed` — bench, gate, and baseline in the same
  cut), and [`examples/follow_os.rs`](crates/alacer/examples/follow_os.rs)
  drives the door from the consumer side.
- **M74 — the OS setting seam (ADR-0080)**: ADR-0079's pre-registered
  re-open rule cashes — the OS's dark-mode fact existed nowhere in the
  pipeline, a widget that wanted it guessed or polled, and "follow the
  OS" meant rebuilding the palette by hand — so the fact installs as one
  [`SettingSeam`](crates/alacer-core/src/settings.rs)
  (`PassPipeline::set_settings`), the platform's
  [`WinitSettings`](crates/alacer-platform/src/lib.rs) adapter pushing
  the window's opening `theme()` and every `ThemeChanged` event through
  the door (the adapter the platform owns, ADR-0009 — a platform that
  reports no fact installs nothing), the flip broadcasting `Change::PAINT`
  on the tree and resolving the fact once per paint pass into
  [`PaintCtx`](crates/alacer-core/src/scene.rs) beside the palette — the
  fact never munging the palette: the roles the widgets borrowed
  re-resolve from the *installed palette* exactly as they were, an app
  that wants palette-following swapping through the theme door it already
  owns — and a flip that lands where the fact already is answers `false`
  and queues nothing (the idle-frame property, A1's rule at the setting
  tier; the canon light installed where nothing is installed *is* that
  no-op). The flip streams one
  [`SettingRecord`](crates/alacer-core/src/inspect.rs) — the bool *is*
  the fact — in
  [`InspectEvent::Setting`](crates/alacer-core/src/inspect.rs) at the
  tick's start drain (the theme trail's shape, one frame after the door
  answered; the thread trail bounded 256, the recorder's ring 512), the
  wire grows additively at version 1 (`setting` — the `dark` fact), the
  embedded client renders the setting row beside the theme's, the corpus
  grows 76 → 79 by the coordinated rule (`settings/note_disabled`,
  `settings/note_enabled_bounded`, `settings/door_no_op_flip` — bench,
  gate, and baseline in the same cut), and
  [`examples/dark_mode.rs`](crates/alacer/examples/dark_mode.rs) drives
  the seam from the consumer side.
- **M73 — the runtime theme seam (ADR-0079)**: the paint tier's own
  seam — the palette every widget borrowed for its un-overridden paint
  roles was ten compile-time constants on
  [`alacer_widgets::theme`](crates/alacer-widgets/src/lib.rs), "make it
  dark" meant rebuilding every widget, and no door answered the ask at
  all — so the palette installs as one
  [`ThemeSeam`](crates/alacer-core/src/theme.rs)
  (`PassPipeline::set_theme`), resolves once per paint pass into
  [`PaintCtx`](crates/alacer-core/src/scene.rs) (one dynamic call per
  frame, never per node), the swap broadcasting `Change::PAINT` on the
  tree — the mask's own documented semantics, never a private repaint
  channel — and a swap that lands where the palette already is answers
  `false` and queues nothing (the idle-frame property, A1's rule at the
  theme tier). The swap streams one
  [`ThemeRecord`](crates/alacer-core/src/inspect.rs) — the palette's
  tag beside the door, never its colors (the payload rule's shape) — in
  [`InspectEvent::Theme`](crates/alacer-core/src/inspect.rs) at the
  tick's start drain (the route-drop trail's shape, one frame after the
  door answered; the thread trail bounded 256, the recorder's ring 512),
  the wire growing additively at version 1 (the `theme` event, the
  client rendering the palette's row), and the canon palette pinning the
  constants the blessed snapshot corpus was blessed with byte for byte
  (`CanonPalette`, the provided implementation beside the trait). The
  corpus grows 73 → 76 (`theme/note_disabled`, `theme/note_enabled_bounded`,
  `theme/door_no_op_swap` — bench, gate, and baseline in the same cut).

### Fixed

- **The view reconciler counts a binding re-hang as a visual patch
  no more** ([`alacer-view/src/mount.rs`](crates/alacer-view/src/mount.rs)):
  `visual_props_changed` excluded handler churn but still counted the
  `binding:`/`binding:`-value re-hang the patcher itself calls a no-op
  ("re-hanging the same binding is a no-op, never a visual change",
  ADR-0038) — so every identical rebuild carrying a binding streamed a
  `ViewPatch` fact and queued `LAYOUT | PAINT`, and the m32 canon's own
  idle-frame rule ("the no-op re-render streams none", the M72 gate)
  failed against the M72 record. The re-hang is the patcher's own
  no-op; the value a binding carries rides the flux bridge's masks.
  The m4 suite's patched-generation count re-agrees (2 → 1).
- **The performance dashboard re-publishes the M71/M72 baselines**
  ([`benchmarks/dashboard/index.html`](benchmarks/dashboard/index.html)):
  the dashboard's last render predates the frame-refusal and view-patch
  baselines — the m5 suite's dashboard test failed on the stale render
  (`scripts/perf_dashboard.py` re-run, the 76-baseline corpus
  published).


### Added

- **M72 — the view's patch joins the causal log (ADR-0078)**:
  the view tier's own kill rule cashed — a consumer cannot answer
  "what did my re-render change — and how much?" from the log the
  causal trail streams: the view reconciliation, the tier every
  re-render rides, answered a
  [`PatchReport`](crates/alacer-view/src/mount.rs) the imperative
  caller read and dropped (the mount's build streamed nothing at
  all), no record on the log — so the view grows its own named fact
  ([`ViewPatchRecord`](crates/alacer-core/src/inspect.rs) — the
  reconciler's four counts beside each other, named by its lengths,
  the props never riding — in
  [`InspectEvent::ViewPatch`](crates/alacer-core/src/inspect.rs) at
  the tick's start drain, the route-drop trail's shape, one frame
  after the re-render that asked for it; the thread trail bounded
  256, the recorder's ring 512), the note sites the exact places the
  patch resolves (the reconciler's mount for the build — the tree
  named by its element count — and the update for the diff, the
  counts verbatim, gated on the diff moving something — the
  idle-frame property), one fact per patch that moved something,
  never per frame (the frames that re-render nothing stream none,
  and the no-op re-render streams none too — the m32 canon's own
  pre-registered rule, the flood rule at the view tier shipped at
  this cut), the record beside the door's
  own `PatchReport` (the door's contract unchanged), the wire's flat
  `view_patch` event at version 1, the embedded client rendering the
  counts' row, and
  [`examples/patch_trail.rs`](crates/alacer/examples/patch_trail.rs)
  walking the beat; the corpus grows 71 → 73 by the coordinated rule
  (`view_patch/note_disabled`, `view_patch/note_enabled_bounded` —
  bench, gate, and baseline in the same cut — and the M71 cut's
  promised `frame_refusal` baselines publishing in the same run)
- **M71 — the frame's refusals join the causal log (ADR-0077)**:
  the frame tier's own kill rule cashed — a consumer cannot answer
  "why did nothing paint?" from the log the causal trail streams: the
  frame, the tier every other tier's work rides, answered a
  `tracing::error!` line and a skipped frame when it went unpainted —
  so the frame grows its own named fact
  ([`FrameRefusalRecord`](crates/alacer-core/src/inspect.rs) — the
  species beside the failing node when one is the fact — in
  [`InspectEvent::FrameRefusal`](crates/alacer-core/src/inspect.rs) at
  the tick's start drain, the route-drop trail's shape, one frame
  after the frame that refused; the thread trail bounded 256, the
  recorder's ring 512), the note sites the exact places the refusals
  resolve — the tick's layout block for the engine seam's own
  [`LayoutError`](crates/alacer-core/src/passes.rs) (the `?`'s
  propagation spelled out, the failing node riding verbatim,
  ADR-0023's vocabulary) and the present ladder's stages in
  `alacer-platform` (`render_failed` / `partial_repaint_failed` /
  `framebuffer_lost` / `present_failed` — the softbuffer ladder's one
  silent arm, the full re-rasterization whose `None` kept a stale
  framebuffer without logging anything, named at the same cut), the
  species the ladder's own stages (`layout_refused`/`render_failed`/
  `partial_repaint_failed`/`framebuffer_lost`/`present_failed`), one
  fact per refusal (the frames that painted stream no refusal fact —
  the flood rule at the frame tier), the record riding *beside* the
  refusal's own door (the tick still answers `Err(LayoutError)`, the
  ladder still skips the frame — never instead of it; the engines'
  prose staying on the error types and the trail's trace lines, the
  payload rule's shape), the wire carrying the additive
  `frame_refusal` event at version 1 (`species`/`node`,
  `FrameRefusalSpecies::name` the single source), the embedded client
  rendering the refused stage's row on one log,
  `examples/frame_refusals.rs` walking the beat from the consumer side
  (a refusing layout engine behind the real pipeline, the `Err` the
  door still answers, the record beside it), the corpus growing 69 →
  71 (`frame_refusal/note_disabled` — the A7 contract's one disabled
  branch per note; `frame_refusal/note_enabled_bounded` — the enabled
  steady state's bounded push, bench, gate, and baseline in the same
  cut), and the living documents re-agree (the m71 suite pins the
  engine-seam refusal through the real pipeline, the recovered frames'
  honest absence, the stable tags, and the documents)
- **M70 — the activation names its rider (ADR-0076)**:
  ADR-0075's own kill rule cashed — a consumer cannot tell the AT's
  press from the keyboard's Space/Enter activation on the log the
  action trail streams (the record answered "did the press land — or
  why not", never "who pressed") — so the record grows the rider
  beside the ask
  ([`ActionRider`](crates/alacer-core/src/inspect.rs), the M30
  attribution rule at the activation tier: `ActionRoute` — the AT's
  ask the dispatch funnel routed, the funnel's own note site;
  `Keyboard` — the focused control's Space/Enter, the M10 parity the
  widgets answer like a click, the button's GTK release rule and the
  checkbox's toggle-on-press, named where the route is known), the
  keyboard's ride carrying the key route's stamp (M45, ADR-0051 — the
  focused control's AccessKit-stable id, the guard the key visit
  holds, never guessed at the note site), one fact per activation (the
  pressed phase arming nothing on the log, the release where the
  button's door answered — the flood rule at the activation tier), the
  wire carrying the additive `rider` field at version 1
  (`action_route`/`keyboard`, `ActionRider::name` the single source),
  the embedded client rendering "the AT's press landed" and "the
  keyboard's press landed" on one log, `examples/action_trail.rs`
  walking the rider beat beside the activation beats, no new corpus
  case (the keyboard's activation rides the published routes — the
  coordinated rule's no-growth arm, the corpus at 69), the living
  documents re-agree (the m70 suite pins both riders through the real
  routes, the pressed phase's absence, the checkbox's toggle, and the
  documents)
- **M69 — the activation ask answers on the log (ADR-0075)**:
  the action tier's own kill rule cashed — a consumer prices the AT's
  press (the default accessibility action the platform's action route
  aims at a control: the ask arrived, the target's door answered, and
  the answer was a bare `bool` whose `false` collapsed the two facts a
  trail reader wants apart — the availability tier's wall and the
  doorless target's silence; "did the AT's press land — or why not"
  unanswerable from the log alone) — so the action tier grows its own
  trail ([`ActionRecord`](crates/alacer-core/src/inspect.rs) — `ask`,
  `target`, `outcome` — in
  [`InspectEvent::Action`](crates/alacer-core/src/inspect.rs) at the
  tick's start drain, the preedit drain's shape, one frame after the
  ask; the thread trail bounded 256, the recorder's ring 512), the
  note site the exact place the ask resolves — the dispatch funnel
  (`PassPipeline::dispatch_a11y_action`, the route-drop trail's shape,
  ADR-0063) — the species computed where both facts live (the route
  holds the ask, the aimed id, the door's answer, and the tree's one
  `enabled` truth, M22/ADR-0028:
  [`ActionRefusal::Unavailable`](crates/alacer-core/src/inspect.rs)
  when the wall held the control,
  [`ActionRefusal::NoDoor`](crates/alacer-core/src/inspect.rs) when
  nothing did — the collapsed `false` split, never guessed at a
  widget-private call site), one fact per trail (the value tier's
  writes, the scroll tier's asks, and the text-write tier's rewrites
  stream no action fact — the M39 rule at the activation tier), the
  mask-queueing `bool` contract of record unchanged (the trail rides
  beside it, never instead of it), the wire carrying the flat `action`
  event at version 1, the embedded client rendering the row the other
  trails render, the corpus growing by the coordinated rule
  (`a11y_action/route_100_presses_1k` — the worst-case walk, the
  door's refusal, the trail's one disabled branch under the A7
  contract — bench, gate, and baseline in the same cut, the corpus
  pins moving 68 → 69), `examples/action_trail.rs` walking the
  activation beat, the living documents re-agree (the m69 suite pins
  the landing, the wall, the doorless silence, the tiered asks'
  absence, and the documents)

- **M68 — the payloadless chord names its read (ADR-0074)**:
  ADR-0068's second kill rule cashed — a consumer prices the
  payloadless chord (the copy or cut whose selection is empty: the
  chord arrived, the field's door answered, nothing was there to
  move, and the key-edit trail answered nothing — "why did my copy
  do nothing" unanswerable from the field's tier alone) — so the
  wall grows the read's species beside the door's
  ([`KeyEditRefusal::EmptySelection`](crates/alacer-core/src/inspect.rs),
  the ask named by the door's own match — the action vocabulary
  growing `KeyEditAction::Copy`, a delivered copy answering no fact,
  the m60 rule standing), one fact per trail (the round trip keeping
  its `NoSelection` fact, the field's refusal beside it — the
  delivered chord's two-fact shape mirrored at the wall), the paste
  keeping the honest absence (its read is the clipboard seam's, the
  m60 pin standing), the wire carrying the new `action` value
  (`copy`) and `refusal` species (`empty_selection`) additively at
  version 1, the embedded client rendering the refusal row it
  already renders, `examples/paste_edits.rs` walking the read beat,
  no new corpus case (the refusal rides the editing route the
  key-edit bench already prices — the M62 precedent, the corpus at
  68), the living documents re-agree (the m68 suite pins the
  refusal through the real chord route, the absence on the delivered
  copy and the refused paste, and the documents)
- **M67 — the adapter's full-tree echo joins the causal log (ADR-0073)**:
  M59's pre-registered kill rule cashed — a consumer prices the
  adapter activation's timing and the a11y sink's full-tree fallbacks
  (the AT connected or reconnected; the app answered with the whole
  resolved tree — "why did the app just push the whole tree, and how
  big was it" unanswerable from the log alone) — so the a11y tier
  grows its own trail
  ([`A11yEchoRecord`](crates/alacer-core/src/inspect.rs), the tree
  named by its node count — the lengths rule at the a11y tier, the
  payload never riding; `0` never riding, the honest absence), the
  note site the sink's own full-tree build
  ([`AccessKitSink::full_tree_update`](crates/alacer-a11y/src/lib.rs)
  — the same builder the deltas use, the build's purity unchanged,
  the note the trail's one branch while nobody listens), the trail
  the route-drop trail's shape (bounded 256, the tick's start drain
  ahead of the frame's routes — a connect predates the asks), the
  wire carrying the flat `a11y_echo` event at version 1, the
  embedded client rendering "the AT answered with the full tree
  (N nodes)", the protocol document's section carrying the field and
  the bounds list re-agreeing (the M56–M66 rings it had lagged ride
  the same line), `examples/a11y_echo.rs` walking the echo beat
  beside the reconnect, no new corpus case (the echo rides no
  measured route — the coordinated rule's no-growth arm, the corpus
  at 68), the living documents re-agree (the m67 suite pins the echo
  through the real sink build, the absence on the unbuilt tree, and
  the documents)
- **M66 — the journal's echo names its field (ADR-0072)**:
  ADR-0037's second kill rule cashed — a consumer needs per-field
  attribution (two inputs, two journals, one log: the depths streamed
  but the field unnamed left "whose journal moved" unanswerable from
  the rows alone) — so the record grows the M43 stamp beside the
  depths ([`EditRecord::target`](crates/alacer-core/src/inspect.rs),
  the AccessKit-stable id the routed event arrived under, stamped
  where the echo resolves — the same guard the key route has held
  since M45 and the AT action route since M43, the identity never
  guessed at the note site), the app's direct rides carrying the
  honest absence (the stamp fills only what the route knows, a
  dropped guard leaving no stale identity), the wire carrying the
  `target` field additively at version 1 (the route-drop `rider`
  field's rule), the embedded client rendering "— the field N" beside
  the depths, `examples/edit_trail.rs` walking the two stamps (one
  committed composition routed through the real IME route into a live
  `TextInput`, its echo carrying the field's id beside the direct
  rides' honest absence), no new corpus case (the field rides the
  routes the journal already rides — the coordinated rule's no-growth
  arm, the corpus at 68), the living documents re-agree (the m66
  suite pins the stamp through the real key and action routes, the
  absence on the direct rides, and the documents)
- **M65 — the route drop names its rider and splits its silence (ADR-0071)**: two pre-registered prices cashed — ADR-0063's third
  kill rule (the drops need their rider: the platform boundary's
  mapping arms, the dispatch funnel's target resolution, and the IME
  route's silence all resolve drops, and one collapsed record left a
  reader asking "which route refused my ask" unanswered) and
  ADR-0065's first kill rule (the dead focused node's drop priced the
  same as the never-focused one) — so the record grows
  [`RouteDropRider`](crates/alacer-core/src/inspect.rs) beside the aim
  (`boundary`/`funnel`/`ime_route`, the M30 attribution rule at the
  route tier, the attribution declared where the drop resolves, never
  guessed from the species) and the silence splits
  (`no_focused_field` keeping the never-focused half, the focused
  node's death riding its own `dead_focus` — the composition naming
  *why* it had nowhere), the wire carrying the rider field and the new
  species value additively at version 1 (the clipboard `engine`
  field's rule), the embedded client rendering "the funnel refused it"
  beside the species, `examples/route_drops.rs` walking the rider
  beats (the funnel's drop, the boundary's species, the never-focused
  commit, the composition whose field died mid-session), no new
  corpus case (the field rides the routes the drops already ride —
  the M57 precedent, the corpus at 68), the living documents
  re-agree (the m65 suite pins the three riders through the real
  dispatch, the split on the real dead focus, and the documents)
- **M64 — the observer's tee joins the seam (ADR-0070)**: the
  second-door ask the single slot pre-registered cashed — the
  pipeline's observer slot is one (`PassPipeline::set_observer`), and
  two devtools at once (the inspector's recorder on the wire beside
  the trail's provided reader in-process, M63's) needed either a
  hand-rolled wrapper (the benches' own `Share` shape, again) or one
  door silently losing the other — so the seam grows
  [`Tee`](crates/alacer-core/src/inspect.rs) beside `InspectObserver`:
  two boxed observers behind one, each event forwarded first to
  `first` then to `second` (the arrival order the causal reads rely
  on), a third reader nesting (`Tee::new(a, Tee::new(b, c))`), the
  fan-out priced not assumed (`trail/tee_announce_100_1k` under the
  `trail_tee_overhead` budget ≤ 2.5× the unobserved announce route and
  its own same-host regression gate, the corpus at 68, the m5/m6 pins
  re-agreeing beside M63's), `examples/inspector_demo.rs` wiring the
  recorder and the reader behind one tee (the recorder keeps the wire,
  the app holds the in-process reading handle), no wire change (the
  slot's single `None` branch unchanged — the A7 idle contract), the
  living documents re-agree (the m64 suite prices the doors' order,
  both doors fed, the recorder+reader share, and the nested tee)
- **M63 — the trail's provided reader joins the seam (ADR-0069)**: the
  collector ask the m-suite pre-registered cashed — every gate file and
  every trail example hand-rolled the same shape (the
  `Rc<RefCell<VecDeque>>` behind a one-method observer, 33 files over;
  the benches' `Share` wrapper beside it) — so the seam grows
  [`TrailLog`](crates/alacer-core/src/trail.rs) beside
  `InspectObserver` (the clipboard seam's provided `MemoryClipboard`
  precedent, clonable, `Arc<Mutex>`-backed so one installation serves
  many readers), the drain the reader's own window (the held events
  handed back oldest-first, the window emptied behind them), the look
  reading without disturbing, the tip named, the eviction counted
  (`dropped` — the named-refusal rule at the reader's own tier, never
  a silent loss), the window bounded (`TRAIL_RING_CAP` = 1,024,
  `with_cap` pricing a reader's own, a `0` cap refused with the
  house's honest panic), the harness growing the one-call door
  (`TestHarness::trail` / `trail_with_cap` — one line where every
  suite wrote eleven), the corpus growing by the coordinated rule
  (`trail/reader_announce_100_1k` + `trail_ring/fill_drain_1024` under
  the `trail_reader_overhead` budget ≤ 2× the unobserved announce
  route, the pins re-agreeing to 68), `examples/trail_reader.rs`
  walking the beats, no wire change (the reader holds the events the
  routes already stream), the living documents re-agree (the m63 suite
  prices the typing route, the announce route, the ring's arithmetic,
  the drain's window, and the undrained trail's bound)
- **M62 — the typing door's refusals join the causal log
  (ADR-0068)**: ADR-0064's second kill rule cashed — a consumer
  prices the closed field's wall on the log (the M58 record named
  every edit that landed; the wall was pure silence, a typed run
  beside a wall indistinguishable from an honest gap) — so the record
  grows its outcome beside the landed shape
  (`KeyEditOutcome::Landed` on every delivered edit — the M58 shape
  named additively; `Refused(KeyEditRefusal::Unavailable)` on the
  wall — the availability tier's species in the value tier's own
  word, ADR-0027's `unavailable` tag, one refusal vocabulary across
  the tiers), the ask named by the door's own match (`refused_action`
  — the classify half of `edit_key`'s match: a character, a unit or
  word delete, and the chords name themselves, while the caret's
  walk, the submit's Enter, and the copy and journal chords ride
  none — the wall prices edit asks, never every routed keystroke; a
  release names no ask), the chords' refusals riding behind their own
  actions (`paste`/`cut` under the `clipboard` driver — ADR-0066's
  third kill rule cashing beside; the engine never ran, one fact per
  trail), the shape priced at nothing and the selection the untouched
  one (never a munged delivered edit; an open door's no-op staying
  silent — the M20 rule's own word), the window the availability
  tier's own (the M22 eviction is the frame's pass, not the door —
  the wall answers the frame it is held down in; the boundary's
  post-eviction silence the route's own vocabulary — the key route's
  fallbacks, the IME route's `no_focused_field` drop, M59 — never a
  munged typing-door fact; the M58 pin re-agrees with the boundary
  named), the supersede staying the delivered run's fold (a refusal
  at the trail's tip supersedes nothing — the outcome guard, the
  refusal never riding the fold), the wire carrying the additive
  `outcome`/`refusal` fields at version 1 (no new kind), the client
  rendering "the keyboard refused — the wall held (unavailable)",
  `examples/edit_refusals.rs` walking the beats, no new corpus case
  and no new bench case (the record rides the route the keys already
  take and the editing bench already prices — the wall off the
  delivered hot path; the corpus stands at 65), the living documents
  re-agree (the M58 suite's disabled-field pin re-agrees to the
  priced window)
- **M60 — the clipboard chords' field changes join the key-edit
  vocabulary (ADR-0066)**: ADR-0064's third kill rule cashed — a
  consumer prices the chords' edits at the field on the log (the
  paste's shape, the cut's span were answerable only by re-pricing
  the round trip's counts against the journal's) — so the record's
  action vocabulary grows the chords: `KeyEditAction::Paste`/`Cut`
  and `KeyEditDriver::Clipboard` (never munged onto the insert and
  delete they mirror — a cut that streamed a delete fact and a cut
  fact would double the edit, the round trip's own rule), the note
  sites standing where the chord resolves (the round trip's fact
  first, the causal order clipboard → edit), one fact per trail (the
  round trip keeps the clipboard trail's, the journal the depths),
  the refused paste and the copy answering no fact (the honest
  absence), the chords never riding the flood rule (one fact per
  chord, the supersede staying the typed run's), the wire carrying
  the new `action`/`driver` values at version 1 (no new kind), the
  client naming the chord's door, `examples/paste_edits.rs` walking
  the beats, no new corpus case (the note rides the route the
  clipboard bench already prices, the corpus stands at 65), the
  living documents re-agree (the M58 suite's clipboard-chord pin
  re-agrees to the priced chord)
- **M61 — the field's composing state joins the causal log
  (ADR-0067)**: ADR-0064's first kill rule cashed — a consumer
  prices the preedit's own trail, the composition's inline shape (a
  committed composition streamed its `KeyEditRecord`, the composing
  state itself was traced and gone) — so the IME tier grows its own
  record beside the commit's: one `PreeditRecord` per composing fact
  (the phase `composing`/`cleared`, the shape named by its UTF-16
  length — never the payload, ADR-0011's rule at the composition
  tier — the preedit-local cursor beside it, the funnel's stamp
  riding the IME route's guard), the flood rule the session's own
  supersede (one live session = one growing record, never a
  default-on echo of every composition frame), the preedit landing
  nowhere (the commit's key-edit fact riding beside it — one fact
  per trail; the spurious clear answering no fact), the preedit
  trail the key-edit trail's shape (`PREEDIT_TRAIL_CAP` = 256, the
  tick's start drain after the key-edit drain), the wire carrying
  the flat `preedit` event at version 1, the client rendering "the
  composition shows (shape 3) — the cursor at 2",
  `examples/preedit_trail.rs` walking the beats, no new corpus case
  (the note rides the route the IME events already take, the corpus
  stands at 65), the living documents re-agree
- **M59 — the IME's boundary drops join the route-drop vocabulary
  (ADR-0065)**: ADR-0063's first kill rule cashed — a consumer needs
  the platform's other silences on the log (an IME event no live
  field held dropped with only a consumed `Ignored` verdict — "my
  IME committed; where did the text go" was unanswerable from the
  log alone) — so the route's own vocabulary grows the species:
  `RouteDropSpecies::NoFocusedField` (`no_focused_field`) — the
  event named by its stable tag (`ime_preedit`/`ime_commit`), the
  aimed id riding its honest absence (the boundary had no aim to
  name — the drop *is* the missing field), a delivered commit never
  riding a drop record (the M57 rule's honest absence), no new trail
  and no new wire kind (the record rides the M57 route-drop trail,
  the `route_drop` event's species field carrying the new value at
  version 1, the client's prose rendering the echo unchanged),
  `examples/route_drops.rs` walking the fourth beat, no new corpus
  case (the corpus stands at 65), the living documents re-agree
- **M58 — the typing door's edits join the causal log (ADR-0064)**:
  ADR-0062's first kill rule cashed — a consumer needs the
  keyboard's and IME's edits on the log (a trail reader cannot
  answer "what did the typist change in this field" from the log
  alone: the journal streamed its depths, the AT's and app's doors
  their own `TextWriteRecord`s, and the typing door nothing) — so
  the edit stream grows its own per-change record beside the
  journal's: one `KeyEditRecord` per typing-door edit — the action
  named by the field's own editing vocabulary (`insert`,
  `backspace`, `delete`, `word_delete_back`, `word_delete_forward` —
  the word chords never munged onto the unit deletes they mirror),
  the driver named (`keyboard`/`ime`, the M30 rule at the typing
  tier), the change's shape named by lengths (`replaced`/`inserted`,
  UTF-16 units — never the payload: a field's value can carry a
  secret, ADR-0011), the landing selection beside them, and the
  journal's own fold riding the record — the flood rule the
  journal's fold: `EditHistory::record` answers whether it
  coalesced, the trail superseding a coalesced run's record in
  place (N keystrokes in one pure-append run land as one record,
  the drivers' records never merging), the clipboard chords riding
  the clipboard trail (one fact per trail), a no-op door answering
  no fact, the trail the text-write trail's shape (bounded 256, the
  tick's start drain after the route-drop drain), the IME route
  holding the funnel's stamp (the key route's own discipline since
  M45), the wire carrying the additive `key_edit` event at version
  1, the embedded client rendering "the keyboard insert (0 replaced
  by 3) — the selection at 3..3 — the journal folded the run",
  `examples/key_edits.rs` the consumer demonstration, one new corpus
  case (`editing/route_100_key_edits_1k` — the corpus stands at 65,
  the regression gate and the baseline published, the m-suite pins
  re-agreed), the living documents re-agree
- **M57 — the boundary's drops join the causal log (ADR-0063)**:
  ADR-0061's third kill rule cashed — a consumer needs the
  platform's defensive drops on the log (a trail reader cannot
  answer "the AT asked; why did no door ever answer" from the log
  alone) — so the boundary route grows its own refusal vocabulary:
  one `RouteDropRecord` per drop — the dropped action named by its
  stable `snake_case` tag (`A11yAction::name` in core, the
  boundary's `action_tag` beside it), the species naming why the
  boundary refused (`unknown_action`, `no_payload`,
  `off_target_selection`, `unknown_target`), and the aimed id
  riding verbatim — never re-resolved against the tree (a
  re-resolved aim would be the funnel's answer, not the AT's ask),
  and never a munged widget-door fact (no door ever answered —
  ADR-0016's faithful-mapping rule and ADR-0017's single-node
  selection contract are the refusals the species name). The four
  drop sites join the trail: `map_action_request`'s three mapping
  arms in `alacer-platform` and the dispatch funnel's own
  unknown-target refusal in `PassPipeline::dispatch_a11y_action`
  (the consumed `false` verdict the m10 contract pins, byte for
  byte, beside the fact that now explains it). The trail is the
  text-write trail's shape (thread-local, bounded at 256,
  oldest-first eviction, the A7 idle contract — one branch per
  drop, the drops the cold path outside the frame's hot loops),
  the facts draining at the tick's start after the text-write
  drain into `InspectEvent::RouteDrop`. The wire's `route_drop`
  event carries the record flat at version 1 (clients that ignore
  unknown kinds unaffected), the embedded client rendering
  "dropped: unknown_target (aimed at …)" beside the other trails,
  and `examples/route_drops.rs` is the consumer demonstration (the
  funnel's drop through the real dispatch route, the delivered ask
  beside it streaming nothing — the honest absence). No new
  corpus case — the record rides no measured route and runs
  outside the frame's hot paths; the corpus stands at 64, the
  living documents re-agree at the cut (M57, sixty-four decision
  records).
- **M56 — the app's text write joins the causal log (ADR-0062)**:
  ADR-0061's second kill rule cashed — a consumer needs the app's
  own writes attributed (a trail reader cannot answer "did the
  screen reader rewrite this field, or did the app" from the log
  alone — the M30 rule's own question, at the tier whose door the
  rule held off) — so `TextWriteRecord` grows the additive
  `source: TextWriteSource` (`AtDoor` / `AppDoor`, the M30
  attribution rule at the text tier): the AT door's four M55 note
  sites name `at_door`, and the app's imperative `set_value` — the
  door the mutate seam calls and the view reconciliation's patches
  ride — streams its own fact beside the journal's reset echo (the
  depths the journal's story, the write the door's). One write,
  one fact: the AT's `SetValue` rides a raw `apply_value` beneath
  its own note, so the app door's record never double-streams an
  AT ask, and the app door — riding no funnel route — carries the
  stamp's honest absence (the trail fills only what the caller
  cannot know: an identity, never a door, M43's rule). The shape
  and the privacy rule ride unchanged: the write named by lengths
  (`replaced` — the value the field carried, `inserted` — the
  value the app handed over, UTF-16 units) and never the payload
  (a field's value can carry a secret — the clipboard trail's
  rule, ADR-0045), the caret the write left beside them, the
  journal's reset still streaming (ADR-0021's rule — the AT's
  write is an app write, undo never fights it). The wire's
  `text_write` event carries the additive `source` field at
  version 1 (clients that ignore unknown fields unaffected), the
  embedded client rendering "the app's text write landed (9
  replaced by 18) — the selection at 18..18" beside the AT door's
  rows, and `examples/app_text_writes.rs` is the consumer
  demonstration (the mutate seam's write, the AT's ask, the doors
  tellable apart on one log). No new corpus case — the record
  rides the write the app already makes; the corpus stands at 64,
  the living documents re-agree at the cut (M56, sixty-three
  decision records).
- **M55 — the AT's text write joins the causal log (ADR-0061)**:
  ADR-0046's second kill rule cashed — a consumer needs the input
  tier's rides priced (a trail reader cannot answer "did the screen
  reader rewrite this field — what shape, where did the caret land,
  or why not" from the log alone) — so the input's AT door streams
  one `TextWriteRecord` per ask: the action beside the outcome
  (`set_value`/`replace_selection`/`set_selection`, `delivered`/
  `refused`), the write's shape named by lengths (`replaced`/
  `inserted`, UTF-16 units — the whole value for a `set_value`, the
  selection's span for a `replace_selection`) and never the payload
  (a field's value can carry a secret — the clipboard trail's rule,
  ADR-0045), the landing selection beside them, the ask beside the
  landing on the selection door (the M44 rule at the text tier — the
  forgiving floor and clamp visible as data), and the closed door's
  refusal streaming too (`unavailable` — the same fact the value
  tier names, ADR-0028's tier). The record's `target` fills from the
  routing funnel's own guard (M43, ADR-0049 — the same stamp the
  value tier's writes and the keyboard's steps stamp from), and the
  facts ride the tick's start drain into `InspectEvent::TextWrite`
  (the scroll drain's shape — dev-mode, bounded, silent while nobody
  listens). The journal's own echoes are untouched: a `set_value`
  still resets the journal (ADR-0021's rule — the AT's write is an
  app write, undo never fights it) and the reset still streams. The
  wire's `text_write` event carries the record flat at version 1
  (clients that ignore unknown kinds unaffected), the embedded client
  rendering "the text write landed (11 replaced by 21) — the
  selection at 21..21" beside the landings, and
  `examples/at_text_writes.rs` is the consumer demonstration. No new
  corpus case — the record rides the route the write already rides;
  the corpus stands at 64, the living documents re-agree at the cut
  (M55, sixty-two decision records).
- **M54 — the value step's page joins the scroll record (ADR-0060)**:
  ADR-0058's first kill rule cashed — a consumer needs the step's
  unit named — the value step's page is a unit-like distance a
  reader may want attributed the same way — so the AT action door
  fills the unit beside the step species too: `Some(ScrollUnit::Page)`
  beside the `Increment`/`Decrement` actions (the step steps a page,
  always, by the action vocabulary's own contract, M10/ADR-0016), the
  same field the unit scrolls answer by — no new field, no version
  bump, the field's contract grown beside its shape (the additive
  rule ADR-0010 names; clients that render the M52 null as absence
  render "by one page" unchanged). The honest absence keeps its
  other stories: the wheel's coalesced run (a run is not an ask —
  M53 priced it its own magnitude beside the unit's seat), the app's
  own door, and the content's own clamp (ADR-0057). The embedded
  client renders the step's page beside the landings, and
  `examples/step_pages.rs` is the consumer demonstration. No new
  corpus case — the field rides the route the step already rides;
  the corpus stands at 64, the living documents re-agree at the cut
  (M54, sixty-one decision records).
- **M53 — the wheel's line count joins the scroll record (ADR-0059)**:
  ADR-0058's second kill rule cashed — a consumer needs the wheel's
  line count named — the coalesced run's notional lines — so
  `ScrollRecord` grows the additive `lines: Option<f32>` beside the
  `Wheel` action, the run priced by the platform's own convention
  (one line = 40 px, §10.2 — the rule the platform's wheel conversion
  prices `LineDelta` with, so a notch answers 1.0 and the trackpad's
  pixel runs their honest fraction), computed at the door before the
  clamp arithmetic — a pinned run still names the run it was. The
  honest absence rides every story with no wheel run behind it: the
  AT's unit scrolls (their unit is the distance, M52), the value
  steps (the step is the distance), the app's own door, and the
  content's own clamp (ADR-0057); the non-finite run prices no count
  (the refusal names the wall, a count of a run that never was riding
  no wire). The wheel door prices the run where the delta is decoded;
  the wire's `scroll` event carries the additive `lines` field at
  version 1 (clients that ignore unknown fields unaffected), the
  embedded client rendering "the run priced 1 line" beside the
  landings, and `examples/wheel_lines.rs` is the consumer
  demonstration. No new corpus case — the field rides the route the
  wheel already rides; the corpus stands at 64, the living documents
  re-agree at the cut (M53, sixty decision records).
- **M52 — the asker's unit joins the scroll record (ADR-0058)**:
  ADR-0056's first kill rule cashed — a consumer needs the asker's
  unit named — one item or one page — on the AT's unit scrolls, the
  distance the ask is priced against — so `ScrollRecord` grows the
  additive `unit: Option<ScrollUnit>` beside the `ScrollUp`/
  `ScrollDown` actions, and `ScrollUnit` grows its `name()`
  (`item`/`page`, the protocol's `snake_case` rule). The honest
  absence rides every story with no unit: the wheel's coalesced
  run, the value step's page (a step, not a unit scroll), the app's
  own door, and the content's own clamp (ADR-0057) — never a
  guessed unit reconstructed from the distance. The AT action door
  fills the unit where the ask is decoded; the wire's `scroll`
  event carries the additive `unit` field at version 1 (clients
  that ignore unknown fields unaffected), the embedded client
  rendering "the scroll landed (the offset 140) by one page"
  beside the landings, and `examples/scroll_units.rs` is the
  consumer demonstration. No new corpus case — the field rides the
  routes the asks already ride; the corpus stands at 64, the
  living documents re-agree at the cut (M52, fifty-nine decision
  records).
- **M51 — the content-side clamp joins the causal log (ADR-0057)**:
  ADR-0056's third kill rule cashed — a consumer needs the content-side
  clamp named (`set_content_height` silently re-clamping the offset when
  the content shrank), so the content's own door grows its own fact:
  `ScrollAction::ContentClamp` (the door with no ask — the clamp is the
  content's story, not the ask's) beside the ask vocabulary, and
  `ScrollRefusal::NonFiniteContent` (the ask tier's `non_finite_ask`
  wall, at the content's door) beside the refusals. The delivered record
  carries the landing beside the wall it pinned (`pinned:
  Option<RangeBound>` — ADR-0054's one named-edges concept spoken
  identically wherever a landing names it, never a second action
  species), no ask beside it (a resize is not an ask) and no route
  behind it (the stamp's honest absence), computed through the same
  `pinned_offset_bound` arithmetic the ask doors answer by; a resize
  the offset never felt queues nothing (the M20 idle-frame property at
  the content's own door). The micro-debt the record's design priced is
  cashed with it: the non-finite content height that rode the clamp
  arithmetic and spread `NaN` into the offset and the stored span alike
  now refuses before the arithmetic, the state keeping the height and
  offset it held, the refusal a named fact on the log. The wire's
  `scroll` event carries the additive tags at version 1 (clients that
  ignore unknown strings unaffected), the embedded client rendering
  "the content clamp landed the offset 200, pinned at the max" beside
  the refusals, and `examples/content_clamp.rs` is the consumer
  demonstration. No new corpus case — the door rides no route and runs
  outside the frame's hot paths; the corpus stands at 64, the living
  documents re-agree at the cut (M51, fifty-eight decision records).
- **M50 — the scroll's story joins the causal log (ADR-0056)**:
  ADR-0055's second kill rule cashed — a consumer needs the wall named
  on a tier whose funnel knows no range (the list's scroll clamp), so
  the scroll tier grows its own record beside its species:
  `ScrollRecord` carries the action (`ScrollAction` — the wheel, the
  AT's unit scrolls and value steps, the app's own door), the landed
  offset beside the finite ask, the wall the clamp pinned
  (`pinned: Option<RangeBound>` — ADR-0054's one named-edges concept
  spoken identically wherever a landing names it), and the honest
  refusals named (`no_scrollable_content`, `non_finite_ask`). The trail
  is the value trail's shape (ADR-0046): the thread trail bounded at
  256, the doors noting where the answer resolves, the pipeline
  draining at the tick's start into `InspectEvent::Scroll`, the stamp
  riding the wheel and action routes (the M43/M45/M46 discipline at the
  scroll tier). The micro-debt the trail's design exposed is cashed
  with it: the non-finite wheel delta that rode the clamp arithmetic
  and spread NaN into the scroll state now refuses before the
  arithmetic, the refusal a named fact on the log. The wire's `scroll`
  event carries the additive fields at version 1 (clients that ignore
  unknown fields unaffected), the embedded client rendering "asked 450,
  landed 400, pinned at the max" beside the refusals, and
  `examples/scroll_trail.rs` is the consumer demonstration. No new
  corpus case — the routes the ask rides are already measured; the
  corpus stands at 64, the living documents re-agree at the cut (M50,
  fifty-seven decision records).
- **M49 — the funnel's wall is a named fact (ADR-0055)**: ADR-0053's
  second kill rule cashed — a consumer needs *which* bound the value
  pinned when a write was turned back at the range's edge, so the
  refusal's payload grows beside the species: `ValueRecord` grows
  `pinned: Option<RangeBound>` — the range's edge the funnel pinned
  the landing at (the clamp, the snap, the wall that held a no-op
  against a past ask), beside the action and outcome species, never a
  second action species (`ClampToMax` would munge the range's answer
  into an action the AT never asked). The absences are honest in both
  directions: a landing mid-range and every refusal carry `None` (a
  write that moved nothing pinned nothing), and the ask that wanted
  the wall exactly — the M47 jumps — carries `None` too (the species
  already names the bound it asked for). The slider's AT and arrow
  note sites compute the wall through one `pinned_bound` door (the
  read-only `Progress` carrying the absence at its `no_write_door`
  refusal), the wire's `value` event carries the additive `pinned`
  field at version 1 (clients that ignore unknown fields unaffected),
  the embedded client rendering "asked 150, landed 100, pinned at the
  max" beside the M44 clamp span, and `examples/pinned_writes.rs` is
  the consumer demonstration. No new corpus case — the routes the
  wall rides are already measured; the corpus stands at 64, the
  living documents re-agree at the cut (M49, fifty-six decision
  records).
- **M48 — the drag's wall is a named fact (ADR-0054)**: ADR-0053's
  first kill rule cashed — a consumer needs the bound jump the
  *pointer* asked for (a drag that slams an edge), so the M46 gesture
  record's vocabulary grows: `GestureRecord` grows
  `bound: Option<RangeBound>` — the range's edge the `Ended` record's
  landing pinned, beside the landing it produced, never the AT tier's
  species munged onto a pointer act. The beginning is not the wall's
  fact: every `Started` record and every mid-track landing carries
  the honest absence. The slider's release and the leave answer
  through one `landed_bound` door (the wall computed where the
  knowledge lives — the widget's own `total_cmp` equality against the
  range it owns), the M46 pointer route's stamp riding for free, the
  flood rule untouched (the moves still stream nothing — the wall
  rides the per-gesture record, never a per-move fact). The wire's
  `gesture` event carries the additive `bound` field at version 1,
  the embedded client rendering "pinned at the max" beside the
  landing, and `examples/drag_bounds.rs` is the consumer
  demonstration. No new corpus case — the route the wall rides is
  already measured (`gestures/route_100_drag_moves_1k`); the corpus
  stands at 64, the living documents re-agree at the cut.
- **M47 — the keyboard's jumps are named facts (ADR-0053)**: ADR-0051's
  second kill rule cashed — a consumer needs Home/End on the log, so
  the jumps answer with their own named species: `ValueAction` grows
  `JumpToMin`/`JumpToMax` beside `SetValue`, never a munged jump (the
  M11 rule's own words — the vocabulary the kill rule pre-registered,
  not a payload that never was). The slider's bounds' door
  (`jump_note`) streams the jump through the same shared route the
  step tier rides: the landing the funnel's own arithmetic, the bound
  the ask beside it (M44's discipline — the ask living where the
  knowledge lives, the clamp-and-snap never moving it), the M45 key
  route's stamp riding for free (the focused control's id on the
  record, one funnel, four named facts), and the no-op jump answering
  like the landed write it is (the M20 idle-frame property, the log
  naming it). The wire's `action` vocabulary grows the two
  `snake_case` tags additively at version 1 (the documented field
  values' rule — clients that ignore unknown field values
  unaffected), the embedded client rendering the species beside the
  record, and `examples/bound_jumps.rs` is the consumer
  demonstration.
- **M46 — the drag's story joins the log, per-gesture (ADR-0052)**:
  ADR-0046's second kill rule's open arm and ADR-0051's first kill
  rule cashed together — a consumer needs the drag on the log (the
  value tier's trail answers the AT's writes and the keyboard's steps,
  but the drag queued nothing, so a trail reader could not answer "why
  did the slider's value move while the hand dragged it"), priced the
  way the kill rules name: one per-*gesture* record, never per-move.
  One `GestureRecord` per phase — a `Started` the moment the press
  arms the drag (the value it found, before the press's own write
  lands), an `Ended` the moment the release or the leave resolves it
  (the value it left the control holding, the beginning remembered
  beside it — each record self-contained) — so one hundred moves
  inside one gesture stream exactly two records, the flood rule
  holding by construction. The `gesture` trail is the value trail's
  shape (`crates/alacer-core/src/gesture.rs`: the bounded queue, the
  A7 one-branch idle contract, the note site where the phase resolves,
  the tick's-start drain after the value drain), and the funnel's
  pointer route holds `GestureTargetGuard` — the deepest hit that
  captured (ADR-0025) is the *who* the drag's door cannot know, the
  M43 stamp discipline at the pointer tier, one `Cell` read while
  nobody listens. The wall (M22, ADR-0028) arms nothing and streams
  nothing: a gesture that never was is the honest absence. The wire
  grows the flat `gesture` event additively at version 1, the
  recorder's `gestures` ring carrying the trail beside the others, the
  embedded client rendering the phases, the corpus growing 63 → 64
  (`gestures/route_100_drag_moves_1k`, measured, published, and gated
  at this run — the coordinated rule), and `examples/drag_gesture.rs`
  is the consumer demonstration.
- **M45 — the keyboard's ride joins the value log (ADR-0051)**:
  ADR-0049's first kill rule cashed — a consumer needs the keyboard's
  value rides on the log *attributed*, so the focused slider's arrows
  stream the same `ValueRecord` the AT tier streams through the
  keyboard's own door: the dispatch funnel's key route holds the stamp
  the action route holds (the focused control's AccessKit-stable id,
  one `Cell` read while nobody listens — the A7 idle contract, never
  an id-path walk off an observer), the record carrying the step's own
  action species (`Increment`/`Decrement`), the landed value, M44's
  ask beside it, and the control the key rode. Home/End stay off the
  log (the record vocabulary has no bound action — a jump munged into
  a `SetValue` would be the M11 rule's own munging), and the drag
  stays off (the per-pointer-move flood the kill rule refuses,
  ADR-0046's second kill rule standing).
- **M44 — the value record shows the clamp (ADR-0050)**: ADR-0049's
  second re-open rule cashed — a consumer needs the *requested* value
  beside the landed one (a clamp the trail cannot show), so one
  additive `ValueRecord::requested` rides every value fact: the finite
  scalar the AT asked before the control's clamp-and-snap moved it,
  the fact living where the knowledge lives (the widget's own door
  computes the ask, the note site carries it — never a value
  reconstructed from the landing), the wire carrying it additively at
  version 1 (the `target` field's rule), the embedded client rendering
  "asked 150, landed 100" as data. `None` is the honest absence: the
  unparsable payload, the door refused before arithmetic, the ask
  itself non-finite (JSON cannot carry it — the species names the
  wall).
- **M42 — the clipboard read names its facts (ADR-0048)**: ADR-0047's
  third re-open rule cashed — a consumer needs the read side's facts
  the paste's `None` collapsed (the empty clipboard, the non-text
  payload, the unopenable clipboard) so the seam's read door grows
  beside the write door: one *provided* `ClipboardSeam::read_reported`
  answering `ClipboardReadFact` (the M37 provided-method discipline —
  the provided default maps the `Option` door and names its absence
  honestly, a real engine overrides and names its species; the
  `read_text` door stays the contract of record byte-for-byte, and a
  refusal through either inserts nothing). The species are the
  engine's own answers: `unnamed` (the honest absence), `unsupported`
  (no clipboard subsystem at all, or one that cannot open), `empty`
  (nothing the OS counts as content), `non_text` (content, but not
  text — an image), `os_error` (the unclassifiable) — the arboard
  engine splitting the empty and the non-text payload by the OS's own
  second answer (the image question asked only when the text read
  refused; each species the OS's answer to a question the engine
  asked). The round trip's record grows the additive `read` field
  beside the paste's verdict (`Some(species)` on a read-refused paste,
  `None` on a delivery, on the field's own refusal — a payload that
  sanitized to nothing, the read delivered — and on every write-side
  round trip: a copy never asked a read question), the M39
  `empty_clipboard` collapse split at last — one verdict, two facts,
  the log answering "did the paste refuse for the read, or for the
  field". The wire grows the same field at version 1 (clients that
  ignore unknown fields unaffected), the recorder's ring carrying the
  record verbatim, and the embedded client rendering the species
  beside the paste's refusal. The memory engine names its own known
  emptiness, so headless CI's paste refusals carry the species too.
  The trail's shape is untouched (the same 256-bounded queue, the same
  tick's start drain, one branch per chord while nobody listens), a
  refused read inserts nothing and queues no masks, and
  `examples/clipboard_read.rs` is the consumer demonstration.
- **M43 — the value record names its control (ADR-0049)**: ADR-0046's
  first re-open rule cashed — a consumer needs the *who* (the
  control's identity on the record) so the record grows beside the
  action's routing, never at the note site: one additive
  `ValueRecord::target` carrying the AccessKit-stable id hash the
  dispatch funnel routed, the stamp side living where the knowledge
  lives (`value::ActionTargetGuard`, held by
  `PassPipeline::dispatch_a11y_action` for the length of one widget
  call — the facts queued beneath the action carrying the id, a
  panicking widget leaving no stale identity, the widget never
  guessing its own identity). A multi-control form's trail is
  attributable at last: two sliders on one page, the facts tellable
  apart, the refusals (the parse's, the read-only tier's) naming the
  control that answered them beside the deliveries naming theirs. The
  wire grows the same field at version 1 (the clipboard `engine`
  field's rule — clients that ignore unknown fields unaffected), the
  recorder's ring carrying the record verbatim, and the embedded
  client rendering the control beside the record. The widget's own
  doors stay off the value log — the keyboard's step and the drag
  stream no value fact (the input tier's records carry those rides
  until a consumer prices them, the second kill rule's open arm), the
  attribution exclusive to the AT tier. The writes themselves are
  untouched (the funnel's clamp-and-snap answering byte-for-byte, the
  A7 idle contract one branch per write while nobody listens), and
  `examples/value_targets.rs` is the consumer demonstration.
- **M41 — the clipboard engine names its facts (ADR-0047)**: ADR-0045's
  own re-open rule cashed — a consumer needs the clipboard's
  *platform-side* facts the widget's round trips cannot name, so the
  seam's engine contract grows beside `ClipboardSeam`: one *provided*
  `write_reported` door answering `ClipboardWriteFact` (the M37
  provided-method discipline — the provided default maps the `bool`
  door and names its absence honestly, a real engine overrides and
  names its species; the `write_text` door stays the contract of record
  byte-for-byte, and a refusal through either deletes nothing). The
  species are the engine's own answers: `unnamed` (the honest absence),
  `unsupported` (no clipboard subsystem at all), `occupied` (another
  party held the clipboard), `conversion` (the payload the format
  cannot carry), `os_error` (the platform answered, the engine never
  classifies past its answer) — the arboard engine mapping its own
  error kinds onto the species in the windowed assembly. The round
  trip's record grows the additive `engine` field beside the
  `write_refused` verdict (`Some(species)` on a refused write, `None`
  on a delivery and on every other refusal — a paste reads, a
  no-selection copy never asked the engine), the wire growing the same
  field at version 1 (clients that ignore unknown fields unaffected),
  the recorder's ring carrying the record verbatim, and the embedded
  client rendering the species beside the refusal — the log answering
  "which clipboard failed it", not only "it failed". The trail's shape
  is untouched (the same 256-bounded queue, the same tick's start
  drain, one branch per chord while nobody listens).
- **M40 — the value tier rides the causal log (ADR-0046)**: the ride
  ADR-0010's additive rule and the M31/M33/M36/M37/M39 rhythm
  pre-register cashed. The value tier's AT actions (ADR-0026, the
  `Slider`; ADR-0027, the read-only `Progress`) answered the widget's
  own `bool`, whose `false` collapsed four different facts — the
  control the availability tier holds down, the payload that parsed to
  nothing, the payload the funnel cannot carry, the control that never
  had a write door — into one silence; a status line, an inspector
  trail, or a test could not answer "did the write land — where, or
  why not" without re-implementing the observation at the action site.
  Now every value-tier write the AT asks streams one
  `InspectEvent::Value` — the action beside the outcome and the value
  the control holds, the refusals named (`unavailable`,
  `invalid_payload`, `non_finite_value`, `no_write_door`), the refused
  writes streaming too. The two parse facts the `bool` collapsed are
  named apart at last ("loud" is `invalid_payload`, "inf" is
  `non_finite_value` — the m20 suite's two refusals), a no-op delivery
  answers like the landed write it is (the M20 idle-frame property,
  the log naming it), and a question outside the value tier (a press,
  a scroll, a text edit) is not a value fact. The fact is noted where
  the write resolves (`value::note_write`, the new trail module — the
  journal's own rule), and the value is the tree's own vocabulary the
  AT already reads, never a secret (the clipboard's count-not-payload
  rule was a secret's rule; the value tier has no payload to hide).
  The trail is the clipboard trail's shape (thread-local, 256-bounded,
  oldest-first eviction, the A7 gate — one branch per action while
  nobody listens), the tick's start drain running after the edit drain
  (a value write explains itself; it deletes nothing); the wire grows
  one `value` kind additively at version 1 (the clipboard event's flat
  rule — `docs/inspector-protocol.md` documents it), the recorder's
  ring holds 512 beside the other seams' rings, and the embedded client
  renders the echo in the causal log pane.
- **M39 — the clipboard rides the causal log (ADR-0045)**: the ride
  ADR-0010's additive rule and the M31/M33/M36/M37 rhythm
  pre-register cashed. The editing contract's clipboard chords
  (ADR-0020) answered the field's own `bool`, whose `false` collapsed
  three different facts — the selection that was not there, the
  clipboard that carried nothing, the platform that refused the write
  — into one silence; a status line, an inspector trail, or a test
  could not answer "did the text move — or why not" without
  re-implementing the observation at the chord site. Now every
  round trip the editing chords answer streams one
  `InspectEvent::Clipboard` — the action beside the outcome and the
  payload's UTF-8 scalar count, the refusals named (`no_selection`,
  `empty_clipboard`, `write_refused`), the refused round trips
  streaming too. The fact is noted where the round trip resolves
  (`clipboard::note_round_trip`, the seam module owning the trail —
  the journal's own rule), and the funnels were restructured so one
  round trip streams one fact: a cut writes its own record and never
  rides the copy funnel, the trail reading clipboard → edit (the
  causal order the round trip ran — what moved out, then the deletion
  it drove). The count names the fact; the payload never rides — a
  clipboard can carry a secret, and the log is a devtools surface,
  not a shadow clipboard. The trail is the edit trail's shape
  (thread-local, 256-bounded, oldest-first eviction, the A7 gate —
  one branch per chord while nobody listens), the tick's start drain
  running ahead of the edit drain; the wire grows one `clipboard`
  kind additively at version 1 (the announce event's flat rule —
  `docs/inspector-protocol.md` documents it), the recorder's ring
  holds 512 beside the other seams' rings, and the embedded client
  renders the echo in the causal log pane.
  `examples/clipboard_trail.rs` is the consumer demonstration: the
  beats run the real chords before the window opens (the honest
  `no_selection`, the counted copy, the cut that precedes the
  journal's step, the paste that lands what the cut moved, the locked
  door whose refused write never deletes) and the *trail* label
  mirrors what the log streamed; live chords play the same round
  trips in the window. The corpus stands at 63 (the no-growth arm —
  the trail is dev-mode and bounded, no new hot loop), and the
  living documents re-agree at M39 (the README's status block, count,
  quickstart, and table; the catalog; the register; the roadmap; the
  m-suite pins moving M38 → M39 and 45 → 46 across the chain; the
  protocol document's `clipboard` section).
- **M38 — the primary chord (ADR-0044)**: the platform's primary
  modifier as chord vocabulary — the re-open rule ADR-0043
  pre-registered (ADR-0042's before it) cashed. A chord's shape is
  *physical*, so an app's most common intent — "the platform's
  primary accelerator" (`Ctrl+S` on Linux/Windows, `Cmd+S` on macOS)
  — could not be named by the exact `Modifiers` shape: the app forked
  its table per platform, or reached for a stringly-typed chord the
  funnel cannot compare (the rule ADR-0041 refused). Now one
  `Chord::primary()` builder resolves at build time the way
  `Modifiers::primary` reads — Command on Apple platforms, Control
  elsewhere — no new field, no logical flag, no match-time platform
  context: the chord's shape *is* the event's physical shape, the
  exact-shape match (the M35 invariant) untouched and the funnel's
  compare the compare it always was. The resolution is the chord's
  identity: off Apple platforms a `primary()` chord *is* a
  `control()` chord — one binding per chord makes a rebind move in
  place, never shadow — and on Apple platforms the two are different
  chords, free for different actions. The aliasing never widens: a
  `Cmd+S` event off Apple platforms is a different shape, the consult
  falls through honestly, and the record carries the chord the funnel
  *asked* about. The `Chord::control` doc-drift is fixed, not the
  flag: the doc claimed "Control (Linux/Windows) or Command (macOS)" —
  a claim the physical shape never kept — and named a `primary()`
  builder that did not exist; it now pins the physical `control` flag
  and points at `Chord::primary`. The wire is untouched (the `key`
  event's `modifiers` field prints the resolved physical shape — the
  truth the wire always carried), and `docs/inspector-protocol.md`
  says so.
  `examples/primary_chords.rs` is the consumer demonstration: the
  real `dispatch_key` beats with the observer riding — the honest
  uninstalled window, the fired record carrying its name under the
  platform's own flag, the other platform's flag falling through
  without aliasing, the physical shape binding over the primary one
  *in place* — with tracing spans at `alacer::primary_chords`. The
  corpus stands at 63: the resolution happens at bind time (a `cfg!`
  branch, no match-time cost — the coordinated rule's no-growth arm).
  The living documents re-agree at M38 and 0.17.0 (the README's
  status block, count, quickstart, and table; the catalog; the
  register; the roadmap; the m-suite pins moving M37 → M38 and 44 →
  45 across the chain; the protocol document's `modifiers` sentence).

- **M37 — the binding names itself in the record (ADR-0043)**: the
  consult's prose name rides — the re-open rule ADR-0042 pre-registered
  on its own kill-criteria list cashed. The M36 record carried the
  chord beside the species (the chord the fired binding's identity,
  one binding per chord), but a trail reader that wanted the
  shortcut's *prose* name — a settings UI reading the log, an
  inspector row a human labels — had to keep its own chord → name
  table beside the log. Now the seam grows a *provided*
  `KeymapSeam::fired_name` beside `dispatch`: the provided `Keymap`
  table answers the binding its `dispatch` would run (the same match,
  the same chord scan), a custom seam names its own answer, and one
  that overrides nothing keeps the default `None` — the M35/M36
  custom seams compiling and streaming unnamed, the record honest
  about the absence. The answer is the record field's own
  `Option<String>` (a wrapper seam delegates through its guard, where
  a borrowed `&str` could not escape), one `consult_name` pure read
  answers it beside `consult` — running no action, the consult's
  dispatch staying the only door (never a second consult channel) —
  and `PassPipeline::dispatch_key` captures it *before* the consult's
  dispatch (the name the consult resolves to, never what a mid-flight
  rebind leaves behind) and only under an observer (the A7 idle
  contract: the unobserved funnel byte-for-byte). `KeyRecord` grows
  `name` beside chord and outcome — `Some` only on a fired consult, `None`
  on a miss or an uninstalled window or a nameless seam — trading its
  M36 `Copy` for `Clone` (the name is a heap `String`; the chord keeps
  the by-value ride). The wire grows the `key` event's `name` field
  additively at version 1 (the rider field's rule, ADR-0010), the
  embedded client renders the name beside the outcome, and
  `docs/inspector-protocol.md` documents the field.
  `examples/shortcut_names.rs` is the consumer demonstration: the real
  `dispatch_key` beats with the observer riding — the provided table's
  binding naming itself, the miss that names nothing, the custom
  seam's own answer, the nameless seam's honest absence, the rebind
  moving the name with the chord — with tracing spans at
  `alacer::shortcut_names`. The corpus stands at 63: the name leg
  costs the unobserved funnel nothing and rides the published keymap
  route's consult (the coordinated rule's no-growth arm, the
  M31/M33/M36 rule). The living documents re-agree at M37 and 0.17.0
  (the README's status block, count, inspector bullet, quickstart,
  and table; the catalog; the register; the roadmap; the §13.3
  questions table; the m-suite pins moving M36 → M37 and 43 → 44
  across the chain; the protocol document's `key` section).

- **M36 — the keymap rides the causal log (ADR-0042)**: the consult's
  answer as data — the re-open rule ADR-0041 pre-registered on its own
  kill-criteria list cashed. The M35 funnel consult answered a bare
  `bool` whose `false` collapsed three different facts; now one
  `KeymapConsult` (`Uninstalled` / `FellThrough` / `Fired`, `fired()`
  the verdict accessor, `name()` the wire's tag) and one `consult`
  door answer the read as data with `dispatch_installed` staying the
  fired verdict (the M35 contract byte-for-byte), and
  `PassPipeline::dispatch_key` streams one `InspectEvent::Key`
  (`KeyRecord`: the chord beside the species) per consult it runs —
  the moment it resolves, before the fallbacks' focus records, so a
  key event's trail reads keymap → focus → input → frame. The chord is
  the fired binding's identity (one binding per chord), the focused
  widget's precedence is visible as an absent record, and `Key`/
  `Chord` gain the additive `Copy` the by-value record rides. The
  trail is dev-mode and bounded: one branch per consult when no
  observer is installed (the A7 idle contract,
  `PassPipeline::set_observer` enables it), the recorder growing the
  key ring beside its rings (`CausalRecorder::keys()`, 512). The wire
  grows the `key` event additively at version 1 (the rider field's
  rule, ADR-0010), the embedded client renders the echo beside the
  focus and edit trails, and `docs/inspector-protocol.md` documents
  the event.
  `examples/keymap_trail.rs` is the consumer demonstration: the real
  `dispatch_key` beats with the observer riding — the uninstalled
  window's honest answer, the fired chord naming itself, the miss
  that falls through, the rebind moving the record's chord with it —
  with tracing spans at `alacer::keymap_trail`. The corpus stands at
  63: the record rides the published keymap route's consult and
  streams only under an observer (the coordinated rule's no-growth
  arm, the M31/M33 rule). The living documents re-agree at M36 and
  0.17.0 (the README's status block, count, inspector bullet,
  quickstart, and table; the catalog; the register; the roadmap; the
  §13.3 questions table; the m-suite pins moving M35 → M36 and 42 →
  43 across the chain; the protocol document's `key` section).

- **M31 — the journal rides the causal log (ADR-0037)**: every
  `EditHistory` action streams one `InspectEvent::Edit` — the hook the
  journal's own docs pre-registered ("the inspector may *someday*
  surface the depth") cashed. The record carries the action
  (`EditAction`: `record`/`undo`/`redo`/`reset`, `name()` the wire's
  tag), the `coalesced` flag (the pure-append fold visible as a depth
  that never moves), the `refused` flag (an exhausted undo or an empty
  redo branch — the refusal streams too, the depths naming the journal
  it did not move), and the depths the action left. The journal queues
  the fact on the editing thread (the clipboard-seam shape, ADR-0020 —
  handlers cannot reach the pipeline, axiom A1) and the dispatching
  pipeline drains it at the tick's start (the announcement drain's
  shape, ADR-0024) — one frame after the chord. The trail is dev-mode
  and bounded: one branch per action when no observer is installed
  (the A7 idle contract; `PassPipeline::set_observer` enables it), a
  256-fact queue with the journal's own oldest-first eviction (the
  §14.3 rule applied to the echo). The wire grows the `edit` event
  additively at version 1 (the rider field's rule, ADR-0010), the
  recorder grows the bounded edit ring beside its rings
  (`CausalRecorder::edits()`, 512), and the embedded client renders
  the echo beside the focus trail.
  `examples/edit_trail.rs` is the consumer demonstration: the M15
  journal driven with the observer riding — the coalescing, the
  applied chords, the app-write reset, the exhausted journal's
  refusal — with tracing spans at `alacer::edit_trail`. The corpus
  stands at 62: the echo rides no hot loop the corpus does not
  already price (the coordinated rule's no-growth arm).

- **M32 — the value tier joins the view registry (ADR-0038)**: the
  re-open rule the M6 canon's registry work left open cashed — the
  `slider` and `progress` elements join `default_registry`, so the
  value tier's widgets are `rsx!` vocabulary like the M6 canon before
  them. Create reads the value props (`name`/`min`/`max`/`step`/
  `value`/`on_change`/`accent`, `fill`, `binding`); patch takes what
  has an in-place writer, and the patch answers are the widgets' own:
  the slider's programmatic write clamps and snaps inside the widget,
  fires no handler, and reports visual only when the funnel moved the
  value (the widget's own `total_cmp` equality — never `f32::eq`);
  the progress's `set_value` bool is the M21 idle-frame property — a
  write that lands where the value already is queues no masks. The
  prop vocabulary grows the two types the tier needs:
  `PropValue::NumberHandler` carries the funnel's value (`on_change` —
  the flag/text handlers' rule at a value, the patcher's
  handler-churn exemption covering it), and `PropValue::NumberBinding`
  rides the f32 `RenderBinding` (`binding: my_f32_binding` — the value
  typed by what it carries; a bound progress's patch re-hangs the same
  slot idempotently, the bound label's rule at the value tier). The
  bridge still owns the value: the assembly-time `bind_value` the
  `counter_view` rule names — the DSL expresses the shape, the flux
  bridge expresses the reactivity, neither grows a second channel
  (axiom A1). `examples/value_view.rs` is the consumer demonstration:
  the slider's funnel routing to one signal, the bridge fanning the
  write out to the bar, the label, and the tree's answer. The corpus
  stands at 62: the elements ride the published view routes (the
  coordinated rule's no-growth arm).

- **M33 — the announcement rides the causal log (ADR-0039)**: every
  speech act the pipeline answers streams one `InspectEvent::Announce`
  — the hook the M18 contract's own refusals pre-registered ("why did
  the live region speak — or refuse") cashed. The record carries the
  politeness the act asked for and the outcome it got
  (`AnnounceOutcome::name` the wire's tag: `delivered`, `superseded` —
  the last-write-wins rule visible as data, the batch draining whole —
  or `refused` with the named species `AnnounceRefusal::{EmptyText,
  NoRoot}`; the refusals stream too, the M28 honesty at the speech-act
  tier). The facts stream where the answer resolves: the direct door's
  answers at the door (`PassPipeline::announce`), the push-side empty
  refusal on the thread's trail (the clipboard-seam shape, ADR-0020),
  the drain's batch answers at the tick's start (the edit trail's
  shape, ADR-0037). The trail is dev-mode and bounded: one branch per
  act when no observer is installed (the A7 idle contract;
  `PassPipeline::set_observer` enables it beside the edit trail's), a
  256-fact queue with the §14.3 oldest-first eviction. The wire grows
  the `announce` event additively at version 1 (the rider field's
  rule, ADR-0010), the recorder grows the bounded announce ring beside
  its rings (`CausalRecorder::announces()`, 512), and the embedded
  client renders the echo beside the focus and edit trails.
  `examples/announce_trail.rs` is the consumer demonstration: the M18
  seam driven with the observer riding — the delivered act, the empty
  refusal, the storm's superseded facts beside the survivor — with
  tracing spans at `alacer::announce_trail`. The corpus stands at 62:
  the speech acts ride the published announcement route (the
  coordinated rule's no-growth arm; the route re-measured at this cut:
  18.62 ms median against the published 18.81 ms, -1.0%).

- **M34 — the backend features are true (ADR-0040)**: the capability
  the feature names pre-registered (`backend-wgpu` /
  `backend-softbuffer` forward to optional dependencies, promising
  single-backend builds the tree never delivered — the platform's
  backend modules, enum, and construction paths compiled
  unconditionally, so `--no-default-features` failed with 151 errors
  and every single-backend combination failed at `cargo check`)
  cashed. Each present path compiles with its feature: the modules,
  re-exports, `Backend` variants, and per-backend builders gate with
  the feature that owns them — all four combinations compile
  warning-free under `-D warnings` (both backends — the default, its
  behavior unchanged — each alone, and none). A preference whose
  backend the build compiled out refuses honestly at startup:
  `PlatformError::BackendUnavailable { requested }` (the preference's
  own name, `BackendPreference::name` the single source) — the display
  naming the gate that answers it, the chain ending at the build fact
  (the M16 boundary's leaf rule at the build tier; never a panic,
  never a munged fallback). CI gains the features job exercising all
  four combinations: the no-backend build compiling and running its
  refusal tests, each backend alone, and the facade's software-only
  and pipeline-only builds.

- **M35 — the keymap seam: the app's shortcuts ride the dispatch
  contract (ADR-0041)**: the keyboard's window-level half ships — the
  actions an application names (save, find, export) answer wherever
  the focus sits, through one [`KeymapSeam`] the assembly site
  installs (the clipboard seam's thread-local discipline, ADR-0020:
  `Chord` names one logical `Key` plus the exact `Modifiers` shape it
  answers — `Ctrl+S` never answers `Ctrl+Shift+S`, that is a
  different chord free for another action; `Keymap` is the provided
  table, one binding per chord, `bind` replacing in place so the
  settings story is a move and never a shadow, `unbind` returning the
  chord to the framework's fallbacks, `names` the introspection a
  settings read-back walks; the install trio `install` /
  `is_installed` / `dispatch_installed`). `PassPipeline::dispatch_key`
  consults the seam when the focused widget did not answer and before
  the framework's own fallbacks — the precedence made three-tier and
  pinned by the m35 exit suite: the focused widget's vocabulary (a
  focused input's Space types, never fires), the app's shortcuts (the
  explicit beats the implicit: a bound plain `Tab` holds focus while
  the unbound `Shift+Tab` — a different chord — still walks
  backward), and the framework's Tab cycling (M24) and directional
  ring (M29) standing exactly where the app stayed silent. The
  consult needs no focused widget at all: window-level vocabulary
  answers wherever the focus sits, and a shortcut is focus-neutral —
  the fired action queues no masks of its own (the M21 idle-frame
  property at the app seam). The press-only match carries the GTK
  activation rule to the window tier (a release is never a second
  firing, an auto-repeat one run per physical press), the fired
  binding names itself on the `alacer::keymap` target (the house
  tracing idiom), and the seam is optional so the read is total — no
  panic door, the clipboard's rule inverted for an optional seam; a
  window with no shortcuts answers byte-for-byte today's behavior
  (the M11 rule at the dispatch seam, the A7 idle contract
  unchanged). `examples/shortcut_map.rs` is the consumer
  demonstration (tracing at `alacer::shortcut_map`). The corpus
  grows 62 → 63 by the coordinated rule: the keymap's mixed route
  (`keymap/dispatch_100_mixed_1k` — one thousand events alternating
  a miss and a mid-table hit against one hundred bindings, through
  the funnel's own read) joins the published routes with its gate
  and its baseline in the same cut (the m5–m30 chain's pins moving
  together).

- **M30 — the trail names the rider (ADR-0036)**: every `FocusRecord`
  the causal log streams grows a `rider` field beside its `movement` —
  a `FocusRider` naming who rode the walk: `Keyboard` (the keyboard's
  Tab/Shift+Tab fallback, the M24 seam), `Ring` (the four directional
  keys' fallback, the M29 seam — a D-pad button, a TV remote's ring),
  or `Consumer` (the app's own `focus_move`/`focus_move_in` door
  calls) — ADR-0035's pre-registered re-open rule ("a consumer needs
  the rides distinguished in the causal log → grow `FocusRecord`
  beside its `movement` (a rider field), priced by a real reader —
  never a second trail") cashed. One funnel, three riders, one trail:
  the attribution is declared where the route is known — the public
  doors stream `Consumer`, the dispatch fallbacks stream through one
  private rider-aware door (`Keyboard` for Tab, `Ring` for the
  arrows), the walk behind the doors untouched (the same candidate
  list, the same availability filter, the same wrap, the same
  immediate resolution, the same no-mask idle refusals — the M11 rule
  stays). The wire grows the `rider` field (`keyboard` / `ring` /
  `consumer`, `FocusRider::name` the single source) as the additive
  kind the protocol's versioning rule admits at version 1 —
  `PROTOCOL_VERSION` stays 1 — and the embedded client renders the
  rider beside the movement (M31). `examples/focus_riders.rs` is the
  consumer demonstration: the M26 window walked by all three riders,
  every beat logging the record's rider beside the answer it names,
  with tracing spans at `alacer::focus_riders`. The corpus stands at
  62: the riders ride the published routes (the coordinated rule's
  no-growth arm).

- **M29 — the directional keys ride the walk (ADR-0035)**: a plain
  directional key the focused widget did not answer — a D-pad button,
  a TV remote's ring — maps onto its `FocusMove` at
  `PassPipeline::dispatch_key`'s fallback (the seam Tab and Shift+Tab
  ride since M24) and lands the geometry's answer through the real
  dispatch path — ADR-0032's pre-registered re-open rule cashed. The
  platform's key map already carries the four arrows
  (`alacer-platform/src/keys.rs`), so the seam is one `match` — no
  second dispatch channel. The widget's answer always takes precedence
  (a focused input's caret, a slider's step — the m20/m11/m13
  contracts stand), a modifier held means the key belongs to whatever
  vocabulary answers it (the chords are the widgets'; the window's
  walk is never a chord's rider), and a refused direction falls
  through honestly (`EventResponse::Ignored`, the refused Tab's
  shape) with the M28 trail carrying the arrows' rides like every
  ride — one funnel, now three riders: Tab, the code, the ring.
  `examples/dpad_navigation.rs` is the consumer demonstration: the
  M26 window walked by arrow presses through the real dispatch, with
  tracing spans at `alacer::dpad_navigation`. The corpus stands at 62:
  the arrows ride the published routes (no new criterion case).

- **M28 — the walk joins the causal log (ADR-0034)**: every focus-move
  door ride streams one `InspectEvent::Focus` the moment the door
  resolves, carrying a `FocusRecord` (the movement the caller asked
  for, the scope that bounded the walk — `None` for the unscoped
  funnel — and the door's own `FocusMoveOutcome` verbatim) —
  ADR-0033's pre-registered re-open rule ("a consumer needs to observe
  the walk's refusals as *streamed events*") cashed. One funnel, two
  doors, one trail: the keyboard's Tab fallback rides `focus_move`, so
  its rides stream the same records the consumer's code streams, and
  the scoped door's dead-scope refusal streams before the walk runs.
  The walk behind the doors is untouched (the same candidate list, the
  same availability filter, the same wrap, the same immediate
  resolution, the same no-mask idle refusals — the M11 rule stays);
  without an observer the doors pay one branch per call (the A7 idle
  contract unchanged). The recorder grows the bounded focus ring
  beside its frames/edges/writes/inputs (`FOCUS_RING_CAP`, oldest
  dropped first — the §14.3 discipline; `CausalRecorder::focus()` is
  the read door), and the wire grows `WireEvent::Focus` with the
  documented flat shape (`movement`, `scope`, `moved`, `from`, `to`,
  `wrapped`, `refusal`, `node`, `direction`) — a new event kind,
  additive by the protocol's own versioning rule, `PROTOCOL_VERSION`
  stays 1. `examples/focus_trail.rs` is the consumer demonstration:
  the M26 window walked one beat per `FocusMove` with the observer
  riding, every beat logging the streamed record beside the door's
  own answer, with tracing spans at `alacer::focus_trail`. The
  observed route (`input/route_100_observed_moves_1k`) joins the
  published corpus beside the four focus routes, the corpus pins
  moving 61 → 62 across the m5–m27 chain in the same cut.

- **M27 — the walk's answer as data (ADR-0033)**: the two focus doors
  (`PassPipeline::focus_move`, `PassPipeline::focus_move_in`) grow their
  answer from the M24-era boolean into the `FocusMoveOutcome` vocabulary —
  ADR-0031's pre-registered re-open rule ("a door needs to observe the
  walk's refusals as data") cashed. A `Moved(FocusLanding)` carries the
  endpoints (`from` — `None` for the entry — and `to`) and the wrap (the
  tree-order move crossed a list end; the entry and the directional moves
  never carry it); a `Refused(FocusRefusal)` names its species — the
  empty list, the dead scope, the geometry-less focus, the empty
  direction (with its node and direction), the single-candidate stall —
  instead of five refusals collapsed into one silent `false`. The walk
  behind the doors is untouched: the same candidate list, the same
  availability filter, the same wrap, the same immediate resolution, the
  same no-mask idle refusals (the M11 rule stays; the answer stops
  making the caller guess), the per-species help staying plain text on
  the `alacer::focus` target (ADR-0029's rule), the keyboard's Tab
  fallback routing on the `moved()` verdict accessor unchanged.
  `examples/focus_outcome.rs` is the consumer demonstration: the M26
  window walked one beat per `FocusMove`, every beat logging what the
  walk answered as data, with tracing spans at `alacer::focus_outcome`.
  The outcome route (`input/route_100_outcome_moves_1k`) joins the
  published corpus beside the three focus routes, the corpus pins moving
  60 → 61 across the m5–m26 chain in the same cut. The repair debt the
  sweep priced rides the cut: the M23-era required `SceneJournal::stats`
  never landed on three `NullJournal` impls (the `pass_overhead` bench
  and two test harnesses — the M23 cut's own benches and tests could not
  compile at the shipped HEAD), the three focus-route baselines the M26
  cut promised at its measurement run and never published (the m5
  bijection red at HEAD, `baselines.json` stalled at 57 entries against
  60 pins) land this cut, the `focus_trap`/`focus_traversal` examples'
  unboxed multi-line-closure buttons and unused `Widget` imports go (the
  examples did not compile at HEAD), and the dead `Slider::set_on_change`
  and the dead `AccessKitSink::is_initialized` (its only caller was its
  own unit test — the M25 species) go with them.

- **M26 — the directional focus walk (ADR-0032)**: the walk reads the
  window, not only the tree. ADR-0030's pre-registered re-open rule
  (spatial/D-pad navigation) ships as four `FocusMove` directions
  (`Left`/`Right`/`Up`/`Down`) riding the same two doors
  (`PassPipeline::focus_move`, `PassPipeline::focus_move_in`) and the
  same one walk (`cycle_focus_in(scope, movement)` — the boolean the
  M25 walk took becomes the movement it aims). The tree-order moves
  keep the pre-order list and their wrap; the directional moves read
  the bounds pass 6 wrote (`directional_neighbor` — a candidate
  qualifies fully beyond the focused node's edge in the direction, the
  nearest edge distance wins, cross-axis center misalignment breaks
  ties, then tree order), the M22 availability filter runs before the
  geometry answers, the scope bounds the directional walk exactly as it
  bounds the tree-order one, and space never wraps — the window's edge
  is the walk's end, an edge refusal queues no masks (an idle frame),
  and the refusals carry their help as plain diagnostic text on the
  `alacer::focus` target (ADR-0029's rule). The entry contract is
  repaired to what ADR-0031 has documented since M25: from no focus —
  or from a focus outside the filtered candidate list — the walk takes
  the list's first candidate, both directions (the M25-era arithmetic
  landed the second forward and the last backward).
  `examples/focus_direction.rs` is the consumer demonstration: a
  two-by-two grid where the geometry's answer and the tree's next stop
  disagree, the corner's honest refusal while `Next` still wraps, and
  the modal's form walked directionally inside its scope, with tracing
  spans at `alacer::focus_direction`. The micro-debt the sweep priced
  rides the cut: the selection's dead `is_degenerate` and the color's
  dead `to_premultiplied_rgba8` (its only caller was its own unit test
  — the M25 species), and the corpus debt the M24/M25 coordinated rule
  left rides the cut: the promised focus routes
  (`input/route_100_focus_moves_1k`, `input/route_100_scoped_moves_1k`)
  join the published corpus beside the directional route
  (`input/route_100_directional_moves_1k`) at this cut's measurement
  run, the corpus pins moving 57 → 60 across the m5–m25 chain in the
  same commit.

- **M25 — the focus scope (ADR-0031)**: the focus walk bounds to a
  subtree. ADR-0030's pre-registered re-open rule ships as one
  `PassPipeline::focus_move_in(scope, movement)` door beside the
  unscoped funnel — the modal's focus trap priced by the real UI it
  names. The scope is the subtree's root node (the cheapest honest
  predicate specialization: no closures in the public API, no second
  walk), the walk generalizes behind both doors (`cycle_focus_in` — the
  same pre-order candidate list pass 2 recomputes, the M22 cycle-time
  `enabled` filter, the wrap at both ends, the immediate resolution),
  the scope bounds the walk never the candidate list (`focusables()`
  and `focus()` keep answering the whole tree, the unscoped door stays
  the keyboard's funnel, and a root-scoped walk lands exactly where
  `focus_move` lands), and the refusals are honest and named — a dead
  scope, an empty trap, or a trap the availability tier holds down
  answers `false` with no masks (an idle frame, never a
  first-candidate reset), their help plain diagnostic text on the
  `alacer::focus` target (ADR-0029's rule). The landings and refusals
  ride the `alacer::focus` tracing target, and
  `examples/focus_trap.rs` is the consumer demonstration: a background
  control and a modal's form walked forward, backward, and into the
  refusal inside the trap, with the window's Tab proving the unscoped
  door live. The micro-debt the sweep priced rides the cut: the layout
  mapping's dead `fixed_size_of` and the color's dead `to_f32_rgba`
  (documented for a GPU-uniform consumer that never landed), and the
  living documents re-agree at M25 and 0.16.0 — the README badge and
  status block riding the pending-cut convention, the migration guides'
  family line among them, and the m7 release-state pin re-joining the
  chain after missing the M23/M24 sweeps.

- **M24 — the focus walk (ADR-0030)**: the focus walk joins the
  dispatch contract. Tab's private cycle becomes public vocabulary: one
  `FocusMove` direction (`Next`/`Prev`) and one pipeline door —
  `PassPipeline::focus_move` — that the keyboard's Tab/Shift+Tab
  fallback and the consumer's code both ride: one funnel, two doors,
  the walk's order never duplicated at the call site. The walk follows
  the pre-order
  candidate list pass 2 recomputes, skips controls the availability
  tier holds down (the M22 cycle-time `enabled` check), wraps at both
  ends, resolves immediately (key routing sees the new focus this
  event), and refuses honestly when no candidate exists — no masks, an
  idle frame, never a first-candidate reset. The landings and refusals
  ride the `alacer::focus` tracing target, and
  `examples/focus_traversal.rs` is the consumer demonstration: a
  four-control form walked forward, backward, and into the refusal,
  with the window's Tab proving the funnel live. The micro-debt the
  sweep priced rides the cut: the flux runtime's dead `effect_count`
  introspection and the harness's dead `spec_of`/`with_text_context`/
  `text_size` probes, and the living documents re-agree at M24 — the
  m1 suite's header among them, which had said "36 blessed snapshots"
  over a corpus its own pin counts at 37.

- **M23 — the frame's ledger (ADR-0029)**: the scene journal becomes
  legible to its consumers. Every seam since M16 paired with a consumer
  demonstration, but the M2 headline — the journal whose economics every
  other seam spends — stayed undemonstrated, and its introspection
  accessors were dead code precisely because no consumer existed. One
  required `SceneJournal::stats()` seam ships, carrying the
  `JournalStats` vocabulary (commits, drops, live layers) with
  `PassPipeline::journal_stats()` as the pipeline's one read door; the
  consumer demonstration ships: `examples/frame_ledger.rs` reads the
  ledger at the two beats that matter — the mount frame (every layer
  commits once) and one ticker write batch (exactly one layer
  re-commits, the static layer's clean pixels replaying untouched) —
  with the interactive *Tick* press riding the same one-batch route and
  the ledger on the `alacer::frame_ledger` tracing target. The m1
  corpus's broken trio is repaired at the source: the `list_scrolled`
  scenarios never fed `set_content_height`, so the scroll clamp folded
  every offset to zero and three distinct scroll positions blessed one
  byte-identical image — the shared helper now derives the content
  height from the laid-out children, `list_scrolled_120` grows to twelve
  panels so its 120 px target is inside the clamp's real range, the
  m23 exit suite pins the delta headlessly and the three images
  pairwise-distinct, and the next bless run re-blesses them distinct
  (CONTRIBUTING's diff-is-the-review rule). The micro-debt the sweep
  priced rides the cut: the journal's dead inherent accessors
  (`commits`, `drops`, `layer_count`, the slice-view `layer` duplicate)
  replaced by the seam that gives their data a caller, the todo canon's
  dead `footer_node` getter with its write-only field, the `rsx!`
  body-cardinality diagnostic that now names its cardinality with
  miette-class `help:` text (plain diagnostic text — the
  no-new-dependency rule outranks adopting the crate), and the
  inspector's accept loop reporting the session errors its silent
  `let _ =` swallowed (through the adopted `tracing` core — the one
  dependency edge the cut adds, and it stays inside the resolved graph).
  The living documents re-agree at 0.14.0: the README's badge, status
  block, quickstart, perf bullet, and example table; the catalog; the
  architecture register (28 → 29 with ADR-0029); the roadmap; the
  migration guides' family line; the widgets manifest's shipped
  vocabulary; the facade's shipped-surface rustdoc; and the m20/m21/m22
  pins move with them in the same commit
- **M22 — the unavailable control (ADR-0028)**: the availability tier
  joins the widget contract. The ledger M21 emptied had no value-tier
  candidate left, and the audit-before-the-cut pass priced the property
  every real form needs beside it: no widget could *report that it must
  refuse* — a spent button looked, read, focused, and answered exactly
  like a live one, the AT announced a control the tree claimed was
  actionable, and the press died silently. One `Widget::enabled` source
  of truth ships: the default `focusable()` derivation consults it (an
  unavailable control is never a Tab stop and never a focus candidate),
  pass 3's sentinel discipline evicts focus from a control that becomes
  unavailable, and every door refuses honestly — the pointer is a wall
  that never clicks through (the HTML `disabled` semantics), the
  keyboard and the IME are answered with silence, and every AT action is
  refused, never munged (the M11 rule). The flag rides pass 9's
  snapshot — `TreeNode.enabled`, no second channel — and the a11y sink
  maps it faithfully onto AccessKit's `disabled` property, so the AT
  reads the refusal before it happens. Two doors land the same
  `PAINT | A11Y` currency: the fixed door (`.enabled(bool)` at
  construction, `set_enabled` in place) answers the move it made — a
  write that lands where the flag already is queues nothing — and the
  button's binding door (`enabled_binding`) tracks a `RenderBinding<bool>`
  the flux bridge drives (the `Progress::bound` rule, at the
  availability tier). The look tells the truth the tree carries: the
  muted paint the theme constants borrow, with the shipped corpus's
  enabled pixels untouched. The consumer demonstration ships:
  `examples/disabled.rs` drives the bound door from the consumer side
  (the *Enable sending* checkbox's one write batch) and walks the mutate
  seam at startup, with `tracing` spans at the new `alacer::disabled`
  target. The coordinated corpus completes: the refused press's criterion
  case `disabled/route_100_refused_presses_1k` is measured (100
  samples), published with its baseline entry, and gated same-host —
  corpus 56 → 57, every M-suite pin moved together — and the m1 corpus
  grows 36 → 37 with the availability window. The micro-debt the sweep
  priced rides the cut: the software renderer's dead `glyphs_rendered`
  (the M21 gate's own blind spot — the same-named accessor the wgpu cut
  removed still shipped here), the dead `PassPipeline::clear_observer`
  and `PassPipeline::timings_enabled`, and the `ViewRoot::root`
  duplicate whose doc claimed a caller that does not exist; the `rsx!`
  fall-through diagnostic names what stood where an element was
  expected, and the unterminated-`{` text error carries the escape
  hatch's `help:`. The living documents re-agree: the README's count,
  bullet, table, and status block, the catalog, the architecture
  register (28 → 29), the roadmap, and this changelog moved with it
- **M21 — the reported value (ADR-0027)**: the value tier grows its read
  half. The ledger M20 emptied had one unclaimed completion: the AT can
  *write* a value, but an app had no widget-shaped way to *report* one.
  The progress indicator ships as `Progress`
  (`crates/alacer-widgets/src/progress.rs`) over a new
  `Role::ProgressIndicator` in core — requires-name (an unnamed report
  reads as a bare number, ADR-5), not focusable (a report holds no
  focus), and carrying the app's bounded scalar through the M3
  `value()` contract, with the a11y sink mapping the role faithfully
  (`AccessKitRole::ProgressIndicator`). Two app doors land the same
  `PAINT | A11Y` currency: the binding bridge (one write batch, the
  caller's masks — the reactive-text seam, at a value) and the mutate
  seam (the pass-4 queue's lease, the write in place, the write's own
  masks on the `WidgetMut` guard), whose `set_value` answers the move it
  made so a write that lands where the value already is queues nothing —
  the idle-frame property, at the app seam. The AT tier is read-only by
  construction: every action is refused honestly, never munged (the M11
  rule). The consumer demonstration ships: `examples/progress.rs` drives
  both doors with `tracing` spans at the new `alacer::progress` target —
  the mutate seam's first consumer demonstration. The coordinated corpus
  completes: the value-write route's criterion case
  `progress/route_100_value_writes_1k` is measured (100 samples),
  published with its baseline entry, and gated same-host — corpus
  55 → 56, every M-suite pin moved together — and the m1 corpus grows
  35 → 36 with the reported value's window. The micro-debt the audit
  priced rides the cut: the dead `PassPipeline::has_observer` and the
  wgpu backend's dead stats surface (`glyphs_rendered`,
  `instance_frames_skipped`, `atlas_clears`) go. The living documents
  re-agree: the migration guides' 0.12.0 self-description names the
  shipped family again (the M16 pattern's remainder), and the README,
  the catalog, the architecture register (27 → 28), the roadmap, and
  this changelog moved with it
- **M20 — the valued widget (ADR-0026)**: the slider joins the widget
  vocabulary and the value tier becomes a widget's contract. The
  ledger's last feature candidate ships as `Slider`
  (`crates/alacer-widgets/src/slider.rs`) over a new `Role::Slider` in
  core — requires-name (an unnamed value control reads as a bare
  number, ADR-5), focusable, and carrying its bounded value through the
  M3 `value()` contract, with the a11y sink mapping the role faithfully
  (`AccessKitRole::Slider`). One funnel, four doors: a drag writes the
  value from the pointer's x against the node's resolved frame (the
  capture route ADR-0025 shipped keeps the thumb with the hand), the
  arrow keys step by the step, Home/End jump to the ends, and the AT's
  tier — `SetValue` (parsed, then snapped like every write),
  `Increment`, `Decrement` — answers through the same action route the
  editor and the scroll area hold. Every write clamps to the range and
  snaps to the step inside the widget; a write that lands where the
  value already is queues no masks (the idle-frame property, at the
  widget tier); a non-numeric or non-finite request is refused
  honestly — never munged into a step, the M11 rule. The consumer
  demonstration ships: `examples/slider.rs` (one slider, one bound
  status line mirroring the value the tree carries), pinned by the
  widget's unit tests and the new m20 exit suite. The coordinated
  corpus completes M19's promise: the capture route's criterion case
  `input/route_100_captured_moves_1k` is measured (100 samples),
  published with its baseline entry, and gated same-host — corpus
  54 → 55, every M-suite pin moved together — and the m1 corpus grows
  33 → 35 with the promised drag-capture scenario and the slider's own
  window. The living documents re-agree: the README's badge and status
  block finally quote the 0.13.0 family the manifest bumped (the M16
  pattern's remainder), the vocabulary list names the widgets it
  ships, and the catalog, migration guides, architecture register
  (26 → 27 with ADR-0026), and roadmap moved with it
- **M19 — the captured drag (ADR-0025)**: the pointer capture joins
  the dispatch contract. Pass 1 hit-tested every event independently,
  so a drag that wandered off its target lost its route mid-gesture —
  the M12 drag-select clamped at the field's edge, and a
  press-inside/release-outside button stayed pressed with no release
  ever arriving. A press now implicitly holds its deepest accepting
  hit (single-pointer — the event vocabulary carries no pointer id,
  ADR-0014's shape), the gesture's `Move`/`Up`/`Leave` stream routes
  through the captured node's ancestor chain with the same
  capture→bubble phases, the `Up`/`Leave` and the target's death
  release the capture honestly, the wheel stays hit-tested (it scrolls
  the hover, not the gesture), and a frame with no capture in flight
  is the per-event hit test exactly as before — one `Option` check on
  the hot path. `PassPipeline::captured` introspects the holder and
  the capture route traces at the new `alacer::capture` target (the
  per-seam idiom). The consumer demonstration ships:
  `examples/drag_capture.rs` (press the panel, drag past its edge, the
  log keeps counting; wave without pressing and only the moves inside
  the frame arrive), pinned by the pipeline's unit tests and the new
  m19 exit suite. The surface work rides the cut: the dead
  `EventCoalescer::pending` goes (zero call sites workspace-wide, the
  M18 pattern), and the `rsx!` single-root diagnostic names the token
  it found with its help text instead of discarding it. The living
  documents re-agree: the README's criterion count finally quotes the
  54-case baseline, the architecture's §10.1 routing spec states the
  capture it ships (the ADR register grows to 26 with ADR-0025), and
  the examples catalog carries the demonstration. The capture route's
  criterion case and the example's blessed-corpus scenario join at the
  measurement run by the coordinated rule (ADR-0011; CONTRIBUTING's
  publishing rule)
- **M18 — the announced delta (ADR-0024)**: the a11y announcement seam
  joins the invalidation currency. The app's voice — state changes that
  own no widget ("Saved", "3 items left", "Connection lost") — ships as
  a reserved live-region node in pass 9's snapshot: the same snapshot,
  the same `A11yDelta`, the same `Change::A11Y` currency as every other
  a11y fact (no second channel). The region is present from the first
  frame (the root's last child, named empty), so announcing is one
  node-only delta — no `STRUCTURE` churn, no ghost removals; the
  politeness rides the node's `aria-live` priority and the sink emits
  the ARIA pairing (`status` speaks politely, `alert` interrupts, the
  `accesskit::Live` property a screen reader watches). Two doors, one
  destination: `PassPipeline::announce` for assembly sites and tests,
  and the handler-reachable `alacer_core::announce::announce` queue
  (the clipboard-seam shape) drained at the next tick's start — the
  input queue's shape, one frame later, last write wins, a refused act
  (empty text, no root) honest with no masks. The consumer
  demonstration ships: `examples/announcements.rs` (polite, assertive,
  and the honest refusal, one `tracing` span per route at the
  `alacer::announce` target), pinned by the m1 corpus's new
  `announcements` scenario (the corpus grows 32 → 33 scenarios, 34 →
  35 PNGs), and the announcement route joins the bench corpus by the
  coordinated rule — `announce/route_100_announcements_1k`, 53 → 54
  cases, the baseline published with its same-host regression contract
  and every M-suite pin moved together. The error-chain discipline
  completes: `PlatformError::Softbuffer` and `BackendError::
  SurfaceCreation` carry their structured causes
  (`softbuffer::SoftBufferError`, `wgpu::CreateSurfaceError`) instead
  of flattened strings — no public error type in the workspace
  discards a cause into prose now. The dead convenience APIs go:
  `PassPipeline::has_queued_input`, `run_hot_window_with_channel`, and
  the `HotView`/`ViewRoot` `registry()` accessors had zero call sites.
  The living documents re-agree: the architecture's ADR count (now 25
  with the register's new row), the migration guides' snapshot counts
  (33 styled scenarios / 35 PNGs), the journal's materialization-shape
  rustdoc (the bench consumes `SceneBatches`, the backends walk the
  snapshot), the core pass-order rustdoc (the ten-pass contract), and
  the README's `counter_view` attribution (the example reconciles in
  place; it carries no keys)
- **M17 — the demonstrated queue (ADR-0023)**: the continuation of the
  consumable-surface work. The M8 coalescer — the one core vocabulary
  with a published budget and no runnable demonstration — gets its
  consumer demonstration: `examples/input_storm.rs` drives the
  frame-batched input queue from the consumer side, a custom
  `StormPanel` probe (a `Widget` with its required role and label)
  records every routed pointer event, *Read the log* mirrors the log
  into the window through the one-batch route, and the observability
  rides the workspace's idiom (`tracing` spans at the `alacer::input`
  target, the routed count as a field); the deterministic fifty-move
  storm is pinned by the m1 corpus's new `input_storm` scenario (the
  corpus grows 31 → 32 scenarios, 33 → 34 PNGs; the bench corpus stays
  53, the storm's hot path is already contracted). The living documents
  re-agree with the checked-in data: the README now quotes the
  published baseline's ratios — 14.06× for the W1 incremental-vs-full
  gate and 40.7× for the input-storm batching budget, which the
  M13–M15 corpus re-publishes had moved while the prose kept the
  M9-era 14.76× and 40.9× — the migration guides map onto Alacer 0.12.0
  instead of 0.5, and the facade's rustdoc describes the shipped
  surface instead of the M0 dependency spine. The error-chain
  discipline reaches the two seams M16 did not: `Recording::save` and
  `Recording::load` hand their `serde_json` cause to the
  `io::Error` chain (a malformed `.alc` — a bug report, by ADR-10's
  definition — now walks to the real parse error; the schema-mismatch
  leaf keeps its honest both-schemas message), and `LayoutError` — the
  public `LayoutEngine` contract's never-constructed error type —
  carries the failing node as data (`node: Option<NodeId>`) instead of
  the doc's promise that widget ids live in prose
- **M16 — the consumable contract (ADR-0022)**: the consumer surface the
  editing series (M11–M15) grew faster than it could be consumed. The
  0.12.0 cut completes: the twelve internal dependency pins in
  `[workspace.dependencies]` and the lockfile ride
  `[workspace.package]`'s version (the version-bump commit had left them
  at 0.11.0, and `cargo` refused to resolve the half-bumped workspace),
  and the README's badge follows the workspace. The error boundary
  carries its chain: `PlatformError` and `BackendError` implement
  `Error::source` (the `EventLoopError`, `OsError`, `RequestDeviceError`,
  and `wgpu::Error` inside them stay reachable instead of being
  formatted away), and the no-adapter message names the escape hatch it
  knows (`ALACER_SOFTWARE=1` for the softbuffer path). The widgets
  crate's manifest description and module rustdoc describe the shipped
  vocabulary (label, button, text input, checkbox, column, row, spacer,
  list, panel, table, and the M6 canons) instead of the M0 set. The M15
  journal gets its consumer demonstration: `examples/undo_redo.rs`
  drives `EditHistory` directly — the pure-append coalescing rule, the
  selection each step returns, the app-write reset, the honest refusal,
  `undo_depth`/`can_redo` live in the window, and one `tracing` span per
  route at the `alacer::editing` target — and a blessed snapshot pins
  the demonstration (the m1 corpus grows 32 → 33 PNGs, the bench corpus
  stays 53; `tracing` joins the facade as a dev-dependency, so nothing
  published moves)
- **M15 — the undo stack joins the editing contract (ADR-0021)**: the
  value — scalar-addressable since M1, selection-managed since M11,
  pointer-edited since M12, word-edited since M13, clipboard-edited
  since M14 — is reversible. The `EditHistory` journal joins the editing
  rules beside `paste_text` and `word_hop` (`alacer-core/history.rs`):
  pure UTF-16 text arithmetic, no widget dialect — each step carries the
  replaced span, the removed and inserted text, and the `(anchor, caret)`
  selection on both sides, so an undo restores not just the text but the
  range the user was extending when they made the edit. The coalescing
  rule is the pure-append rule (a typed run undoes as one step; a
  deletion, a word chord, a cut, a paste, an IME commit are their own),
  a whole-value replacement — the view's reconciliation, the AT's
  `SetValue` — resets the journal (the app's write is never undone), and
  the journal is bounded at 100 steps with the oldest evicted FIFO (a
  cache with an eviction policy and tests that prove it, §14.3). The
  editor routes every committed edit through one journaling funnel and
  answers Ctrl/Cmd+Z, Shift+Ctrl/Cmd+Z, and the Ctrl/Cmd+Y alternate
  with the M11 selection rules: an exhausted or reset journal is
  consumed with no masks — an idle frame — and the chords are inert
  while a preedit composes, the a11y delta riding unchanged
- **M15 — the harness and the corpus**: the corpus grows 52 → 53 by the
  coordinated rule — the new `editing/route_100_undo_redo_1k` criterion
  bench (100 undo/redo chords routed through the pipeline on a
  1,000-node arena, measured on the shipped `TextInput`) is published
  with a same-host regression gate, and the M5–M14 pin suites move
  together with the new M15 exit suite
- **M14 — the clipboard joins the editing contract (ADR-0020)**: the
  selection — scalar-addressable since M1, keyboard-extended and
  AT-managed since M11, pointer-placed since M12, word-addressable since
  M13 — is transferable. The clipboard seam joins the assembly seams
  (`ClipboardSeam`: `read_text`/`write_text`, installed once per thread
  like the text seam) with no provided implementation on principle: the
  OS clipboard is a platform fact, and a widget-internal buffer would
  silently break the round trip every other application shares — the
  engine rides arboard (the adopted core the architecture already
  ratified), owned by `alacer-platform` per the adapter-dependency rule,
  opened lazily so a headless session answers reads with `None` and
  refuses writes instead of panicking. The permission/escape story is
  the seam's contract: a refused write is never munged into success, so
  a cut whose write was refused does not delete, and a refused chord —
  a copy without a selection, a paste from an empty clipboard — is
  consumed but costs an idle frame. Core fixes the single-line paste
  rule (`paste_text`: every line break — CR, LF, or a CRLF pair counted
  once — pastes as exactly one space; one definition, no widget
  dialect). The editor consumes it: Ctrl/Cmd+C copies the selection's
  exact UTF-16 span and edits nothing, Ctrl/Cmd+X moves it out, Ctrl/
  Cmd+V inserts the sanitized text consuming an active selection first —
  an edit like any other, riding the same `PAINT | A11Y` currency, inert
  while a preedit composes — and the a11y tree rides unchanged
- **M14 — the harness and the corpus**: `TestHarness::set_clipboard` /
  `clipboard_text` drive the seam the tests' in-memory store answers —
  a headless CI never races the operator's actual OS clipboard; the
  corpus grows 51 → 52 by the coordinated rule — the new
  `editing/route_100_clipboard_ops_1k` criterion bench (100
  select-copy-paste round trips routed through the pipeline on a
  1,000-node arena, measured on the shipped `TextInput`) is published
  with a same-host regression gate, and the M5–M13 pin suites move
  together with the new M14 exit suite
- **M13 — the word joins the editing contract (ADR-0019)**: the caret —
  scalar-addressable since M1, keyboard-extended since M11, pointer-placed
  since M12 — is word-addressable. The text seam grows the required
  `word_segments` — the run's UAX #29 word segmentation in the §10.3
  UTF-16 contract unit, with no provided implementation on principle: a
  whitespace-and-ASCII approximation would silently mis-hop `don't` and
  every multi-script run, so the contract requires the engine's tables
  (ICU's `WordSegmenter`, the same segmenter parley's line breaking
  adopts, dictionary-backed for Thai and friends). Core fixes the hop
  rule (`word_hop`) — a hop lands on the far edge of the word-like
  segment that spans the caret, or on the near edge of the next one;
  separators are never landings, so `", "` takes one hop; the edges
  clamp. The editor consumes it: Alt/Ctrl/Meta+Left/Right hop word-wise
  (Shift extends from the anchor, a plain press collapses a live
  selection first — the GTK rules the arrows already follow),
  Ctrl/Alt+Backspace and Delete delete exactly the span the caret would
  hop (a selection is consumed whole first, the M11 edit rule), and a
  double-click selects the word under the hit — anchor at its start,
  caret at its end — collapsing on a separator rather than wandering to
  the nearest word. The event contract carries the gesture fact the
  widget cannot time: `PointerEvent::Down` grows `click_count`, computed
  by the platform's click chain (400 ms, 4 px — GTK's defaults), and the
  a11y tree rides unchanged — a word selection is the same anchor+caret
  vocabulary every writer shares
- **M13 — the harness and the corpus**: `TestHarness::double_click`
  drives the gesture the way the platform's click chain reports it; the
  corpus grows 50 → 51 by the coordinated rule — the new
  `editing/route_100_word_moves_1k` criterion bench (100
  direction-alternating word hops routed through the pipeline on a
  1,000-node arena, measured on the shipped `TextInput`) is published
  with a same-host regression gate, and the M5–M12 pin suites move
  together with the new M13 exit suite
- **M12 — the pointer joins the editing contract (ADR-0018)**: the
  caret — until now keyboard- and AT-addressable only — is
  pointer-addressable. A click inside the field places the caret at the
  hit point (edges clamp, ties break to the cluster's start), a
  press-drag extends the selection from the M11 anchor — the drag is the
  anchor state's second writer, keyboard and pointer alike — and a
  release ends the gesture while the range survives as a state; an
  unpressed move is hover and never moves the caret; the pre-M12
  press-collapses-at-the-stale-caret fallback is deleted. The seam that
  makes it honest is two-sided: the pass-1 router hands every pointer
  handler the node's resolved layout frame (`EventCtx::bounds()`, fed
  from the arena before the capture walk), and the text seam grows the
  bounds-aware offset mapping (`TextSeam::hit_test`) — the caret and the
  highlight's own prefix widths, inverted, so a click maps onto what is
  drawn; during a composition the committed text is what offsets map to
  (the preedit is overlay spliced at the caret, not a click target)
- **M12 — the harness and the corpus**: `TestHarness::pointer_press` /
  `pointer_move` / `pointer_release` / `drag` drive the gesture the way
  a user's hand and the recordings deliver it; the corpus grows 49 → 50
  by the coordinated rule — the new `editing/route_100_caret_clicks_1k`
  criterion bench (100 queued caret placements routed through the
  pipeline on a 1,000-node arena, measured on the shipped `TextInput`)
  is published with a same-host regression gate, and the
  M5/M6/M7/M8/M9/M10/M11 pin suites move together with the new M12 exit
  suite
- **M11 — the UTF-16 selection model joins the editing and a11y contracts (ADR-0017)**:
  the text input grows the anchor+caret selection in the §10.3 contract
  unit — Shift+arrows/Home/End extend from the anchor, Ctrl/Cmd+A selects
  all, a plain arrow collapses to the near edge first (GTK semantics), and
  every edit consumes the selection: typing replaces it, Backspace/Delete
  clear the whole range, a click collapses it, and an IME commit is an
  insertion like any other; the highlight paints from the prefix widths
  the text seam already measures; the focused input exposes the range as
  tree vocabulary (`Widget::selection()` → `TreeNode.selection` →
  AccessKit `TextSelection`), dropping the report while a preedit composes;
  and the AT value tier joins the faithful mapping — `SetValue`,
  `ReplaceSelectedText`, and `SetTextSelection` carry their payload or are
  dropped at the boundary, a refused value action costs an idle frame (a
  button refuses the value tier, an input refuses the press tier), and the
  pre-M11 bug that typed an `a` on Ctrl+A is deleted
- **M11 — the harness and the corpus**: `TestHarness::selection(role,
  name)` reads the range exactly as the AT sees it (normalized UTF-16
  offsets from the a11y snapshot); the corpus grows 48 → 49 by the
  coordinated rule — the new `a11y_action/route_100_setvalue_1k` criterion
  bench (100 payload-carrying `SetValue` actions routed through pass 10 on
  a 1,000-node arena, worst-case target) is published with a same-host
  regression gate, and the M5/M6/M7/M8/M9/M10 pin suites move together with
  the new M11 exit suite
- **M10 — the a11y action tier joins the dispatch contract (ADR-0016)**:
  the platform maps every AccessKit `ActionRequest` faithfully instead of
  forcing it to `DefaultPress` — `Click`, `Increment`, `Decrement`, and
  `ScrollUp`/`ScrollDown` (with the request's `ScrollUnit`) route through
  pass 10, and an action a widget does not implement is refused with no
  masks and no effects, never munged into a press; the `List` answers the
  scroll/step vocabulary translation-only (an item is the 40 px line, a
  page is the viewport, `Increment`/`Decrement` step by a page, clamped
  like the wheel path), and a focused `Button` is keyboard-operable —
  Space/Enter fire one click per press-and-release (GTK release
  semantics, auto-repeat absorbed)
- **M10 — the harness and the corpus**: `TestHarness::act(role, name,
  action)` drives the full action tier through the a11y query API and
  returns the consumption verdict (refusals are assertable); the corpus
  grows 47 → 48 by the coordinated rule — the new
  `a11y_action/route_100_actions_1k` criterion bench (100 mixed actions
  routed through pass 10 on a 1,000-node arena) is published with a
  same-host regression gate, and the M5/M6/M7/M8/M9 pin suites move
  together with the new M10 exit suite
- **M10 — housekeeping**: the CI perf-contracts bench invocation pins
  `--benches` so criterion-only flags never reach a libtest harness; the
  `arena_masks/10000` baseline entry was re-published after the same-host
  checker flagged ~30% host drift that reproduces identically at the M9
  tree (measured by stashing the working tree)
- **M9 — the windowed a11y path joins the incremental contract (ADR-0015)**:
  the pipeline's own `AccessKitSink` becomes the only `TreeUpdate`
  materialization path — the real window drains the sink's dirty-only
  deltas at every frame boundary and answers an adapter-activation request
  (`InitialTreeRequested`) with the sink's own full-tree build, so the
  pre-M9 per-frame full rebuild (and its dropped M6 vocabulary: values,
  toggle state, grid slots) is gone; toggle state and table geometry now
  reach the OS adapter, and the sink's produced-update buffer is ring-capped
  instead of growing without bound in windowed runs
- **M9 — the a11y contract as a number**: the criterion pair
  `a11y/full_rebuild_1k` (the pre-M9 shape) vs
  `a11y/incremental_1k_dirty_10pct` (the M9 shape), published with two
  regression contracts and the `a11y_incremental_speedup` budget (full ÷
  incremental ≥ 2.0×); the corpus grows 45 → 47 by the coordinated rule,
  with the M5/M6/M7/M8 pin suites moving together and the new M9 suite
  asserting all four
- **M9 — the exit suite** (`alacer-testing/tests/m9_exit_criteria.rs`):
  the first frame drains the init tree and quiet frames drain nothing; a
  toggle reaches the window as a dirty-only update carrying its `Toggled`
  state; text values ride the delta; removals surface through the sink's
  removal record and the resolved tree shrinks; the activation response
  equals a fresh sink's init build byte for byte; the ratio is a published
  budget
- **M8 — frame-batched input (ADR-0014)**: the event coalescer moves into
  `alacer-core` (`alacer_core::input::EventCoalescer`) and becomes a field
  of the `PassPipeline` — the platform *queues* pointer events and never
  drains; `tick` drains the coalesced batch at its start, so a storm of
  fifty moves costs one coalesced routing, one event pass, one frame (the
  same one-frame currency signal writes already obey, ADR-2/ADR-6)
- **M8 — the batching contract as a number**: the criterion pair
  `input/storm_50_moves_queued` (7.32 µs — 50 events, one tick) vs
  `input/storm_50_moves_direct` (299.21 µs — the pre-M8 shape), published
  with two regression contracts and the `input_storm_batching` budget
  (direct ÷ queued ≥ 2.0×, measured 40.9×); the corpus grows 43 → 45 by
  the coordinated rule, with the M5/M6/M7 pin suites moving together and
  the new M8 suite asserting all three
- **M8 — the exit suite** (`alacer-testing/tests/m8_exit_criteria.rs`): a
  fifty-move storm routes exactly one coalesced event (probe widget, event
  counter, and frame ledger agree); a ten-tick wheel burst lands as one
  accumulated delta; a direction flip splits the run; horizontal wheel
  deltas reach widgets; the list scrolls through the queue with zero
  measure calls; the platform source pins push-don't-drain
- **M8 — the cast seam completes**: `f64_to_f32`, `f32_to_usize`, and the
  `usize_to_u8`/`usize_to_u16` rounding helpers join `alacer_core::num`,
  the workspace's single documented rounding/casting point
- **M8 — the harness can storm too**: `TestHarness::queue_pointer` exposes
  the frame-batched input queue to headless suites

### Changed

- **The workspace lint regime now actually applies.** The
  `[workspace.lints]` table (missing_docs deny, unsafe_code forbid, clippy
  pedantic) was never inherited — no member crate opted in — so the
  "pedantic denied workspace-wide, zero allow-list" claim was inert. Every
  member crate now carries `[lints] workspace = true`, pedantic fires
  (`warn` + the CI `-D warnings`), and the ~90 latent pedantic violations
  the activation surfaced are fixed or carried by reasoned `#[expect]`s —
  still zero `#[allow]` anywhere
- The platform's wheel mapping carries both axes (the pre-M8 mapping
  dropped `x`); `MouseScrollDelta` → pixel vector is a pure, unit-tested
  mapping (`wheel_delta`)
- The platform's per-callback drain block and its redundant double
  `take_pending_masks` are deleted — input routing decisions live in the
  pipeline, the present decision in the damage ladder
- The baselines are re-published on the same host class (fingerprint
  `ee37c3efc5995352`, dirty tree): the W1 budget re-measures at 14.76×;
  the committed cross-framework comparison is untouched (ADR-0013's rig of
  record)
- `docs/architecture.md` §18 gains the M6/M7 roadmap rows (the README
  claimed them; the table had drifted) and the M8 row; §10.2 notes the
  delivered wiring
- **M9 — the a11y seam drops the split-brain**: `alacer-platform` deletes
  its snapshot-rebuilding `TreeUpdateBuilder` and the `tree_dirty` flag;
  `alacer-core` gains `A11ySink::as_any_mut` (the `Widget::as_any_mut`
  pattern — the trait stays adapter-free) and `PassPipeline::a11y_sink_mut`;
  `alacer-a11y` factors one `build_update` behind `drain_updates` and the
  pure `full_tree_update`; the producer-less `PlatformEvent::TreeChanged`
  variant becomes the embedder's flush nudge it was named for
- The baselines are re-published for M9 (corpus 47, one collecting host):
  the `a11y_incremental_speedup` ratio joins the two existing ratio budgets
  as the host-independent gates
- `docs/architecture.md` gains the M9 roadmap row (ADR-0015) and the
  ADR-0012…0015 register rows; §4.2's phantom `alacer-hot`/`alacer-devtools`
  rows become the real `alacer-macro`/`alacer-inspect` and the facade row
  names the hot-reload runtime it actually hosts

### Fixed

- **`ALACER_SOFTWARE=1` is the platform's own**: the env the README's
  prerequisites, quickstart, and tip promised honored itself at startup —
  `AlacerApp::new` runs the caller's preference through the pure
  `BackendPreference::software_env` parser (the env is passed, never read,
  so tests drive every shape without mutating process state), a `Wgpu`
  preference the env overrides presents through softbuffer, a
  `Softbuffer` preference is already the env's answer, and a value other
  than `1` is refused honestly — a `tracing::warn!` on
  `alacer::platform` naming the value it saw and the one it accepts,
  never a guess (the M11 rule at the platform seam). Until now only the
  examples' per-example `choose_backend` helpers read the env, so a
  consumer app had to re-implement the switch the docs described as the
  platform's.

## [0.17.0] — 2026-11-10



### Added


- The walk's answer becomes data


## [0.16.0] — 2026-11-10



### Added


- The focus walk bounds to a subtree

- The focus trap drives the consumer's window

- The directional walk reads the window

- The directional walk drives the consumer's window


### Changed


- The dead fixed-size spec goes

- The dead color conversion goes

- The dead selection predicate goes

- The dead color conversion goes


### Documentation


- The living documents re-agree at M25 and 0.16.0

- The living documents re-agree at M26 and 0.16.0


### Fixed


- The m7 release-state pin re-joins the chain


### Performance


- The focus routes join the published corpus


### Tests


- The m25 exit suite pins the focus scope

- The m26 exit suite pins the directional walk

- The m-suite pins move with the 60-case corpus


## [0.15.0] — 2026-11-10



### Added


- The focus walk joins the dispatch contract

- The focus walk drives the consumer's window


### Documentation


- The living documents re-agree at M24


### Miscellaneous


- The dead effect counter goes

- The harness's dead probes go


### Tests


- The m24 exit suite pins the focus walk


## [0.14.0] — 2026-11-10



### Added


- The availability tier joins the widget contract

- The sink maps the disabled flag faithfully

- The actionable controls answer the availability tier

- The availability tier drives the consumer's window

- The journal reports its own ledger

- The frame's ledger drives the consumer's window


### Changed


- The root duplicate goes

- The dead glyph counter goes


### Documentation


- The living documents re-agree at M22

- The living documents re-agree at M23 and 0.14.0


### Fixed


- The fall-through diagnostic names the token

- The body-cardinality diagnostic names its cardinality

- The accept loop reports the session errors it swallowed


### Miscellaneous


- The availability case joins the blessed corpus

- The manifest describes the shipped vocabulary

- The handoff manifest rides the gitignore


### Performance


- The refused route joins the published corpus


### Tests


- The m22 exit suite pins the availability tier

- The m-suite pins move with the 57-case corpus

- The m23 exit suite pins the ledger and the scroll trio


## [0.7.0] — 2026-11-08

### Added

- **M7 — the reproducible benchmark rig (ADR-0013)**:
  `scripts/collect_rig.py` writes `benchmarks/rig.json` — the host identity
  (the same sha256-truncated fingerprint basis the baseline uses, so a rig
  and a baseline cross-reference), the toolchain, the measurement protocol
  (5 process runs per case, median-of-medians, criterion sample size and
  window), the backend mode (headless CPU; vsync labeled
  not-applicable-headless rather than omitted), and the pinned dependency
  versions of every measured framework, read from the lockfiles
- **M7 — the competitor harnesses** (`benchmarks/competitors/`): two
  standalone cargo projects — `egui_workloads` (egui 0.37, the named
  immediate-mode DX baseline, driven headlessly through `Context::run_ui` +
  `Context::tessellate`) and `masonry_workloads` (Masonry 0.4, the audit's
  retained pass-contract reference, driven through the public `RenderRoot`
  API with the accessibility tree enabled) — each implementing the §14.2
  beat workloads W1/W2/W3 idiomatically, with selfcheck tests pinning what
  a bench frame claims (the full 10k walk, the re-wrapping paragraphs, the
  1k paint ops) and a README carrying the no-strawman provenance table
  (the framework APIs used, and the virtualization answers — egui's
  `show_rows`, Masonry's `Portal` — deliberately not taken, documented)
- **M7 — the comparison runner** (`scripts/run_comparison.py`): builds the
  competitor projects, runs each case five times, reads the Alacer side
  from the published baseline (same-host by fingerprint, or re-measured
  with `--remeasure-alacer`), and publishes
  `benchmarks/comparisons/results.json` — schema 1, one labeled entry per
  (workload, framework) with median, run medians, scope note, and pinned
  version, plus the same-host beat verdict; `--validate` checks the
  committed artifacts cheaply (the mode the exit suite and CI call)
- **M7 — the first published comparison**: collected on the declared rig
  (fingerprint `ee37c3efc5995352`, same host as the published baseline):
  W1 — incremental relayout of the 10k-node churn frame — measures
  **1.10 ms against egui's 20.7 ms full immediate frame (18.8×) and
  Masonry's 4.6 ms full pass pipeline (4.2×): the beat holds**; W2 and W3
  are published as measured, including the rows where the competitors win
  (Masonry's 0.62 ms re-wrap pipeline beats Alacer's 1.00 ms incremental
  text relayout; both competitors beat Alacer's 23.5 ms W3 frame, whose row
  includes the damaged software raster the competitors' rows exclude —
  the scope labels say so); every row conservative against the thesis per
  ADR-0013 §3
- **M7 — the publication surface**: `benchmarks/contracts.json` grows the
  `comparison_corpus_published` test-enforced gate (enforced by the M7 exit
  suite); the perf dashboard grows the cross-framework comparison section —
  per-workload medians with scope labels, the beat badges, and the advisory
  note when the collecting rig differs from the baseline's
- **M7 — the exit suite** (`alacer-testing/tests/m7_exit_criteria.rs`): the
  rig is declared and referenced by the results; the competitor corpus
  covers {egui, masonry} × {W1, W2, W3} as standalone, pinned,
  no-strawman projects; the runner validates the committed artifacts; the
  beat is computable and same-host with every measurement scoped; the
  contract gate and the dashboard publish the comparison; the M5/M6 corpus
  pins do not move (43 workspace cases — M7 adds competitor data outside
  the corpus, never inside it); the ADR names the harness pieces
- ADR-0013: the M7 scope decision record

### Changed

- The workspace ships as 0.7.0
- The README status reflects M0–M7 and carries the measured beat in the
  features list; the perf-features bullet says 43 criterion cases (it had
  drifted back to 38)
- The `regressions` corpus stays at 43 gates: competitor absolute medians
  are deliberately not regression-gated (ADR-0011's same-host rule —
  absolute gates on silicon the project does not own are benchmark
  theatre)

## [0.6.0] — 2026-11-07

### Added

- **M6 — the widget canon completes the v1 definition of done
  (ADR-0012)**: the `Checkbox` (role `CheckBox`, click / Space / AccessKit
  default action all toggle, the checked state travels the a11y tree as the
  neutral `ToggleState` and lands as AccessKit's `Toggled` property) and the
  `Table` / `TableRow` / `TableCell` / `ColumnHeader` family (roles mapped
  onto AccessKit's `Table` / `Row` / `Cell` / `ColumnHeader`; the neutral
  `TableSlot` carries the grid geometry, so the AccessKit update publishes
  row/column counts and zero-based cell positions — a *navigable* grid, not
  just a pile of roles); `Key::Space` joins the key vocabulary (winit's
  `NamedKey::Space` maps at the platform seam) and a consumed a11y action
  now invalidates through the one currency (`PAINT | A11Y` on the target —
  A1)
- **M6 — the canon apps as shared builders** (`alacer-widgets::canon`):
  `todo_app` (the §5 shape — input submits on Enter via the new
  `TextInput::on_submit`, rows toggle and delete through the a11y tree, the
  footer binding counts what is left) and `data_table_app` (toolbar +
  "Add row" seam + positioned cells that renumber on removal); one
  implementation, two consumers — the `todo` and `data_table` examples
  assemble them behind a window, the M6 exit suite drives them headlessly
  through the a11y queries; `TextInput::on_submit` is the Enter-to-commit
  seam the canon needs
- **M6 — the view vocabulary grows the typed handler props**:
  `PropValue::TextHandler` (`on_submit`, `on_changed` — the event carries
  the committed text) and `PropValue::FlagHandler` (`on_toggle` — the event
  carries the widget's new state), with `ElementProps::text_handler` /
  `flag_handler` accessors; the default registry registers the canon
  elements (`checkbox`, `text_input`, `table`, `table_row`, `table_cell`,
  `column_header`)
- **M6 — the shared workloads W2/W3/W5 as measured, gated benches**:
  W2 text-dense relayout (`alacer-layout/benches/w2_text.rs` — 1k
  paragraphs under ±5% width jitter, incremental + full denominator), W3
  animation loop (`alacer-testing/benches/w3_animation.rs` — 1k nodes,
  translation advance + 10% paint churn through the *real* pipeline and the
  damaged software raster), W5 signal-graph churn
  (`alacer-core/benches/w5_signal_graph.rs` — 10k signals, 1k memos,
  batched writes → one rerun → mask drain); the corpus grows 38 → 43 (the
  M5 and M6 suites pin the same number), every new case carries its
  contracts.json regression gate, the baseline was republished by
  `scripts/collect_baseline.py`, and the dashboard regenerated
- **M6 — the integer wrap rounding, adopted as ALGO** (§8.3): the layout
  host floors text wrap widths to whole device pixels before they reach the
  measurer or the cache key — sub-pixel container jitter cannot flip a line
  break; the no-re-wrap-oscillation gate is test-enforced
  (`alacer-layout/tests/w2_gate.rs`: integrality, sub-pixel stability, and
  cache-hit-on-revisit) and published on the dashboard as
  `w2_no_rewrap_oscillation`
- **M6 — record/replay** (`alacer-testing::recording`, §13.4): the `.alc`
  format (versioned JSON; semantic steps — `press` by role + name, typed
  text, keys, wheel, click — so recordings survive layout drift), the
  `Session` recorder (record + execute in one pass), `Recording::replay` /
  `replay_with` (the per-step hook is where apps with explicit re-render
  seams plug in), and the committed canonical recording
  (`crates/alacer-testing/recordings/todo.alc`, re-blessed with
  `ALACER_RECORD=1 cargo test -p alacer-testing`) replayed green by the
  exit suite — bug reports that are also regression tests
- **M6 — the exit suite** (`alacer-testing/tests/m6_exit_criteria.rs`):
  the checkbox toggles through every input path with the state reaching the
  a11y snapshot; the data table is reachable, named, and positioned; the
  canon apps work end-to-end; the committed recording replays green; the
  workload corpus is published and contracted at 43; the W5 scheduling
  contract holds at graph scale (one write batch ⇒ one effect rerun, memos
  never glitch); the gallery exercises the canon widgets; the M5 pin agrees
- ADR-0012: the M6 scope decision record

### Changed

- The workspace ships as 0.6.0
- The M6 canon builders (`todo_app`, `data_table_app`) take a `WakeUp`
  trigger (the patch-channel fire) so windowed apps re-render with the same
  one-frame contract the exit suite drives headlessly; headless callers
  pass `None` and call `rerender` explicitly
- The M5 corpus pin moves 38 → 43 and the "five benching crates" language
  becomes six (the harness crate now benches W3)
- The perf-contract corpus on the dashboard: 43 baselines, 6 test-enforced
  gates (the W2 no-oscillation gate and the W5 one-rerun gate join the four
  M5 gates)
- The gallery exercises the canon widgets (checkbox + data table tour
  sections)
- The examples catalog lists the canon examples with their contracts

## [0.5.0] — 2026-11-06

### Added

- **M5 — criterion baselines published as contracts (ADR-0011)**: all 38
  criterion benchmark cases across the five benching crates distilled into
  `benchmarks/baselines/baselines.json` by `scripts/collect_baseline.py` —
  median/mean/MAD/sample count per case plus the environment that produced
  them (git commit, dirty flag, toolchain, host fingerprint); the
  architecture's falsifiable gates published as measured **budget
  contracts** (W1 ≥ 2× incremental relayout, measured 14.01×) and one
  **same-host regression gate** per case (fresh median ≤ published × 1.25
  on a matching host fingerprint; foreign hosts skip by policy,
  `--strict-host` for pinned hardware); the uncontracted-benchmark rule
  fails any bench case without a published gate
- **M5 — the perf-contract checker** (`scripts/check_perf_contracts.py`):
  enforces the budgets everywhere (ratios are host-independent), the
  same-host regressions on matching fingerprints, and the contracts ↔
  baseline bijection; exit codes for CI
- **M5 — the perf dashboard** (`scripts/perf_dashboard.py` →
  `benchmarks/dashboard/index.html`): one self-contained HTML file — the
  measured environment, budget contracts with ratios and PASS/FAIL, the
  full baseline table with delta bars, and the four test-enforced gates
  (pass overhead < 15%, TreeUpdate p99 < 0.5 ms, the patch frame budget,
  zero-measure scrolling) each naming its enforcement site; CI
  regenerates and uploads it on main
- **M5 — the examples gallery**: `crates/alacer/examples/gallery.rs` — the
  whole core widget vocabulary (labels, buttons, text input, panels,
  scrolling list, rows, spacers, the shared theme) in one window through
  the canonical write seam (signals in, masks out, one batch one frame);
  `docs/examples.md` catalogs every example with its contract and the
  bless workflow
- **M5 — the egui/Iced migration guides** (`docs/migration/from-egui.md`,
  `from-iced.md`): the two audited reference frameworks mapped
  section-for-section (mental model, app shell, widget mapping,
  reactivity, layout, input and events, accessibility, testing, interop
  limits) with real API on both sides and the version-scope honesty note
- ADR-0011: the M5 scope decision record
- The M5 exit suite (`alacer-testing/tests/m5_exit_criteria.rs`): the
  baseline schema + environment header, the contracts ↔ baseline bijection
  at the pinned corpus size (38), the budgets holding on the published
  data (including the architecture's own 2× W1 value), dashboard
  coverage, catalog coverage, the gallery exercising the widget
  vocabulary through `Flux::batch_of` + `RenderBinding`, and both guides
  carrying the required section set

### Changed

- The workspace ships as 0.5.0
- CI gains the `perf-contracts` job: benches run at reduced sampling, the
  budget contracts are enforced against the published baseline, and the
  dashboard is regenerated and uploaded as an artifact
- `alacer-testing` gains a `serde_json` dev-dependency (the M5 exit suite
  reads the published contracts and baseline)

## [0.4.0] — 2026-11-06

### Added

- **M3 — the a11y contract**: the a11y pass produces `A11yDelta`s (updated /
  removed / focus / initializes); AccessKit `TreeUpdate`s stay dirty-node-only,
  removals surface exactly once, focus travels in the update, editables expose
  `value()`; the §18 gate — incremental TreeUpdate p99 < 0.5 ms @ 1k nodes —
  is test-enforced
- **M3 — partial paint**: the damage model (`alacer-core::damage`, ≤3 regions,
  least-growth merging, NONE/partial/FULL ladder) computed by the paint pass
  from old journal extents + new primitive extents, carried in
  `FrameReport::damage`; software backend repaints regions onto a persistent
  framebuffer, wgpu backend `LoadOp::Load`s an offscreen frame with scissor
  rects; the equivalence fuzz pins partial == full pixel-exactly (Q8)
- **M3 — the IME contract**: UTF-16-native key/IME event vocabulary, the
  `utf16` arithmetic seam (mid-surrogate offsets floor, never slice),
  focus-routed `dispatch_key`/`dispatch_ime` with Tab cycling, per-frame
  `ImeRequest` (purpose + caret anchor), the winit byte→UTF-16 platform seam,
  and the `TextInput` reference widget (caret editing, inline preedit, commit,
  secure mode, focus ring, a11y value)
- `TextInput` widget + the `text_input` example
- ADR-0009: the M3 scope decision record
- **M4 — the hot-reload contract (§13.2, ADR-7)**: `alacer-view::HotView`,
  the view-generation runtime — a patch swaps the generation, the
  reconciler re-renders scoped (patch-in-place, keyed LIS, exact
  differences), and state survives by construction; the facade's
  `hot-reload` feature links `subsecond`, wraps generations in
  `subsecond::call` (the jump-table detour point) and registers the patch
  handler; the platform's `PatchChannel` wakes the winit loop from any
  thread and answers one patch event with exactly one scoped re-render
  (`PlatformEvent::Patch` + `AlacerApp::set_patch_hook`)
- **M4 — the CI hot-patch suite**: `alacer-testing/tests/m4_exit_criteria.rs`
  pins the framework's half of the contract headlessly — a patched
  generation renders its new values with the old state (signal, binding,
  node identity), a patch costs exactly one layout+paint cycle and leaves
  the tree idle (A7), ten-generation churn with interleaved mutations stays
  consistent, untouched subtrees stay out of the patch frame, and the
  unobserved path stays silent (capture off, zero recording); the facade's
  `hot_reload_subsecond` tests pin the subsecond passthrough headless, and
  the `hot_reload` example is the sub-second demo (`dx serve --hotpatch`
  upgrades it to real binary patching, no code change)
- **M4 — the Causality inspector (§13.3)**: the `alacer-core::inspect` seam
  (`InspectEvent`: edge activations, provenance writes, routed inputs,
  frame summaries with the pass ledger + the dirty-node mask heatmap) and
  the `alacer-inspect` crate — `CausalRecorder` over bounded ring buffers
  (every ring's eviction is test-proven, §14.3), a dependency-light RFC
  6455 WebSocket server (`sha1` + `base64`, no async runtime), the
  documented WS protocol (`docs/inspector-protocol.md`, version 1), and the
  embedded single-file HTML client; the five §13.3 questions — why did
  this render, who wrote this signal, who subscribes, why is this frame
  slow, what did this mask touch — answered from recorded truth;
  `Flux::write` / `FluxWriter` is the opt-in write-provenance path;
  `Flux::set_causal_capture` gates activation capture; the protocol suite
  (`alacer-inspect/tests/protocol.rs`) pins the handshake (the RFC 6455
  reference vector), the greeting shape, streaming order, and the controls;
  the `inspector_demo` example is the live demo
- ADR-0010: the M4 scope decision record

### Changed

- `A11ySink::apply` takes a shared `Arc<TreeSnapshot>` (built once per a11y
  run — no per-frame deep clone) and an `&A11yDelta`
- `SceneJournal` gains `layer(node)` — the damage pass reads the layer it is
  about to replace
- Pass 3 notifies focus endpoints (`Widget::set_focused`) and merges their
  PAINT masks same-tick; the tick drains all masks pass 3 merged
- `Widget` gains `on_key_event`, `on_ime_event`, `set_focused`, `ime_purpose`,
  `value` (all defaulted)
- Buttons take keyboard focus on press (click-to-focus, §10.3)
- The workspace is 12 crates (ADR-9's target): `alacer-inspect` joins as
  the devtools layer (`alacer-core` only; `scripts/check_dag.py` enforces
  the order; `alacer-testing` may consume it for the M4 suite)
- `PassPipeline` gains `set_observer` / `clear_observer` — the devtools
  sink; `None` in every default path (the A7 idle contract and the M1
  overhead gate are unchanged and re-verified)
- The workspace ships as 0.4.0 with the full crates.io metadata set on
  every manifest

## [0.3.0] — 2026-11-05



### Added


- Add the Change mask invalidation currency

- Add the single cast seam

- Add shared euclid-backed geometry types

- Add the RGBA color type

- Add id-path prefix hashing

- Add the pointer event vocabulary

- Add typed layout styles and the measure seam

- Add the Widget trait with required role and label

- Add the retained widget arena

- Add the flux bridge over reactive_graph

- Add the scene primitive vocabulary and PaintCtx

- Add the pass contract and PassPipeline

- Wire the core crate public surface

- Add keyed reconciliation

- Add the Alacer-to-Taffy style conversion

- Add the LRU measure cache

- Add torin-style dirty-reason semantics

- Add the persistent Taffy LayoutHost

- Wire the layout crate public surface

- Add the frame-windowed layout cache

- Add the font registry

- Add the zeno glyph rasterizer

- Add the parley facade and thread-local context

- Add the node-layered scene journal

- Add the tiny-skia software rasterizer

- Wire the render crate public surface

- Add the AccessKit sink with incremental TreeUpdates

- Add the Panel decoration

- Add Column, Row and Spacer containers

- Add the Label widget

- Add the Button widget

- Add the scrolling List

- Wire the widget family, theme and BoxedWidget

- Add the event coalescer

- Add the softbuffer present path

- Add the wgpu renderer

- Add the AlacerApp winit driver

- Add the headless TestHarness

- Add the alacer facade crate

- Add u32_to_f32, u64_to_f32 and u128_to_u64 helpers

- Add the EventPhase capture/bubble enum

- Add Widget::focusable and the as_any_mut escape hatch

- Add the FluxRwSignal type alias

- Add the text seam (widgets shape text without alacer-text)

- Add reorder_children, is_live, and drain reporting

- Implement the ten-pass contract with the overhead gate

- Wire text_seam and re-export the M1 pass vocabulary

- Add the ThreadLocalTextSeam adapter

- Add the ViewNode tree and PropValue vocabulary

- Add the WidgetRegistry element table

- Add the ViewRoot in-place reconciler

- Wire the view tree, registry and reconciler modules

- Add the rsx! view DSL proc-macro

- Add in-place setters, as_any_mut and the text seam

- Add style_mut, as_any_mut and Spacer::set_size

- Add in-place setters, as_any_mut and the text seam

- Add style_mut and as_any_mut

- Add set_height, set_color and as_any_mut

- Register the default M0 element set

- Expose the default element registry

- Install the text seam at AlacerApp assembly

- Add the a11y query builder, press, and the text seam

- Expose rsx! and mount_view through the facade

- The root candidate, the bounded publish, and the sync diff

- The cached scene replay


### Benchmarks


- Add arena build/mask/walk benchmarks

- Add flux batch-coalescing and drain benchmarks

- Add reconciliation benchmarks

- Add the incremental vs full relayout benchmark

- Add cold/cached text layout benchmarks

- Add journal commit and replay benchmarks

- Add the §7.2 pass-overhead churn benchmark


### Build


- Scaffold the Alacer workspace

- Pin the initial dependency graph

- Cargo-deny license gate (no AGPL, no unknown licenses)

- Declare the alacer-core crate

- Declare the alacer-view crate

- Declare the alacer-layout crate

- Declare the alacer-text crate

- Declare the alacer-render crate

- Declare the alacer-a11y crate

- Declare the alacer-widgets crate

- Declare the alacer-platform crate

- Declare the alacer-testing crate

- Declare the alacer facade crate

- Register the pass_overhead benchmark target

- Declare the alacer-macro proc-macro crate

- Tighten to core + view (the M1 DAG close-out)

- Add alacer-macro and alacer-view deps

- Add alacer-macro to the facade s dependencies


### Continuous Integration


- Add the crate layering DAG checker

- Add the fmt/clippy/test/doc/deny/dag pipeline

- Add the git-cliff release automation

- Extend the layering for alacer-macro and the M1 tightening

- Ship the counter_view example binary


### Documentation


- Add the counter example

- Add the scrolling list example

- Add the git-cliff configuration

- Add the initial CHANGELOG

- Add the CONTRIBUTING guide

- Add the system architecture document

- Record the M0 scope decision

- Record the workspace discipline decision

- Record the scheduling contract decision

- Record the persistent-Taffy decision

- Record the software-first glyph-seam decision

- Record the testing productization decision

- Record the release automation decision

- Add the counter_view rsx! example

- Record the M1 scope decision

- Add the 0.2.0 release notes

- Update README for the M1 milestone

- Record the M2 scope decision

- The M2 milestone record and the 187-test count


### Fixed


- Treat dead arena slots as not-live during taffy sync

- Repair the broken snapshot corpus — glyph blend, y-convention, stretch, compose


### Miscellaneous


- Ignore build artifacts and editor/OS debris

- Pin the 1.99.1 toolchain with clippy and rustfmt

- Set rustfmt max_width to 100

- Add the Apache-2.0 license text

- Add the MIT license text

- Regenerate the lockfile for the 0.2.0 workspace

- Bless broken pngs

- Readme

- Logo & deny


### Tests


- Add the counter snapshot golden

- Add the scrolled-list snapshot golden

- Add the M0 exit-criteria integration tests

- Implement as_any_mut on the arena bench Node

- Implement as_any_mut on the incremental test Probe

- Implement as_any_mut on the incremental bench TestBox

- Add the M1 exit criteria and 30-scenario corpus

- Add the rsx! end-to-end integration suite

<!-- generated by git-cliff -->
