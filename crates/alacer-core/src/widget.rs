//! The `Widget` trait: accessibility is a type-level contract (ADR-5).
//!
//! `role` and `label` are **required** trait methods — a widget that cannot be
//! named cannot be compiled (Masonry's required-method design). The debug-mode
//! CI walker in `alacer-testing` fails builds on unnamed input controls; the
//! contract holds by construction, not by discipline.

use crate::events::{EventCtx, ImeEvent, KeyEvent, PointerEvent};
use crate::scene::PaintCtx;
use crate::style::Style;
use bitflags::bitflags;
use std::borrow::Cow;
use std::fmt;
use std::sync::{Arc, Mutex};

/// Semantic role of a widget, mapped onto AccessKit roles by `alacer-a11y`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// No meaningful semantic; used for pure decorations.
    Decoration,
    /// A generic grouping container.
    Container,
    /// Static text.
    Label,
    /// An actionable button.
    Button,
    /// A single-line text input.
    TextInput,
    /// A scrollable region.
    ScrollArea,
    /// One row of a list.
    ListItem,
    /// A collection of list items.
    List,
    /// A toggle control (the todo canon's item switch, M6): the binary
    /// pair plus, since M94 (ADR-0098), the tri-state summary — a
    /// checkbox holding `ToggleState::Mixed` is the select-all parent,
    /// "some of the children are"; every user door resolves it.
    CheckBox,
    /// A value control (the slider, M20): a bounded scalar the user sets
    /// by drag or arrow keys and the AT sets through the value tier
    /// (ADR-0026). The value itself rides the M3 `value()` contract.
    Slider,
    /// A read-only value report (the progress indicator, M21): a bounded
    /// scalar the *app* writes and the AT reads through the M3 `value()`
    /// contract — the value tier's read half (ADR-0027). The write
    /// vocabulary is the app's; every AT action is refused honestly.
    ProgressIndicator,
    /// A data table: headers, rows, cells (the data-table canon, M6).
    Table,
    /// One row of a data table.
    TableRow,
    /// One cell of a data table row.
    TableCell,
    /// A table's column header.
    ColumnHeader,
    /// A free-form custom-paint surface (the canvas, M82): the app's paint
    /// closure records the primitives through the paint seam, and the AT
    /// gets the name — a named canvas is a chart, a diagram, a drawing
    /// surface; an unnamed one is an unusable blank (the ADR-5 rule the
    /// named roles carry).
    Canvas,
    /// The app's reserved announcement region (§11.6, ADR-0024): a live
    /// region the pipeline owns — its content is the last announcement,
    /// and assistive technologies speak its changes. Never a widget's
    /// role; the pipeline builds the node itself.
    LiveRegion,
    /// A single-choice control in a set of one-of-N (the radio tier,
    /// M124, ADR-0128): the selection the M6 `toggled` contract
    /// announces — checked or off, never mixed (a radio has no summary;
    /// the tri-state parent is the checkbox's own word, ADR-0098). The
    /// user's doors *select* and never deselect — the redundancy of a
    /// second select is the M21 idle frame, the exclusivity the app's
    /// wiring writes through the handlers; the one deselect door is the
    /// programmatic write (view reconciliation), the checkbox's
    /// `set_checked` rule at the radio.
    Radio,
    /// The named container a set of radios reads as (the radio tier's
    /// container half, M124, ADR-0128): the group word the AT announces
    /// before it walks the members, the set tier's span word
    /// ([`Widget::size_of_set`]) its natural carrier. An unnamed group
    /// reads as a bare box — the ADR-5 rule the named roles carry.
    RadioGroup,
    /// A hyperlink control (the link tier, M125, ADR-0129): the
    /// "opens something" fact AccessKit's own link role spells. The
    /// activation verb is the link's own — the open act the app's
    /// navigation rides (the handler), the keyboard's key the platform
    /// convention names (Enter; Space stays the scroller's key the
    /// framework hands on), the AT's default action the open ask the
    /// door answers. The destination is the app's: the tree says
    /// "opens", never where — the name and the description carry the
    /// words the app declared.
    Link,
    /// A section title the document outline walks (the heading tier,
    /// M126, ADR-0130): the "names a section" fact AccessKit's own
    /// `Heading` role spells — not a bold label wearing bigger glyphs.
    /// The depth in the outline rides the level word the M113 tier
    /// published ([`Widget::level`]): the heading's natural carrier,
    /// never a second depth vocabulary. A heading names structure, it
    /// walls nothing — the section's content reads as it always did.
    Heading,
    /// One page of a tab strip (the tab tier, M127, ADR-0131): the
    /// "selects a page" fact AccessKit's own `Tab` role spells — not a
    /// button wearing the strip's clothes. The selection rides the
    /// selected word the tree already carries ([`Widget::selected`]):
    /// the user's doors select and never deselect, the exclusivity the
    /// app's wiring — the radio tier's own shape (ADR-0128) at the
    /// page tier.
    Tab,
    /// The named strip a set of tabs reads as (the tab tier's
    /// container half, M127, ADR-0131): the strip word the AT announces
    /// before it walks the pages. An unnamed strip reads as a bare box
    /// — the ADR-5 rule the named roles carry.
    TabList,
    /// A bounded number field the AT walks by arrows (the spin-button
    /// tier, M128, ADR-0132): the "writes one bounded number" fact
    /// AccessKit's own `SpinButton` role spells — not a slider wearing
    /// text (a slider's write is the *continuous drag*, a spin
    /// button's write is the *discrete step* the arrows speak) and not
    /// a bare text input carrying no value words. The value rides the
    /// M3 `value()` contract, the span the M97 range word, the number
    /// the M99 numeric word — the value tier's own vocabulary, never a
    /// second one.
    SpinButton,
    /// The named strip a set of tools reads as (the toolbar tier,
    /// M129, ADR-0133): the strip word the AT announces before it
    /// walks the tools — the tab-list/radio-group shape (the named
    /// container, ADR-0128's member-and-container discipline) at the
    /// tool tier. The tools are the members' own controls — buttons,
    /// inputs, the vocabulary the app mounts — and the strip is their
    /// layout (a horizontal run), never a second channel to them. An
    /// unnamed strip reads as a bare box — the ADR-5 rule the named
    /// roles carry.
    Toolbar,
    /// An instant on/off control (the switch tier, M130, ADR-0134):
    /// the "writes one on/off fact" AccessKit's own `Switch` role
    /// spells — not a checkbox wearing the wrong word (a checkbox's
    /// write resolves a *form contract*: checked, unchecked, the mixed
    /// summary the select-all parent needs; a switch's write is the
    /// instant on/off the settings vocabulary speaks, and a switch has
    /// no summary to hold). The binary pair rides the M6 `toggled`
    /// contract — `On` or `Off`, never `Mixed` (ADR-0098's rule at the
    /// switch tier: never a summary the control does not hold).
    Switch,
    /// The named pane the selected page's content lives in (the
    /// tab-panel tier, M131, ADR-0135): the "holds the selected page's
    /// content" fact AccessKit's own `TabPanel` role spells — not a
    /// bare box wearing no name (the set the AT cannot name is a set it
    /// cannot walk, the ADR-5 rule the named roles carry) and not a
    /// second tab list wearing the wrong word (a tab list *names the
    /// pages' strip*; a tab panel *names the page's content*). The
    /// member-and-container shape (ADR-0128) grown to its third word:
    /// the member ([`Role::Tab`]), the strip ([`Role::TabList`]), the
    /// pane ([`Role::TabPanel`]). The pane is the content's layout,
    /// never a second channel to it — the children keep the roles and
    /// labels they already carry, and the pane holds no focus and
    /// answers no activation (the M126/M129 container rule).
    TabPanel,
    /// A scalar measurement within a declared scale (the meter tier,
    /// M133, ADR-0137): the "reads as a gauge" fact AccessKit's own
    /// `Meter` role spells — not a progress bar wearing the wrong word
    /// (a progress bar's value is *task progress*, how far some task
    /// has come toward done; a meter's value is a *measurement* on a
    /// scale the app declares — how much of something there is, ARIA's
    /// `role="meter"` beside its `role="progressbar"`). The reading
    /// rides the value tier's own vocabulary — the M3 `value()` word,
    /// the M97 range word, the M99 numeric word — never a second value
    /// language. The gauge holds no focus and answers no activation
    /// (the M126 heading's own rule at the value tier's read half),
    /// and the AT tier is read-only by construction.
    Meter,
    /// The window the modal vocabulary speaks as "this is a dialog"
    /// (the dialog tier, M134, ADR-0138): the "reads as a modal
    /// window" fact AccessKit's own `Dialog` role spells — not a bare
    /// panel wearing no name (the set the AT cannot name is a set it
    /// cannot walk, the ADR-5 rule the named roles carry) and not a
    /// second tab panel wearing the wrong word (a tab panel names the
    /// *selected page's content*; a dialog names *the window the
    /// conversation lives in*). The container is the content's
    /// layout — the children keep the roles and labels they already
    /// carry — and the window holds no focus and answers no
    /// activation (the M126/M129 container rule). The modal
    /// *semantics* ride the focus scope the M25 tier published
    /// (ADR-0031's `focus_move_in`), never a second channel: the word
    /// names the window, the walk bounds it.
    Dialog,
    /// The named collection the AT walks by disclosure and depth (the
    /// tree tier's container half, M135, ADR-0139): the "reads as a
    /// hierarchical collection" fact AccessKit's own `Tree` role
    /// spells — not a table wearing the wrong geometry (a grid's
    /// geometry is rows × columns; a tree's is parentage) and not a
    /// bare list whose members cannot say what branches they stand
    /// in. The hierarchy *is* the arena's own nesting — the member
    /// word ([`Role::TreeItem`]) rides under the container exactly a
    /// row's cells ride under the table — and the container carries
    /// the collection words the tiers already published: the M119
    /// span word ([`Widget::size_of_set`]), the M109 capacity word
    /// ([`Widget::multiselectable`]), the M117 updating word
    /// ([`Widget::busy`]), the M120 declared region ([`Widget::live`]).
    /// An unnamed collection reads as a bare box — the ADR-5 rule the
    /// named roles carry.
    Tree,
    /// The named help bubble the pointer pair summons (the tooltip
    /// tier, M136, ADR-0140): the "reads as a hover-help bubble" fact
    /// AccessKit's own `Tooltip` role spells — not a description
    /// wearing a role (the M95 description tier is the help text
    /// *beside a name*, always on; a tooltip is *the bubble itself*,
    /// present exactly while the app mounts it) and not a dialog
    /// wearing one word (a dialog is a modal window the walk bounds
    /// itself into; a tooltip is a transient bubble the pointer pair
    /// summons). The bubble is the help's layout, never a second
    /// channel to it — the children ride the arena exactly a pane's
    /// do, each keeping the role and label it already carries — and
    /// the bubble holds no focus and answers no activation (the
    /// M126/M129 container rule, at the hover tier). An unnamed bubble
    /// reads as a bare box — the ADR-5 rule the named roles carry.
    Tooltip,
    /// One member of a hierarchical collection (the tree tier's member
    /// half, M135, ADR-0139): the "reads as a branch or a leaf" fact
    /// AccessKit's own `TreeItem` role spells — not a row wearing the
    /// wrong container (a table row stands in a grid; a tree item
    /// stands in parentage). The words the member says are the tiers
    /// the tree already published, never a second vocabulary: the M108
    /// pick ([`Widget::selected`]), the M110 disclosure
    /// ([`Widget::expanded`]), the M113 depth ([`Widget::level`]), the
    /// M119 place ([`Widget::position_in_set`]). The member's children
    /// ride the arena exactly a row's cells do — nested items under a
    /// branch, the app's own controls under either — and a tree item
    /// holds no focus and answers no activation (the M126/M129
    /// container rule, at the tree tier). An unnamed member reads as a
    /// bare mark — the ADR-5 rule the named roles carry.
    TreeItem,
    /// The named strip of menus a menu bar reads as (the menu tier's
    /// container half, M137, ADR-0141): the "reads as a set of menus"
    /// fact AccessKit's own `MenuBar` role spells — not a toolbar
    /// wearing the wrong word (a toolbar *offers tools*, a menu bar
    /// *names the menus the commands live in*) and not a bare row
    /// wearing no name — the set the AT cannot name is a set it cannot
    /// walk, the ADR-5 rule the named roles carry. The strip is the
    /// menus' layout (a horizontal run), never a second channel to
    /// them — the children ride the arena exactly a toolbar's do, each
    /// keeping the role and label it already carries — and the strip
    /// holds no focus and answers no activation (the M126/M129
    /// container rule, at the menu tier).
    MenuBar,
    /// The named list of commands a menu offers (the menu tier's
    /// summoned half, M137, ADR-0141): the "reads as a list of
    /// commands" fact AccessKit's own `Menu` role spells — not a dialog
    /// wearing the wrong transient (a dialog is a modal window the walk
    /// bounds itself into; a menu is a transient list the opener
    /// summons, present exactly while the app mounts it — the tooltip
    /// tier's presence rule, ADR-0140) and not a bare column wearing no
    /// name — the ADR-5 rule the named roles carry. The list is the
    /// entries' layout (a vertical run, the shape a menu is), never a
    /// second channel to them — the children ride the arena exactly a
    /// pane's do — and the list holds no focus and answers no
    /// activation (the M126/M129 container rule, at the menu tier).
    Menu,
    /// One command a menu offers (the menu tier's entry half, M137,
    /// ADR-0141): the "fires one command" fact AccessKit's own
    /// `MenuItem` role spells — not a button wearing the wrong container
    /// (a button stands wherever the app mounts it; a menu item stands
    /// in a named menu the AT walks by name) and not a bare label
    /// carrying no activation at all. The activation funnel is the
    /// button tier's own (the M10 parity): the click on release, Enter
    /// while focused, and the AT's default action the door answers —
    /// and the availability tier guards it (M22: an unavailable entry
    /// refuses, never munges). An unnamed entry reads as a bare mark —
    /// the ADR-5 rule the named roles carry.
    MenuItem,
    /// A named picture the app presents (the image tier, M138,
    /// ADR-0142): the "reads as a picture" fact AccessKit's own `Image`
    /// role spells — not a bare label wearing no picture word (the
    /// picture the AT cannot name is a picture it cannot offer on
    /// demand, the ADR-5 rule the named roles carry) and not a canvas
    /// wearing the wrong half (the canvas tier, M82, names a *drawing
    /// surface the app paints into*; a picture is *named content the
    /// app presents*). The pixel story is the app's own — the paint
    /// seam the canvas tier published, or the embedded raster the app
    /// owns; the word never paints. The alt text rides the M95
    /// description word beside the name, and the picture holds no focus
    /// and answers no activation (the M126/M129 container rule, at the
    /// picture tier).
    Image,
    /// The named strip the page opens with (the landmark tier, M143,
    /// ADR-0147): the "reads as the page's banner" fact AccessKit's own
    /// `Banner` role spells — not a bare panel wearing no container
    /// word (the region the AT cannot name is a region it cannot offer
    /// on demand, the ADR-5 rule the named roles carry) and not a
    /// toolbar wearing the wrong word (a toolbar *offers tools*; a
    /// banner *names the page's opening strip* — the site title, the
    /// logo, the masthead the AT's landmarks list walks first). The
    /// strip is the furniture's layout, never a second channel to it —
    /// the children ride the arena exactly a pane's do, each keeping
    /// the role and label it already carries — and the strip holds no
    /// focus and answers no activation (the M126/M129 container rule,
    /// at the landmark tier).
    Banner,
    /// The named strip the document's way through lives in (the
    /// landmark tier, M143, ADR-0147): the "reads as the page's
    /// navigation" fact AccessKit's own `Navigation` role spells — not
    /// a list wearing no container word (the way through the document
    /// the AT cannot name is a way it cannot offer on demand) and not a
    /// toolbar wearing the wrong word (a toolbar *offers tools*; a
    /// navigation strip *names the way* — the section links the AT's
    /// landmarks list walks beside the banner and the main). The strip
    /// is the links' layout, never a second channel to them — the
    /// children keep the roles and labels they already carry — and the
    /// strip holds no focus and answers no activation (the M126/M129
    /// container rule, at the landmark tier).
    Navigation,
    /// The named region the page's content lives in (the landmark tier,
    /// M143, ADR-0147): the "reads as the page's main region" fact
    /// AccessKit's own `Main` role spells — not a bare panel wearing no
    /// container word (the content the AT cannot name is content it
    /// cannot jump to — the landmarks list's own destination) and not a
    /// tab panel wearing the wrong word (a tab panel names *the
    /// selected page's content inside a strip*; a main names *the
    /// document's one central region*, the furniture the rest of the
    /// page reads beside). The region is the content's layout, never a
    /// second channel to it — the children keep the roles and labels
    /// they already carry — and the region holds no focus and answers
    /// no activation (the M126/M129 container rule, at the landmark
    /// tier).
    Main,
    /// The named strip the page closes with (the landmark tier, M143,
    /// ADR-0147): the "reads as the page's footer" fact AccessKit's own
    /// `ContentInfo` role spells — not a bare panel wearing no container
    /// word (the colophon the AT cannot name is a colophon it cannot
    /// offer on demand) and not a banner wearing the wrong half (a
    /// banner *names the page's opening strip*; the content info
    /// *names the closing one* — the copyright, the contact lines, the
    /// colophon the AT's landmarks list walks last). The strip is the
    /// furniture's layout, never a second channel to it — the children
    /// keep the roles and labels they already carry — and the strip
    /// holds no focus and answers no activation (the M126/M129
    /// container rule, at the landmark tier).
    ContentInfo,
    /// The named region the content beside the main lives in (the
    /// landmark tier's growth, M144, ADR-0148): the "reads as a
    /// complementary aside" fact AccessKit's own `Complementary` role
    /// spells — not a bare panel wearing no container word (the aside
    /// the AT cannot name is an aside it cannot offer on demand) and
    /// not a main wearing the wrong half (a main *names the document's
    /// one central region*; a complementary *names the content the
    /// main reads beside* — the reading list, the related links the
    /// AT's landmarks list walks beside the central region). The
    /// region is the furniture's layout, never a second channel to it —
    /// the children keep the roles and labels they already carry — and
    /// the region holds no focus and answers no activation (the
    /// M126/M129 container rule, at the landmark tier).
    Complementary,
    /// The named region the page's form lives in (the landmark tier's
    /// growth, M144, ADR-0148): the "reads as a form" fact AccessKit's
    /// own `Form` role spells — not a bare panel wearing no container
    /// word (the controls the AT cannot name as one form are controls
    /// it cannot offer as one) and not a main wearing the wrong word
    /// (a main *names the document's content*; a form *names the
    /// region whose controls submit together* — the checkout, the
    /// sign-in the AT's landmarks list walks as one unit). The region
    /// is the fields' layout, never a second channel to them — the
    /// children keep the roles and labels they already carry — and the
    /// region holds no focus and answers no activation (the M126/M129
    /// container rule, at the landmark tier).
    Form,
    /// A named region of the page (the landmark tier's growth, M144,
    /// ADR-0148): the "reads as a named region" fact AccessKit's own
    /// `Region` role spells — the generic word beside the specific
    /// ones (the banner, the way through, the main, the aside, the
    /// form, the search), for the section whose fact none of the
    /// specific roles spells and whose name the app still declares —
    /// not a bare panel wearing no container word (the region the AT
    /// cannot name is a region it cannot offer on demand, the ADR-5
    /// rule the named roles carry). The region is the children's
    /// layout, never a second channel to them — the children keep the
    /// roles and labels they already carry — and the region holds no
    /// focus and answers no activation (the M126/M129 container rule,
    /// at the landmark tier).
    Region,
    /// The named strip the page's search lives in (the landmark tier's
    /// growth, M144, ADR-0148): the "reads as a search region" fact
    /// AccessKit's own `Search` role spells — not a bare panel wearing
    /// no container word (the search the AT cannot name is a search it
    /// cannot offer on demand) and not a form wearing the wrong word
    /// (a form *names the region whose controls submit together*; a
    /// search *names the region whose controls find things* — the
    /// query field, the submit the AT's landmarks list walks as one
    /// unit). The strip is the fields' layout, never a second channel
    /// to them — the children keep the roles and labels they already
    /// carry — and the strip holds no focus and answers no activation
    /// (the M126/M129 container rule, at the landmark tier).
    Search,
}

impl Role {
    /// Roles whose accessible name is mandatory; the CI walker fails on
    /// unnamed widgets of these roles (§11.1). The checkbox joins the set in
    /// M6: an unnamed toggle is unusable with a screen reader. The slider
    /// joins in M20: an unnamed value control reads as a bare number. The
    /// progress indicator joins in M21: an unnamed report reads as a bare
    /// number too. The canvas joins in M82: an unnamed drawing surface
    /// reads as a blank. The radio joins in M124: an unnamed member is a
    /// bare mark, and an unnamed group a bare box — the set the AT cannot
    /// name is a set it cannot walk. The link joins in M125: an unnamed
    /// link reads as bare styled text — the "goes somewhere" the AT
    /// cannot announce. The heading joins in M126: an unnamed section
    /// title reads as bare styled text — the outline the AT cannot
    /// walk. The tab and its strip join in M127: an unnamed page is a
    /// bare mark, an unnamed strip a bare box — the ADR-5 rule the
    /// named roles carry. The spin button joins in M128: an unnamed
    /// number field reads as a bare number — the slider's own rule at
    /// the field tier. The toolbar joins in M129: an unnamed tool
    /// strip reads as a bare box — the set the AT cannot name is a set
    /// it cannot walk. The switch joins in M130: an unnamed instant
    /// on/off reads as a bare pill — the ADR-5 rule the named roles
    /// carry. The tab panel joins in M131: an unnamed pane reads as a
    /// bare box — the set the AT cannot name is a set it cannot walk,
    /// the named-container rule the strip and the tool strip carry.
    /// The meter joins in M133: an unnamed gauge reads as a bare
    /// number — the M21 progress rule's own shape, at the meter tier.
    /// The dialog joins in M134: an unnamed window reads as a bare
    /// box — the named-container rule the strip and the tool strip
    /// carry, at the modal tier. The tree and its items join in M135:
    /// an unnamed collection reads as a bare box, an unnamed member a
    /// bare mark — the named-container rule the strip and the tool
    /// strip carry, at the hierarchical tier. The tooltip joins in
    /// M136: an unnamed bubble reads as a bare box — the ADR-5 rule
    /// the named roles carry, at the hover tier. The menu bar, the menu
    /// and the menu item join in M137: an unnamed strip, an unnamed
    /// list and an unnamed entry read as bare boxes and a bare mark —
    /// the ADR-5 rule the named roles carry, at the menu tier. The
    /// image joins in M138: an unnamed picture reads as a bare mark —
    /// the ADR-5 rule the named roles carry, at the picture tier. The
    /// landmarks join in M143: an unnamed banner, an unnamed navigation
    /// strip, an unnamed main region and an unnamed footer read as bare
    /// boxes — the named-container rule the strip and the tool strip
    /// carry, at the landmark tier. The landmark tier's remaining words
    /// join in M144: an unnamed aside, an unnamed form, an unnamed
    /// region and an unnamed search strip read as bare boxes — the
    /// same rule, the growth the constructor words ride.
    #[must_use]
    pub const fn requires_name(self) -> bool {
        matches!(
            self,
            Self::Button
                | Self::TextInput
                | Self::CheckBox
                | Self::Slider
                | Self::ProgressIndicator
                | Self::Canvas
                | Self::Radio
                | Self::RadioGroup
                | Self::Link
                | Self::Heading
                | Self::Tab
                | Self::TabList
                | Self::SpinButton
                | Self::Toolbar
                | Self::Switch
                | Self::TabPanel
                | Self::Meter
                | Self::Dialog
                | Self::Tree
                | Self::TreeItem
                | Self::Tooltip
                | Self::MenuBar
                | Self::Menu
                | Self::MenuItem
                | Self::Image
                | Self::Banner
                | Self::Navigation
                | Self::Main
                | Self::ContentInfo
                | Self::Complementary
                | Self::Form
                | Self::Region
                | Self::Search
        )
    }

    /// Human-readable role name, used by the CI walker's diagnostics.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Decoration => "decoration",
            Self::Container => "container",
            Self::Label => "label",
            Self::Button => "button",
            Self::TextInput => "text_input",
            Self::ScrollArea => "scroll_area",
            Self::ListItem => "list_item",
            Self::List => "list",
            Self::CheckBox => "checkbox",
            Self::Slider => "slider",
            Self::ProgressIndicator => "progress_indicator",
            Self::Table => "table",
            Self::TableRow => "table_row",
            Self::TableCell => "table_cell",
            Self::ColumnHeader => "column_header",
            Self::Canvas => "canvas",
            Self::LiveRegion => "live_region",
            Self::Radio => "radio",
            Self::RadioGroup => "radio_group",
            Self::Link => "link",
            Self::Heading => "heading",
            Self::Tab => "tab",
            Self::TabList => "tab_list",
            Self::SpinButton => "spin_button",
            Self::Toolbar => "toolbar",
            Self::Switch => "switch",
            Self::TabPanel => "tab_panel",
            Self::Meter => "meter",
            Self::Dialog => "dialog",
            Self::Tree => "tree",
            Self::TreeItem => "tree_item",
            Self::Tooltip => "tooltip",
            Self::MenuBar => "menu_bar",
            Self::Menu => "menu",
            Self::MenuItem => "menu_item",
            Self::Image => "image",
            Self::Banner => "banner",
            Self::Navigation => "navigation",
            Self::Main => "main",
            Self::ContentInfo => "content_info",
            Self::Complementary => "complementary",
            Self::Form => "form",
            Self::Region => "region",
            Self::Search => "search",
        }
    }

    /// The name door's inverse (M147, ADR-0151): the vocabulary's
    /// `snake_case` word parsed back to the role it names — the wire's
    /// word ask rides it, and any consumer that speaks the same words
    /// (the diagnostics, the tests, the tooling) parses with the same
    /// door. One vocabulary, two doors, the round-trip pin in
    /// `passes`' suite keeping the two unable to drift apart. A word
    /// outside the vocabulary answers `None` — the honest refusal the
    /// caller names (the wire's malformed-frame behavior, at the name
    /// tier).
    #[must_use]
    pub fn from_name(word: &str) -> Option<Self> {
        match word {
            "decoration" => Some(Self::Decoration),
            "container" => Some(Self::Container),
            "label" => Some(Self::Label),
            "button" => Some(Self::Button),
            "text_input" => Some(Self::TextInput),
            "scroll_area" => Some(Self::ScrollArea),
            "list_item" => Some(Self::ListItem),
            "list" => Some(Self::List),
            "checkbox" => Some(Self::CheckBox),
            "slider" => Some(Self::Slider),
            "progress_indicator" => Some(Self::ProgressIndicator),
            "table" => Some(Self::Table),
            "table_row" => Some(Self::TableRow),
            "table_cell" => Some(Self::TableCell),
            "column_header" => Some(Self::ColumnHeader),
            "canvas" => Some(Self::Canvas),
            "live_region" => Some(Self::LiveRegion),
            "radio" => Some(Self::Radio),
            "radio_group" => Some(Self::RadioGroup),
            "link" => Some(Self::Link),
            "heading" => Some(Self::Heading),
            "tab" => Some(Self::Tab),
            "tab_list" => Some(Self::TabList),
            "spin_button" => Some(Self::SpinButton),
            "toolbar" => Some(Self::Toolbar),
            "switch" => Some(Self::Switch),
            "tab_panel" => Some(Self::TabPanel),
            "meter" => Some(Self::Meter),
            "dialog" => Some(Self::Dialog),
            "tree" => Some(Self::Tree),
            "tree_item" => Some(Self::TreeItem),
            "tooltip" => Some(Self::Tooltip),
            "menu_bar" => Some(Self::MenuBar),
            "menu" => Some(Self::Menu),
            "menu_item" => Some(Self::MenuItem),
            "image" => Some(Self::Image),
            "banner" => Some(Self::Banner),
            "navigation" => Some(Self::Navigation),
            "main" => Some(Self::Main),
            "content_info" => Some(Self::ContentInfo),
            "complementary" => Some(Self::Complementary),
            "form" => Some(Self::Form),
            "region" => Some(Self::Region),
            "search" => Some(Self::Search),
            _ => None,
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Where a widget's accessible name comes from.
#[derive(Clone)]
pub enum LabelSource {
    /// A fixed string owned by the widget.
    Text(Cow<'static, str>),
    /// A live string shared with reactive state (e.g. a bound label).
    Computed(Arc<Mutex<String>>),
    /// A render binding — the same slot the widget paints from.
    Binding(crate::flux::RenderBinding<String>),
    /// The name is the concatenation of descendant names (containers).
    FromChildren,
}

impl fmt::Debug for LabelSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(t) => f.debug_tuple("Text").field(&t).finish(),
            Self::Computed(s) => {
                if let Ok(guard) = s.lock() {
                    f.debug_tuple("Computed").field(&guard.as_str()).finish()
                } else {
                    f.debug_tuple("Computed").field(&"<poisoned>").finish()
                }
            }
            Self::Binding(b) => f
                .debug_tuple("Binding")
                .field(&b.with(String::len))
                .finish(),
            Self::FromChildren => f.write_str("FromChildren"),
        }
    }
}

impl LabelSource {
    /// Resolves to the current accessible name. Never panics: a poisoned
    /// name cell resolves to the empty string — a name is a label, never
    /// a frame-buster.
    #[must_use]
    pub fn resolve(&self) -> String {
        match self {
            Self::Text(t) => t.to_string(),
            Self::Computed(cell) => cell.lock().map(|guard| guard.clone()).unwrap_or_default(),
            Self::Binding(binding) => binding.with(Clone::clone),
            Self::FromChildren => String::new(),
        }
    }

    /// A fixed-text source from any string-like value.
    #[must_use]
    pub fn text(t: impl Into<Cow<'static, str>>) -> Self {
        Self::Text(t.into())
    }
}

/// How far an [`A11yAction::ScrollUp`], [`A11yAction::ScrollDown`],
/// [`A11yAction::ScrollLeft`], or [`A11yAction::ScrollRight`] should
/// travel, normalized across platforms (the AccessKit `ScrollUnit` shape,
/// mirrored in core so the DAG stays adapter-free — ADR-9). Since M91
/// (ADR-0095) the unit travels the horizontal pair too: an item is one
/// line, a page is the viewport — the viewport's *width* on the x axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollUnit {
    /// A single item of a list (one line, the platform's 40 px wheel notch).
    Item,
    /// The amount of content that fits in the viewport.
    Page,
}

impl ScrollUnit {
    /// The unit's name, the wire vocabulary's `snake_case` tag (M52,
    /// ADR-0058; the protocol's stable, documented field values — the
    /// asker's unit the scroll record carries beside the unit-scroll
    /// actions, the same naming rule the action species and the
    /// refusal species ride).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Item => "item",
            Self::Page => "page",
        }
    }
}

/// The single axis a scroll container declares its scrolls to ride
/// (M98, ADR-0102; the AccessKit `Orientation` shape, mirrored in core
/// so the DAG stays adapter-free — ADR-9).
///
/// The vocabulary is deliberately two-variant: it names a *single-axis*
/// declaration, the word ARIA's `aria-orientation` carries. A container
/// that declared both axes (the two-axis list, ADR-0093) has no
/// single-axis word to say — its honest answer is the trait door's
/// `None`, the absence ARIA spells `undefined`. The enum never guesses
/// an axis the constructor did not name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollOrientation {
    /// The declared scrolls ride the y axis — the vertical track.
    Vertical,
    /// The declared scrolls ride the x axis — the horizontal track.
    Horizontal,
}

impl ScrollOrientation {
    /// The orientation's name, the wire vocabulary's `snake_case` tag
    /// (the same naming rule [`ScrollUnit::name`] rides — the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
        }
    }
}

/// One scroll axis's position and extent (M101, ADR-0105): the offset
/// the clamp landed and the wall it clamps to — AccessKit's
/// `ScrollY`/`ScrollYMax` shape (the x halves beside it), mirrored in
/// core so the DAG stays adapter-free (ADR-9). The floor is the
/// funnel's own zero — the clamp arithmetic every scroll door answers
/// by (`clamp(0.0, span)`), never a guessed minimum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollAxisPosition {
    /// The resolved offset the doors landed (positive = scrolled down
    /// on the vertical axis, right on the horizontal one) — the same
    /// state [`Widget::scroll_offset`] names for compose.
    pub offset: f32,
    /// The wall the clamp lands on: the declared span (content minus
    /// viewport), the largest offset any door can land. A position at
    /// the wall is the clamp's own landing — the tree says where the
    /// container stopped.
    pub max: f32,
}

/// The scroll container's resolved position and extent (M101,
/// ADR-0105): the fact the doors resolve and the AT reads as "scrolled
/// 200 of 0..600" — AccessKit's `ScrollY` / `ScrollYMin` / `ScrollYMax`
/// properties (the `ScrollX` halves beside them on a two-axis
/// container), mapped by `alacer-a11y` in the same snapshot every other
/// property rides.
///
/// Each axis rides its own honesty: an axis whose span is empty (the
/// content fits the viewport, or the width story was never declared)
/// carries no position — the same wall every scroll door answers
/// `no_scrollable_content` with, never a guessed zero. A container with
/// no live axis on either side answers `None` outright.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollPosition {
    /// The vertical axis's fact — the story every scroll container
    /// declares first. `None` when the y span is empty.
    pub vertical: Option<ScrollAxisPosition>,
    /// The horizontal axis's fact (M89, ADR-0093's other axis) —
    /// `None` when the x span is empty or undeclared.
    pub horizontal: Option<ScrollAxisPosition>,
}

/// A range of text, expressed in the contract's unit — **UTF-16 code
/// units** into the widget's value (§10.3; the same unit as the caret).
///
/// `anchor` is where the selection started and does not move as the
/// selection grows; `focus` is the active end (the caret while extending).
/// `anchor == focus` is the degenerate selection — a bare caret. Every
/// conversion to Rust's UTF-8 strings funnels through
/// [`crate::utf16`], so surrogate pairs never slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextSelection {
    /// Where the selection began, as a UTF-16 offset into the value.
    pub anchor: usize,
    /// The active end (the caret while extending), as a UTF-16 offset.
    pub focus: usize,
}

impl TextSelection {
    /// A selection from `anchor` to `focus` (both UTF-16 offsets).
    #[must_use]
    pub const fn new(anchor: usize, focus: usize) -> Self {
        Self { anchor, focus }
    }

    /// The selection's edges in ascending order: `(start, end)`.
    #[must_use]
    pub const fn normalized(&self) -> (usize, usize) {
        if self.anchor <= self.focus {
            (self.anchor, self.focus)
        } else {
            (self.focus, self.anchor)
        }
    }
}

/// An accessibility action requested by an assistive technology.
///
/// The vocabulary is the routing contract (ADR-0016): the platform maps each
/// AccessKit action onto exactly one variant — an action a widget does not
/// implement is *refused* by that widget, never munged into a different
/// action. Growing the enum is how the AT tier widens; every variant gains
/// a consuming widget in the same milestone. M11 (ADR-0017) adds the value
/// tier: `SetValue`, `ReplaceSelectedText`, and `SetTextSelection` carry
/// their payload, so the enum is `Clone` but no longer `Copy` — the price
/// of a faithful mapping, and the reason refusal (no allocation, no move)
/// stays observable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum A11yAction {
    /// The default action — "press" for buttons, "toggle" for checkboxes.
    DefaultPress,
    /// Increment a value by one step (a scroll area steps by one page).
    Increment,
    /// Decrement a value by one step (a scroll area steps by one page).
    Decrement,
    /// Scroll up by `unit` (the AT's scroll command on a scroll area).
    ScrollUp(ScrollUnit),
    /// Scroll down by `unit` (the AT's scroll command on a scroll area).
    ScrollDown(ScrollUnit),
    /// Scroll left by `unit` (M91, ADR-0095): the AT's horizontal
    /// vocabulary beside the vertical pair — the same unit contract
    /// (an item is one line, a page is the viewport's width), consumed
    /// by the scroll area's own clamp arithmetic on the declared width
    /// span.
    ScrollLeft(ScrollUnit),
    /// Scroll right by `unit` (M91, ADR-0095): the AT's horizontal
    /// vocabulary beside the vertical pair — the same unit contract
    /// (an item is one line, a page is the viewport's width), consumed
    /// by the scroll area's own clamp arithmetic on the declared width
    /// span.
    ScrollRight(ScrollUnit),
    /// Replace the widget's whole value and reset the selection (M11,
    /// ADR-0017) — the AccessKit `SetValue` mapping, faithful to the AT's
    /// request.
    SetValue(String),
    /// Delete any selected text and insert `value` in its place (M11,
    /// ADR-0017) — the AT typing on the user's behalf, like a paste.
    ReplaceSelectedText(String),
    /// Move the selection directly to `anchor`/`focus` (M11, ADR-0017) —
    /// the AT repositioning the caret or the range, UTF-16 units.
    SetTextSelection(TextSelection),
}

impl A11yAction {
    /// The action's name, the wire vocabulary's `snake_case` tag (M57,
    /// ADR-0063): the stable, documented field value the route-drop
    /// trail's records carry beside their species — the dropped
    /// action's own name, never a munge onto the action it did not
    /// become (the platform boundary's mapping uses the same tags for
    /// the actions the two vocabularies share).
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::DefaultPress => "default_press",
            Self::Increment => "increment",
            Self::Decrement => "decrement",
            Self::ScrollUp(_) => "scroll_up",
            Self::ScrollDown(_) => "scroll_down",
            Self::ScrollLeft(_) => "scroll_left",
            Self::ScrollRight(_) => "scroll_right",
            Self::SetValue(_) => "set_value",
            Self::ReplaceSelectedText(_) => "replace_selected_text",
            Self::SetTextSelection(_) => "set_text_selection",
        }
    }

    /// The ask's own word (M156, ADR-0161): the one bit the ask occupies
    /// in the advertised set — the bridge between the routing vocabulary
    /// and the tree's, so the question "is the word the door's answer?"
    /// is one comparison per variant, payload left at the door.
    #[must_use]
    pub const fn advertises(&self) -> A11yActions {
        match self {
            Self::DefaultPress => A11yActions::DEFAULT_PRESS,
            Self::Increment => A11yActions::INCREMENT,
            Self::Decrement => A11yActions::DECREMENT,
            Self::ScrollUp(_) => A11yActions::SCROLL_UP,
            Self::ScrollDown(_) => A11yActions::SCROLL_DOWN,
            Self::ScrollLeft(_) => A11yActions::SCROLL_LEFT,
            Self::ScrollRight(_) => A11yActions::SCROLL_RIGHT,
            Self::SetValue(_) => A11yActions::SET_VALUE,
            Self::ReplaceSelectedText(_) => A11yActions::REPLACE_SELECTED_TEXT,
            Self::SetTextSelection(_) => A11yActions::SET_TEXT_SELECTION,
        }
    }
}

bitflags! {
    /// The asks a control's door answers, said by the tree (M156,
    /// ADR-0161): the action tier's word — the set the AT reads before
    /// it asks, mapped by `alacer-a11y` onto AccessKit's action set
    /// (ARIA's per-role action vocabulary: `click` on a button,
    /// `increment` on a slider). Through M155 the tree said what a
    /// control *is*, its name, its value, whether it is available — and
    /// the asks its door answered had no tier: the platform mapped ten
    /// AccessKit actions onto [`A11yAction`]'s vocabulary (M10, M11,
    /// M91) and every canon control answered or refused each ask
    /// honestly, while the node the AT walked to advertised none of
    /// them — the declared-but-unreported gap (the M94 pattern), at the
    /// action tier.
    ///
    /// The word is the door's own answer said before the ask: a control
    /// advertises exactly the asks its [`Widget::on_a11y_action`]
    /// consumes in the state it is in — an unavailable control
    /// advertises none (the M22 rule reaching the word: a closed door is
    /// no capability), a read-only field the selection ask alone (M105).
    /// The word says the door and opens nothing: an ask still travels
    /// the route it always travelled, the door still answers each ask on
    /// its own merits, and a malformed payload is still refused, still
    /// named. The bits are payload-free — the word names the *kind* of
    /// ask, never the value a `SetValue` would carry (the
    /// lengths-not-payload rule at the tier).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct A11yActions: u16 {
        /// The default ask — "press" for buttons, "toggle" for checkboxes.
        const DEFAULT_PRESS = 1 << 0;
        /// The step up — a value control's one step, a scroll area's one page.
        const INCREMENT = 1 << 1;
        /// The step down — a value control's one step, a scroll area's one page.
        const DECREMENT = 1 << 2;
        /// The travel up, whatever unit the ask names.
        const SCROLL_UP = 1 << 3;
        /// The travel down, whatever unit the ask names.
        const SCROLL_DOWN = 1 << 4;
        /// The travel left, whatever unit the ask names (M91, ADR-0095).
        const SCROLL_LEFT = 1 << 5;
        /// The travel right, whatever unit the ask names (M91, ADR-0095).
        const SCROLL_RIGHT = 1 << 6;
        /// The whole-value write.
        const SET_VALUE = 1 << 7;
        /// The paste-shaped write over the selection.
        const REPLACE_SELECTED_TEXT = 1 << 8;
        /// The selection moved to the ask's UTF-16 offsets.
        const SET_TEXT_SELECTION = 1 << 9;
    }
}

/// A toggle control's checked state, carried neutrally to the a11y sink
/// (M6): `alacer-a11y` maps it onto AccessKit's `Toggled` property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToggleState {
    /// Unchecked.
    Off,
    /// Checked.
    On,
    /// Partially checked (tri-state parents).
    Mixed,
}

impl From<bool> for ToggleState {
    fn from(checked: bool) -> Self {
        if checked { Self::On } else { Self::Off }
    }
}

/// A node's position inside a data table, carried neutrally to the a11y
/// sink (M6). `alacer-a11y` maps the variants onto AccessKit's table
/// attributes — a grid a screen reader can navigate needs positions, not
/// just roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TableSlot {
    /// The table itself: its live geometry (rows × columns).
    Table {
        /// Number of data rows currently mounted.
        rows: usize,
        /// Number of columns (header count).
        columns: usize,
    },
    /// A column header for zero-based `column`.
    Header {
        /// Zero-based column index.
        column: usize,
    },
    /// A data cell at zero-based `(row, column)`.
    Cell {
        /// Zero-based row index.
        row: usize,
        /// Zero-based column index.
        column: usize,
    },
}

/// The bounded scalar's declared span (M97, ADR-0101): the range every
/// value-tier write funnels through, carried neutrally to the a11y sink.
/// `alacer-a11y` maps `min`/`max` onto AccessKit's `MinNumericValue` /
/// `MaxNumericValue` properties — ARIA's `aria-valuemin` /
/// `aria-valuemax` — and `step`, when the widget declares one, onto
/// `NumericValueStep`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValueRange {
    /// The range's lower wall: the smallest value every door lands on.
    pub min: f64,
    /// The range's upper wall: the largest value every door lands on.
    pub max: f64,
    /// The amount one step moves (`None` when the widget declares no
    /// step — the honest absence, never a guessed default).
    pub step: Option<f64>,
}

impl ValueRange {
    /// A span with no step: the report half's shape (the progress
    /// declares where its value lives, not how far one act moves it).
    #[must_use]
    pub const fn new(min: f64, max: f64) -> Self {
        Self {
            min,
            max,
            step: None,
        }
    }

    /// Attaches the step one act moves (the slider's snap arithmetic).
    #[must_use]
    pub const fn with_step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }
}

/// The invalid tier's declared kind (M107, ADR-0111): *how* a control's
/// value fails the form's contract, carried neutrally to the a11y sink.
/// `alacer-a11y` maps the word onto AccessKit's `Invalid` property —
/// ARIA's `aria-invalid` — the variants ARIA's own three values
/// (`true` / `grammar` / `spelling`), so the AT reads the rejection and
/// the kind of rejection in one fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidKind {
    /// The bare rejection (`aria-invalid="true"`): the value is not
    /// valid as given, no prose-error kind named.
    True,
    /// A grammatical error the AT may surface natively
    /// (`aria-invalid="grammar"`).
    Grammar,
    /// A spelling error the AT may surface natively
    /// (`aria-invalid="spelling"`).
    Spelling,
}

/// The auto-complete tier's declared kind (M111, ADR-0115): *how* a
/// field's suggestions ride the contract, carried neutrally to the a11y
/// sink. `alacer-a11y` maps the word onto AccessKit's `AutoComplete`
/// property — ARIA's `aria-autocomplete` — the variants AccessKit's own
/// three values (`inline` / `list` / `both`), so the AT reads the
/// suggestion contract and its mode in one fact. ARIA's fourth value
/// (`none`) is the honest absence — [`Widget::auto_complete`] answering
/// `None` — never a variant of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoCompleteKind {
    /// The field completes the entry inline as the user types
    /// (`aria-autocomplete="inline"`).
    Inline,
    /// The field presents a list of possible completions
    /// (`aria-autocomplete="list"`).
    List,
    /// The field both completes inline and presents the list
    /// (`aria-autocomplete="both"`).
    Both,
}

/// The sort direction tier's declared kind (M112, ADR-0116): *how* a
/// sorted column orders its rows, carried neutrally to the a11y sink.
/// `alacer-a11y` maps the word onto AccessKit's `SortDirection`
/// property — ARIA's `aria-sort` — the variants AccessKit's own three
/// values (`ascending` / `descending` / `other`), so the AT reads the
/// column's order in one fact. ARIA's fourth value (`none`) is the
/// honest absence — [`Widget::sort_direction`] answering `None` — never
/// a variant of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// The column sorts ascending — the AT announces "sorted ascending"
    /// while it walks the rows (`aria-sort="ascending"`).
    Ascending,
    /// The column sorts descending — the AT announces "sorted
    /// descending" while it walks the rows (`aria-sort="descending"`).
    Descending,
    /// The column sorts by an order ARIA names no word for — the AT
    /// reads the declared sort without guessing a direction
    /// (`aria-sort="other"`).
    Other,
}

/// The current tier's declared kind (M114, ADR-0118): *which* item of a
/// set the reader stands in, carried neutrally to the a11y sink.
/// `alacer-a11y` maps the word onto AccessKit's `AriaCurrent` property —
/// ARIA's `aria-current` — the variants AccessKit's own six values
/// (`true` / `page` / `step` / `location` / `date` / `time`), so the AT
/// reads the currency and its flavor in one fact. ARIA's `false` (the
/// attribute's absent default) is the honest absence —
/// [`Widget::current`] answering `None` — never a variant of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentKind {
    /// The set's current item, with no finer flavor to name
    /// (`aria-current="true"`).
    True,
    /// The item is the page the reader stands on — the nav's page
    /// (`aria-current="page"`).
    Page,
    /// The item is the step the reader stands at — the wizard's step
    /// (`aria-current="step"`).
    Step,
    /// The item is the place the reader stands in — the flow's location
    /// (`aria-current="location"`).
    Location,
    /// The item is the day the reader stands on — the calendar's today
    /// (`aria-current="date"`).
    Date,
    /// The item is the time the reader stands at — the schedule's now
    /// (`aria-current="time"`).
    Time,
}

/// The has-popup tier's declared kind (M115, ADR-0119): *what* a
/// control opens, carried neutrally to the a11y sink. `alacer-a11y`
/// maps the word onto AccessKit's `HasPopup` property — ARIA's
/// `aria-haspopup` — the variants AccessKit's own five values (`menu` /
/// `listbox` / `tree` / `grid` / `dialog`), so the AT reads the target
/// and its shape in one fact. The attribute's unset state is the honest
/// absence — [`Widget::has_popup`] answering `None` — never a variant
/// of its own (ARIA's `true` and `false` ride `Menu` and the unset
/// attribute respectively, the values the platform maps).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HasPopupKind {
    /// The control opens a menu (`aria-haspopup="menu"`).
    Menu,
    /// The control opens a listbox — the combobox's pick list
    /// (`aria-haspopup="listbox"`).
    Listbox,
    /// The control opens a tree (`aria-haspopup="tree"`).
    Tree,
    /// The control opens a grid — the date picker's cells
    /// (`aria-haspopup="grid"`).
    Grid,
    /// The control opens a dialog (`aria-haspopup="dialog"`).
    Dialog,
}

/// What happened to a routed event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventResponse {
    /// The widget did not consume the event; it bubbles on.
    #[default]
    Ignored,
    /// The widget consumed the event; bubbling stops.
    Handled,
}

/// The bring-into-view ask's two halves (M89, ADR-0093): the
/// content-space target offsets the pipeline priced per axis — the
/// vertical half exactly the M88 ask, the horizontal half beside it
/// priced by the same edge-pin convention from the bounds ledger the
/// door already owns. One ask, both axes: the container's door
/// answers each by its own declared span (the clamp arithmetic, the
/// named walls), never the pipeline guessing at a widget-private
/// range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BringIntoViewAsk {
    /// The vertical half: the content-space target offset the minimal
    /// vertical scroll prices (the M88 ask — above the window scrolls
    /// to the top edge, below it to the bottom edge, a rect taller
    /// than the viewport pins its top edge).
    pub vertical: f32,
    /// The horizontal half beside it (M89, ADR-0093): the content-space
    /// target offset the minimal horizontal scroll prices — left of
    /// the window scrolls to the left edge, right of it to the right
    /// edge, a rect wider than the viewport pins its left edge.
    pub horizontal: f32,
}

/// One axis's answer to a bring-into-view ask (M89, ADR-0093): the
/// clamp's arithmetic carried as data for that axis — the M27 rule at
/// the scroll tier, the shape [`ScrollLanding`] rode before the door
/// grew the axis. `offset` is the landed offset the container holds
/// on the axis (the clamp's result on a delivery, the untouched
/// offset on a refusal); `pinned` names the wall the clamp turned the
/// ask back at (the ADR-0055 rule the ask doors answer by — `None`
/// when the landing is the ask, on every landing mid-range, and on
/// every refusal).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollAxisLanding {
    /// What the container's clamp answered on this axis — delivered,
    /// or the named refusal species ([`crate::inspect::ScrollRefusal`]).
    pub outcome: crate::inspect::ScrollOutcome,
    /// The offset the container holds on this axis as the answer
    /// resolves.
    pub offset: f32,
    /// The wall the clamp pinned a past ask at (`min`/`max`), `None`
    /// when the landing is the ask or nothing moved.
    pub pinned: Option<crate::inspect::RangeBound>,
}

/// The container's answer to a bring-into-view ask (M88, ADR-0092):
/// the landing beside the outcome, the clamp's arithmetic carried as
/// data — the M27 rule at the scroll tier, the same shape the focus
/// walk's [`crate::passes::FocusLanding`] answers by. `offset` is the landed
/// offset the container holds (the clamp's result on a delivery, the
/// untouched offset on a refusal); `pinned` names the wall the clamp
/// turned the ask back at (the ADR-0055 rule the ask doors answer by —
/// `None` when the landing is the ask, on every landing mid-range, and
/// on every refusal). Since M89 (ADR-0093) the fields answer the
/// *vertical* axis and the horizontal axis's own answer rides beside
/// them ([`ScrollAxisLanding`], the same shape, the same rules — the
/// additive shape the scroll record's own halves ride).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollLanding {
    /// What the container's clamp answered — delivered, or the named
    /// refusal species ([`crate::inspect::ScrollRefusal`]).
    pub outcome: crate::inspect::ScrollOutcome,
    /// The offset the container holds as the answer resolves.
    pub offset: f32,
    /// The wall the clamp pinned a past ask at (`min`/`max`), `None`
    /// when the landing is the ask or nothing moved.
    pub pinned: Option<crate::inspect::RangeBound>,
    /// The horizontal axis's own answer (M89, ADR-0093), beside the
    /// vertical — priced by the same door, clamped by the same
    /// arithmetic, refused by its own declared span.
    pub horizontal: ScrollAxisLanding,
}

/// The retained widget contract.
///
/// Widgets are never re-created on state change (§6.1): they are created once,
/// mounted into the [`crate::arena::Arena`], and mutated in place. Reactive
/// updates land through `alacer-core::flux::RenderBinding`s, not through this
/// trait.
pub trait Widget: 'static {
    /// The widget's semantic role. Required: the a11y contract starts here.
    fn role(&self) -> Role;

    /// The widget's accessible name source. Required: see ADR-5.
    fn label(&self) -> LabelSource;

    /// Layout styles. Paint properties stay on the widget (§7.2 pass 5).
    fn style(&self) -> Style {
        Style::default()
    }

    /// Emits this widget's own primitives (background, text). Children are
    /// painted by the pass system walking the tree; containers override only
    /// for decorations.
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let _ = ctx;
    }

    /// Scroll offset this widget applies to its children during compose
    /// (translation-only updates; §8.2). Non-scrolling widgets return zero.
    fn scroll_offset(&self) -> crate::geom::Vector {
        crate::geom::Vector::zero()
    }

    /// Whether this widget's children translate with [`Self::scroll_offset`].
    ///
    /// The arena indexes scroll containers so the compose pass visits only
    /// real candidates instead of walking every node each frame.
    fn is_scroll_container(&self) -> bool {
        false
    }

    /// The compose tier's paint state ([`crate::scene::Compose`], §14.2's
    /// W3 mechanism): the fade and translation this widget paints itself
    /// and its subtree with. The compose pass folds a node's own state
    /// beside every ancestor's into the paint context — nested fades
    /// multiply, translations add — and the pushes apply the product at
    /// record time, the fade into the alpha channel the renderers already
    /// blend with, the translation into the recorded coordinates. The
    /// default composes nothing: a widget that never overrides it records
    /// exactly the primitives it always did (the byte-identical baseline
    /// the compose-opacity suite pins).
    ///
    /// The state is a *paint* fact, never a layout one (styles stay
    /// layout-only, §7.2 pass 5): an animated frame moves it without
    /// re-measuring — the M8 scrolling contract's tier, generalized from
    /// the scroll offset to any compose state.
    fn compose(&self) -> crate::scene::Compose {
        crate::scene::Compose::IDENTITY
    }

    /// The container's half of the bring-into-view door (M88, ADR-0092):
    /// the pipeline reads the bounds ledger it already owns, computes the
    /// minimal offset that lands the aimed node fully inside this
    /// container's viewport, and hands the ask here — the widget answers
    /// by its own clamp arithmetic (the span walls, the named refusals),
    /// the range living where the widget keeps it, never the pipeline
    /// guessing at a private span. Since M89 (ADR-0093) the ask carries
    /// both halves ([`BringIntoViewAsk`]): the vertical half exactly the
    /// M88 ask, the horizontal half beside it priced by the same
    /// edge-pin convention — and the container answers each axis by its
    /// own declared span, the landing reporting both ([`ScrollLanding`]'s
    /// vertical fields beside [`ScrollAxisLanding`]). The landing is data
    /// (the M27 rule at the scroll tier); the door notes the ask on the
    /// scroll log under [`crate::inspect::ScrollAction::BringIntoView`],
    /// the route stamps filling the container and the aimed node the
    /// widget cannot know (the M43 rule at the scroll tier). `None` when
    /// this widget claims the scroll slot ([`Self::is_scroll_container`])
    /// without shipping the door — the pipeline answers
    /// [`crate::passes::ScrollIntoViewRefusal::NoScrollDoor`] on that
    /// answer, the contract gap named where the consumer can read it.
    fn bring_into_view(&mut self, ask: BringIntoViewAsk) -> Option<ScrollLanding> {
        let _ = ask;
        None
    }

    /// The widget's intrinsic measure request, consumed by the layout engine's
    /// measure functions.
    fn measure_spec(&self) -> Option<crate::style::MeasureSpec> {
        None
    }

    /// Handles a routed pointer event targeting this widget.
    ///
    /// Mutate your own state here and request masks through `ctx` — never
    /// touch other widgets (the pass system owns the tree).
    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        let _ = (ctx, event);
        EventResponse::Ignored
    }

    /// Handles a key event routed to this widget because it holds focus
    /// (§10.3 — keyboard input is focus-routed, never hit-tested). Editing
    /// widgets act on [`crate::events::KeyState::Pressed`] only; returning
    /// [`EventResponse::Ignored`] lets the framework take the key (Tab's focus cycling).
    fn on_key_event(&mut self, ctx: &mut EventCtx, event: &KeyEvent) -> EventResponse {
        let _ = (ctx, event);
        EventResponse::Ignored
    }

    /// Handles an IME event routed to this widget because it holds focus
    /// (§10.3, the IME contract's event half). Preedit is *composing* input:
    /// display it inline (the platform cursor is a UTF-16 offset within the
    /// preedit); an empty preedit cancels; [`ImeEvent::Commit`] inserts final
    /// text at the caret and clears any preedit.
    fn on_ime_event(&mut self, ctx: &mut EventCtx, event: &ImeEvent) -> EventResponse {
        let _ = (ctx, event);
        EventResponse::Ignored
    }

    /// Notifies the widget that keyboard focus moved to it (`true`) or away
    /// (`false`) — the focus-ring hook (§7.2 pass 3). Widgets store the flag
    /// and repaint on their next look; the framework already merges the
    /// PAINT mask on both endpoints of the move.
    fn set_focused(&mut self, focused: bool) {
        let _ = focused;
    }

    /// The IME flavor this widget wants while focused (§10.3).
    /// [`crate::passes::ImePurpose::Off`] (the default) tells the platform no IME is needed; the pipeline
    /// derives the frame's [`crate::passes::ImeRequest`] from the focused
    /// widget's answer.
    fn ime_purpose(&self) -> crate::passes::ImePurpose {
        crate::passes::ImePurpose::Off
    }

    /// The widget's exposed value for the a11y tree (M3 contract): editable
    /// widgets report their content so screen readers read the text, not just
    /// the name. Valueless roles return `None`.
    fn value(&self) -> Option<String> {
        None
    }

    /// Whether the control accepts interaction (M22, ADR-0028): the
    /// availability tier's single source of truth. An unavailable control
    /// reports itself to the a11y tree (the sink maps the flag onto
    /// AccessKit's `disabled` property), refuses every door honestly —
    /// the pointer is a wall (the gesture never clicks through), the
    /// keyboard never reaches it, and every AT action is refused, never
    /// munged (the M11 rule) — holds no focus (the default [`Self::
    /// focusable`] derivation consults this), and is never a Tab stop.
    /// Reporting widgets and decorations are always available: only
    /// actionable controls override this.
    fn enabled(&self) -> bool {
        true
    }

    /// The widget's description for the a11y tree (M95, ADR-0099): the
    /// help text a screen reader reads when the name alone does not
    /// answer "what does this control do?" — AccessKit's `Description`
    /// property, ARIA's `aria-description`, mapped by `alacer-a11y` in
    /// the same snapshot every other property rides. The name is the
    /// control's identity; the description is its *instruction* — a
    /// format ("the 16 digits on the front"), a consequence ("saves the
    /// draft"), the sentence the label is too short to carry.
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// description tier): a widget with nothing to explain announces
    /// nothing, never a guess — the tree M94 handed over. Actionable
    /// controls override this; containers and reports inherit the
    /// default.
    fn description(&self) -> Option<&str> {
        None
    }

    /// The widget's placeholder for the a11y tree (M96, ADR-0100): the
    /// hint text an empty editable announces before the user types —
    /// AccessKit's `Placeholder` property, ARIA's `aria-placeholder`,
    /// mapped by `alacer-a11y` in the same snapshot every other property
    /// rides. Through M95 the field painted the hint and the tree could
    /// not say it: the declared-but-unreported gap, the M94 pattern at
    /// the placeholder tier.
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// placeholder tier): a field with no hint announces nothing, never
    /// a guess — an empty string is the absence, not an empty hint.
    fn placeholder(&self) -> Option<&str> {
        None
    }

    /// The widget's declared value range for the a11y tree (M97,
    /// ADR-0101): the span every value-tier write funnels through — the
    /// clamp-and-snap arithmetic the drag, the arrows, and the AT's own
    /// acts all ride — carried as AccessKit's `MinNumericValue` /
    /// `MaxNumericValue` properties beside the `NumericValueStep` the
    /// slider declares (ARIA's `aria-valuemin` / `aria-valuemax` / the
    /// step). Through M96 the tree said the value ("42") and not the
    /// span — whether that was the middle of 0..100 or the top of
    /// 0..50 — the declared-but-unreported gap, the M94 pattern at the
    /// range tier.
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// range tier): a widget with no span announces nothing, never a
    /// guessed default. The value tier's controls override this; every
    /// other role inherits the default.
    fn value_range(&self) -> Option<ValueRange> {
        None
    }

    /// The widget's numeric now for the a11y tree (M99, ADR-0103): the
    /// number behind the value's spoken word — the scalar the value
    /// tier's funnel holds, carried as AccessKit's `NumericValue`
    /// property, ARIA's `aria-valuenow`, riding the text the tree
    /// already carried as the value's `aria-valuetext`. Through M98 the
    /// tree said the value as text ("40") and not the number (40): the
    /// automation and voice-control tier that reads the range widgets
    /// as numbers read prose — the gap ADR-0101's re-open rule
    /// pre-registered verbatim, cashed at the numeric tier.
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// numeric tier): a widget whose value is prose announces no number,
    /// never a parsed guess — a text field's "16 digits" is not 16, and
    /// a valueless role announces nothing. The value tier's controls
    /// override this; every other role inherits the default.
    fn numeric_value(&self) -> Option<f64> {
        None
    }

    /// Whether the widget is a field of secret entry (M100, ADR-0104):
    /// the flag behind the constructor word
    /// [`ImePurpose::Secure`](crate::passes::ImePurpose::Secure) the
    /// IME contract has carried since M4 — the field the platform's IME
    /// already treats as a password, suppresses echo and suggestions
    /// for. The tree says the word: AccessKit's `PasswordInput` role,
    /// the declaration every platform AT announces ("secure text
    /// entry"), and the value the tree resolves for the field is the
    /// mask pass 9 builds from it — one masking character per code
    /// point, never the secret itself (AccessKit's own contract: a
    /// `PasswordInput` node's value "must only contain masking
    /// characters").
    ///
    /// The default is the honest absence: a widget that declared
    /// nothing is no secure field, and a non-input role answers nothing.
    /// Editable controls override this; every other role inherits the
    /// default.
    fn secure_entry(&self) -> bool {
        false
    }

    /// Whether a declared-secure field's value rides revealed (M102,
    /// ADR-0106) — the word behind the "show password" toggle, the
    /// flag beside the constructor word [`Self::secure_entry`]
    /// answers. AccessKit's own contract names the mapping: "when the
    /// password is revealed, use `Role::TextInput` instead" — the
    /// sink's role branch reads the ordinary editable word, and the
    /// value the tree resolves is the field's ordinary value again
    /// (the mask yielding to the reveal; the pixels and the tree
    /// telling one truth).
    ///
    /// The word is a compound: meaningful only beside a declared
    /// secret entry — a field that answered `secure_entry()` `false`
    /// answers `false` here too, whatever its internal toggle holds
    /// (the plain field's value was always the tree's read, never a
    /// secret to reveal). Editable controls override this; every
    /// other role inherits the default.
    fn secure_revealed(&self) -> bool {
        false
    }

    /// Whether the field's purpose is search (M132, ADR-0136): the
    /// word behind the search box — AccessKit's `SearchInput` role,
    /// the declaration every platform AT announces and the find
    /// conventions key off ("search, edit text"), ARIA's
    /// `role="searchbox"`. Through M131 the tree said the field's
    /// capacity (M122's multiline word), its form facts (M105-M107),
    /// its suggestion contract (M111) — and never what the field was
    /// *for*: an app wiring a search box mounted a plain
    /// `TextInput`, and the AT read "edit text" where the vocabulary
    /// says "search" — the declared-but-unreported gap, the M94
    /// pattern at the purpose tier.
    ///
    /// The word prices nothing but the word: a declared field's doors
    /// answer exactly as they always did — the edit, the selection,
    /// the IME contract unchanged (the purpose is a tree word, never
    /// an IME flavor: [`ImePurpose`](crate::passes::ImePurpose)
    /// carries the platform's composition contract, and search rides
    /// it as the ordinary text entry it is). One role per node: the
    /// tree never guesses a composite. Where the facts compete the
    /// words keep their precedence — the masked contract first (a
    /// secured field keeps the `PasswordInput` word M100 mapped, the
    /// reveal yielding it per M102), the capacity word claiming a
    /// field that declared both (a multiline search box announces
    /// `MultilineTextInput` — AccessKit's vocabulary has no
    /// composite), and the purpose word re-rolling the single-line,
    /// non-secured field — the rule the sink's role branch reads, the
    /// re-open rule the ADR pre-registers beside it.
    ///
    /// The default is the honest absence: a widget that declared
    /// nothing is no search field, and a non-input role answers
    /// nothing. Editable controls override this; every other role
    /// inherits the default.
    fn search_field(&self) -> bool {
        false
    }

    /// The single axis a scroll container declares its scrolls to ride
    /// (M98, ADR-0102) — the word the AT's arrows read before they
    /// ride: ARIA's `aria-orientation`, AccessKit's `Orientation`
    /// property. Since M89 (ADR-0093) a scroll container declares its
    /// axes at construction and every scroll door answers per axis;
    /// since M98 the tree says which.
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// orientation tier): a container that declared both axes has no
    /// single-axis word to say — ARIA's `undefined` is the spec's own
    /// word for "both axes", and the sink sets nothing rather than a
    /// wrong half. A widget that is no scroll container at all inherits
    /// the default. Scroll containers override this; every other role
    /// inherits it too.
    fn scroll_orientation(&self) -> Option<ScrollOrientation> {
        None
    }

    /// The scroll container's resolved position and extent (M101,
    /// ADR-0105): where the doors landed and the walls they clamp to —
    /// AccessKit's `ScrollY` / `ScrollYMin` / `ScrollYMax` properties
    /// (the `ScrollX` halves beside them), ARIA's scroll position
    /// arithmetic the platform ATs announce as "scrolled 200 of
    /// 0..600". Through M100 the tree said the axis word (M98,
    /// ADR-0102) and never where the container *was*: the offsets and
    /// spans the scroll doors resolve (M50–M54, ADR-0056; M89,
    /// ADR-0093) rode the actions and the trail, the
    /// declared-but-unreported gap, the M94 pattern at the scroll
    /// position tier.
    ///
    /// Each axis rides its own honesty (the ADR-0087 presence rule at
    /// the scroll position tier): an axis whose span is empty — the
    /// content fits the viewport, or the width story was never
    /// declared — carries no position, the same wall every scroll door
    /// answers `no_scrollable_content` with, never a guessed zero. A
    /// container with no live axis on either side answers `None`;
    /// a widget that is no scroll container at all inherits the
    /// default. Scroll containers override this; every other role
    /// inherits it too.
    ///
    /// The tree's currency rule is the wheel's own: the doors that pay
    /// the a11y currency (the wheel, the AT's asks, bring-into-view)
    /// merge `PAINT | A11Y` on the container, so the position
    /// re-announces the frame the offset moved; the app's own handle
    /// rides the next paid pass.
    fn scroll_position(&self) -> Option<ScrollPosition> {
        None
    }

    /// Whether the widget's value is the app's, not the user's (M105,
    /// ADR-0109): the read-only tier's declared word — AccessKit's
    /// `ReadOnly` flag, ARIA's `aria-readonly`, mapped by `alacer-a11y`
    /// in the same snapshot every other property rides. Through M104 a
    /// field could be held down (`enabled(false)`, the availability
    /// tier's wall) but the field that reads, selects, copies and
    /// walks its caret like any other while refusing only the writes
    /// had no word: the declared-but-unreported gap, the M94 pattern at
    /// the read-only tier.
    ///
    /// The word walls *user* doors, never the app's: the typing door
    /// (inserts, deletes, undo, redo), the clipboard's write chords
    /// (paste, cut), the IME's commit and the AT's write door
    /// (`SetValue` / `ReplaceSelectedText`) refuse with the named
    /// species ([`crate::inspect::KeyEditRefusal::ReadOnly`],
    /// [`crate::inspect::TextWriteRefusal::ReadOnly`]) — the ask the
    /// open door would have answered streams behind its record, the
    /// shape priced at nothing, never a munged delivered edit. The
    /// read doors answer beside the wall: focus holds, the caret
    /// walks, the selection answers, the copy chord reads. The app's
    /// own `set_value` never asked the field's permission and never
    /// will — the app writes its own field.
    ///
    /// The availability tier stays the outer one: a control that
    /// answers [`Self::enabled`] `false` and this `true` names the
    /// availability wall, the read-only word riding the tree beside it.
    ///
    /// `false` is the honest absence (the ADR-0087 presence rule at the
    /// read-only tier): an editable field declares nothing. Editable
    /// controls override this; every other role inherits the default.
    fn read_only(&self) -> bool {
        false
    }

    /// Whether the control must carry a value before the form's submit
    /// fires (M106, ADR-0110): the required tier's declared word —
    /// AccessKit's `Required` flag, ARIA's `aria-required`, mapped by
    /// `alacer-a11y` in the same snapshot every other property rides.
    /// Through M105 the tree said the read-only word and every value
    /// word beside it, and the word every form vocabulary speaks before
    /// the submit fires ("this one must be filled") had no tier: the AT
    /// announced the field, the hint, the span — and never the ask —
    /// the declared-but-unreported gap, the M94 pattern at the required
    /// tier.
    ///
    /// The word prices the word and nothing else: a required control's
    /// doors answer exactly as they always did — required is not a
    /// wall, the field stays editable, the empty submit the app's own
    /// contract (the AT reads the declaration before it asks).
    ///
    /// `false` is the honest absence (the ADR-0087 presence rule at the
    /// required tier): a control that declared nothing announces
    /// nothing, never a guessed default. The value tier's controls
    /// override this; every other role inherits the default.
    fn required(&self) -> bool {
        false
    }

    /// Whether the control's value fails the form's contract (M107,
    /// ADR-0111): the invalid tier's declared word — AccessKit's
    /// `Invalid` property, ARIA's `aria-invalid`, mapped by
    /// `alacer-a11y` in the same snapshot every other property rides,
    /// the kind ([`InvalidKind`]) riding beside the word. Through M106
    /// the tree said the form's ask (the required word) and every value
    /// word beside it, and the word every form vocabulary speaks when
    /// the value lands wrong ("this one is not valid as given") had no
    /// tier: the AT announced the ask, the hint, the span — and never
    /// the rejection — the declared-but-unreported gap, the M94 pattern
    /// at the invalid tier.
    ///
    /// The word prices the word and nothing else: an invalid control's
    /// doors answer exactly as they always did — invalid is not a wall,
    /// the value stays editable, the re-submit the app's own contract
    /// (the AT reads the rejection while the user edits, which is the
    /// whole point of the word). The kind rides beside the word:
    /// [`InvalidKind::True`] the bare rejection, [`InvalidKind::Grammar`]
    /// and [`InvalidKind::Spelling`] the prose-error kinds the AT reads
    /// natively.
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// invalid tier): a control that declared nothing announces
    /// nothing, never a guessed default. The value tier's controls
    /// override this; every other role inherits the default.
    fn invalid(&self) -> Option<InvalidKind> {
        None
    }

    /// Whether the collection member is the collection's pick (M108,
    /// ADR-0112): the selected tier's declared word — AccessKit's
    /// `Selected` property, ARIA's `aria-selected`, mapped by
    /// `alacer-a11y` in the same snapshot every other property rides.
    /// Through M107 the tree said where a member sits (the M6 table
    /// slot), the form's ask and the form's rejection beside it — and
    /// the word every collection vocabulary speaks about the member
    /// ("this one is the pick") had no tier: the AT announced the
    /// position, the name, the form facts — and never the pick — the
    /// declared-but-unreported gap, the M94 pattern at the selected
    /// tier.
    ///
    /// The word prices the word and nothing else: a selected member's
    /// doors answer exactly as they always did — the word names state,
    /// it walls nothing, the pick's press and focus routes the widget's
    /// own (a tree that refused on the word's behalf would price a
    /// contract it does not own).
    ///
    /// The tier is the honest trio (the ADR-0087 presence rule at the
    /// selected tier): `None` the member that belongs to no selection
    /// collection — the absence ARIA spells `undefined`, never a
    /// guessed `false`; `Some(false)` the member standing unpicked
    /// beside its picked siblings; `Some(true)` the pick. The
    /// collection members override this; every other role inherits the
    /// default.
    fn selected(&self) -> Option<bool> {
        None
    }

    /// Whether the collection container allows more than one pick
    /// (M109, ADR-0113): the multiselectable tier's declared word —
    /// AccessKit's `Multiselectable` property, ARIA's
    /// `aria-multiselectable`, mapped by `alacer-a11y` in the same
    /// snapshot every other property rides. Through M108 the tree said
    /// which member the collection picked (the selected tier) beside
    /// the position the M6 slot carried — and the word every collection
    /// vocabulary speaks about the container ("this one allows more
    /// than one pick") had no tier: the AT announced the picks, the
    /// positions, the names — and never the capacity — the
    /// declared-but-unreported gap, the M94 pattern at the
    /// multiselectable tier.
    ///
    /// The word prices the word and nothing else: a multiselectable
    /// container's doors answer exactly as they always did — the word
    /// names capacity, it walls nothing, the members' press and focus
    /// routes the widgets' own (a tree that refused on the word's
    /// behalf would price a contract it does not own).
    ///
    /// `false` is the honest absence (the ADR-0087 presence rule at the
    /// tier, the read-only shape): a container that declared nothing
    /// announces no capacity, never a guessed one. The collection
    /// containers override this; every other role inherits the default.
    fn multiselectable(&self) -> bool {
        false
    }

    /// Whether this container's subtree is being updated and its
    /// announcements should be withheld until it is ready (M117,
    /// ADR-0121): the busy tier's declared word — AccessKit's `Busy`
    /// property, ARIA's `aria-busy`, mapped by `alacer-a11y` in the
    /// same snapshot every other property rides. Through M116 the tree
    /// said the collection's pick, capacity, disclosure, depth, order,
    /// currency and target beside the form's ask and rejection — and
    /// the word every bulk-update vocabulary speaks about the region
    /// under construction ("this list is still loading, hold what you
    /// would announce") had no tier: the AT announced every
    /// half-settled intermediate the stream wrote into the subtree —
    /// the declared-but-unreported gap, the M94 pattern at the busy
    /// tier.
    ///
    /// The word prices the word and nothing else: a busy container's
    /// doors answer exactly as they always did — the word names the
    /// subtree's updating fact, it walls nothing, the children's press
    /// and focus routes the widgets' own (a tree that refused on the
    /// word's behalf would price a contract it does not own — the AT
    /// defers its own speech, it is never the tree's to silence).
    ///
    /// `false` is the honest absence (the ADR-0087 presence rule at the
    /// tier, the multiselectable shape): a container that declared
    /// nothing announces nothing, never a guessed "updating". The
    /// collection containers override this; every other role inherits
    /// the default.
    fn busy(&self) -> bool {
        false
    }

    /// The collection's declared item count (M119, ADR-0123): the set
    /// tier's container word — AccessKit's `SizeOfSet` property, ARIA's
    /// `aria-setsize`, mapped by `alacer-a11y` in the same snapshot
    /// every other property rides. Through M118 the tree said the
    /// container's capacity to pick (the multiselectable tier, ADR-0113)
    /// and the subtree's updating fact (the busy tier, ADR-0121) beside
    /// the member facts — and the number every collection vocabulary
    /// speaks about its own span ("500 rows") had no tier: the AT
    /// announced the picks, the places, the names — and never the size —
    /// the declared-but-unreported gap, the M94 pattern at the set tier.
    ///
    /// The word prices the word and nothing else: a sized collection's
    /// doors answer exactly as they always did — the word names the
    /// span, it walls nothing, the members' press and focus routes the
    /// widgets' own (a tree that refused on the word's behalf would
    /// price a contract it does not own).
    ///
    /// The size rides the *container* — AccessKit's own contract names
    /// the placement verbatim ("`aria-setsize` is set on each item,
    /// while this property is set on the container"), the one
    /// difference from ARIA the documentation carries, one branch with
    /// no translation. `None` is the honest absence (the ADR-0087
    /// presence rule at the tier): a collection that declared nothing
    /// announces no span, never a guessed one. The collection
    /// containers override this; every other role inherits the default.
    fn size_of_set(&self) -> Option<usize> {
        None
    }

    /// The container's declared live region (M120, ADR-0124): the live
    /// tier's declared word — AccessKit's `Live` property, ARIA's
    /// `aria-live`, mapped by `alacer-a11y` in the same snapshot every
    /// other property rides, the politeness ([`crate::announce::Politeness`]) riding
    /// beside the word. Through M119 the tree said the collection's
    /// span and the member's place beside the form facts and the
    /// updating fact — and the word every streaming vocabulary speaks
    /// about the region whose changes the AT speaks ("this region is
    /// live — announce what lands here") had no app-facing tier: the
    /// AT spoke the pipeline's own reserved region (§11.6, ADR-0024)
    /// and never a region the app declared — the declared-but-unreported
    /// gap, the M94 pattern at the live tier.
    ///
    /// The word prices the word and nothing else: a declared region's
    /// doors answer exactly as they always did — the word names the
    /// region the AT watches, it walls nothing, the containers' press
    /// and focus routes the widgets' own, and the imperative
    /// announcement channel is the channel it always was (a tree that
    /// refused on the word's behalf would price a contract it does not
    /// own — the AT decides what it speaks, it is never the tree's to
    /// silence).
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// tier, the busy shape): a container that declared nothing
    /// announces no region, never a guessed politeness. The containers
    /// override this; every other role inherits the default.
    fn live(&self) -> Option<crate::announce::Politeness> {
        None
    }

    /// Whether the collection member opens and closes (M110, ADR-0114):
    /// the expanded tier's declared word — AccessKit's `Expanded`
    /// property, ARIA's `aria-expanded`, mapped by `alacer-a11y` in the
    /// same snapshot every other property rides. Through M109 the tree
    /// said which member the collection picked (the selected tier)
    /// beside the capacity the multiselectable tier named — and the word
    /// every collection vocabulary speaks about the member that opens
    /// and closes ("this one is expanded, that one collapsed") had no
    /// tier: the AT announced the picks, the positions, the names — and
    /// never the disclosure — the declared-but-unreported gap, the M94
    /// pattern at the expanded tier.
    ///
    /// The word prices the word and nothing else: an expanded member's
    /// doors answer exactly as they always did — the word names state,
    /// it walls nothing, the open and close routes the widget's own (a
    /// tree that refused on the word's behalf would price a contract it
    /// does not own).
    ///
    /// The tier is the honest trio (the ADR-0087 presence rule at the
    /// expanded tier, the selected shape): `None` the member that opens
    /// and closes not — the absence ARIA spells `undefined`, never a
    /// guessed `false`; `Some(false)` the member standing collapsed;
    /// `Some(true)` the member standing open. AccessKit's own contract
    /// names the mapping verbatim ("setting this to `false` means the
    /// node is collapsed; omitting it means this state isn't
    /// applicable"), so the mapping is one branch with no translation.
    /// The collection members override this; every other role inherits
    /// the default.
    fn expanded(&self) -> Option<bool> {
        None
    }

    /// The suggestion contract the field's completions ride (M111,
    /// ADR-0115): the auto-complete tier's declared word — AccessKit's
    /// `AutoComplete` property, ARIA's `aria-autocomplete`, mapped by
    /// `alacer-a11y` in the same snapshot every other property rides,
    /// the kind ([`AutoCompleteKind`]) riding beside the word. Through
    /// M110 the tree said where a member sits, what a control *does*,
    /// what an empty field *hints*, the span a value rides, the form's
    /// ask and the form's rejection, the collection's pick and
    /// capacity, the member's disclosure — and the word every field
    /// vocabulary speaks about the suggestions it offers ("this one
    /// completes inline, that one lists them") had no tier: the AT
    /// announced the hint, the span, the form facts — and never the
    /// suggestion contract — the declared-but-unreported gap, the M94
    /// pattern at the auto-complete tier.
    ///
    /// The word prices the word and nothing else: an auto-completed
    /// field's doors answer exactly as they always did — the word names
    /// the suggestion contract, it walls nothing, the field stays
    /// editable (a tree that refused on the word's behalf would price a
    /// contract it does not own).
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// tier): a field that declared nothing announces no suggestion
    /// contract — ARIA's `none` the absence spelled by `None`, never a
    /// guessed mode. The editable fields override this; every other
    /// role inherits the default.
    fn auto_complete(&self) -> Option<AutoCompleteKind> {
        None
    }

    /// Whether the field takes multiple lines (M122, ADR-0126): the
    /// multiline tier's declared word — AccessKit's `Multiline` flag,
    /// ARIA's `aria-multiline`, mapped by `alacer-a11y` in the same
    /// snapshot every other property rides. Through M121 the tree said
    /// what an empty field *hints* (the placeholder tier), the form's
    /// ask and the form's rejection beside it, the field's suggestion
    /// contract, the secret entry and its reveal, the wall the read-only
    /// field holds — and the word every form-and-composer vocabulary
    /// speaks about *how much the field takes* ("this field wraps onto
    /// many lines") had no tier: the AT announced the hint, the ask, the
    /// rejection, the suggestion contract — and never the capacity — the
    /// declared-but-unreported gap, the M94 pattern at the multiline
    /// tier.
    ///
    /// The word prices the word and nothing else: a worded field's doors
    /// answer exactly as they always did — the word names the capacity,
    /// it walls nothing, the field's typing, caret, selection and submit
    /// routes the widget's own (a tree that refused on the word's behalf
    /// would price a contract it does not own — the word says the field
    /// wraps, it never changes what the field takes).
    ///
    /// `false` is the honest absence (the ADR-0087 presence rule at the
    /// tier, the read-only shape): a field that declared nothing
    /// announces no capacity, never a guessed "multiline" — the
    /// single-line field the AT read before the tier. The editable
    /// fields override this; every other role inherits the default.
    fn multiline(&self) -> bool {
        false
    }

    /// The order the sorted column holds its rows in (M112, ADR-0116):
    /// the sort direction tier's declared word — AccessKit's
    /// `SortDirection` property, ARIA's `aria-sort`, mapped by
    /// `alacer-a11y` in the same snapshot every other property rides,
    /// the kind ([`SortDirection`]) riding beside the word. Through
    /// M111 the tree said what a control *does*, what an empty field
    /// *hints*, the span a value rides, the form's ask and the form's
    /// rejection, the collection's pick, capacity and disclosure, the
    /// field's suggestion contract — and the word every sorted-column
    /// vocabulary speaks about the rows it orders ("this one sorts
    /// ascending, that one descending") had no tier: the AT announced
    /// the position the M6 slot carried, the name, the form facts — and
    /// never the sort — the declared-but-unreported gap, the M94
    /// pattern at the sort direction tier.
    ///
    /// The word prices the word and nothing else: a sorted column's
    /// doors answer exactly as they always did — the word names the
    /// order, it walls nothing, the column's press and focus routes the
    /// widget's own (a tree that refused on the word's behalf would
    /// price a contract it does not own).
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// tier): a header that declared nothing announces no order —
    /// ARIA's `none` the absence spelled by `None`, never a guessed
    /// direction. The column headers override this; every other role
    /// inherits the default.
    fn sort_direction(&self) -> Option<SortDirection> {
        None
    }

    /// The member's hierarchical depth (M113, ADR-0117): the level
    /// tier's declared word — AccessKit's `Level` property, mapped by
    /// `alacer-a11y` in the same snapshot every other property rides.
    /// Through M112 the tree said where a member sits in its grid (the
    /// M6 slot's zero-based row and column), the form's ask and the
    /// form's rejection, the collection's pick, capacity and
    /// disclosure, the field's suggestion contract, the sorted column's
    /// order — and the word every nested-collection vocabulary speaks
    /// about how deep a member stands ("this one is a level-2 row
    /// under the level-1 one") had no tier: the AT announced the grid
    /// position, the name, the form facts — and never the depth — the
    /// declared-but-unreported gap, the M94 pattern at the level tier.
    ///
    /// The word prices the word and nothing else: a nested member's
    /// doors answer exactly as they always did — the word names the
    /// depth, it walls nothing, the member's press and focus routes the
    /// widget's own (a tree that refused on the word's behalf would
    /// price a contract it does not own).
    ///
    /// The count is zero-based — AccessKit's own contract names it
    /// ("this property is zero-based", beside ARIA's one-based
    /// `aria-level` the property maps) — so the mapping is one branch
    /// with no translation, and the base difference is the
    /// documentation's, never a silent cast. `None` is the honest
    /// absence (the ADR-0087 presence rule at the tier): a member that
    /// declared nothing announces no depth — the flat row the AT read
    /// before the tier, never a guessed `0`. The collection members
    /// override this; every other role inherits the default.
    fn level(&self) -> Option<usize> {
        None
    }

    /// The member's declared place in its collection (M119, ADR-0123):
    /// the set tier's member word — AccessKit's `PositionInSet`
    /// property, ARIA's `aria-posinset`, mapped by `alacer-a11y` in the
    /// same snapshot every other property rides. Through M118 the tree
    /// said where a member sits in its grid (the M6 slot's zero-based
    /// row and column), the nested member's depth (the level tier,
    /// ADR-0117), the set's current item (ADR-0118) — and the number
    /// every collection vocabulary speaks about the member's place in
    /// the collection's own span ("row 3 of 500") had no tier: the AT
    /// announced the grid position, the depth, the name — and never the
    /// place — the declared-but-unreported gap, the M94 pattern at the
    /// set tier.
    ///
    /// The word prices the word and nothing else: a placed member's
    /// doors answer exactly as they always did — the word names the
    /// place, it walls nothing, the member's press and focus routes the
    /// widget's own (a tree that refused on the word's behalf would
    /// price a contract it does not own).
    ///
    /// The place is zero-based — AccessKit's own contract names it
    /// ("`aria-posinset` is one-based, while this property is
    /// zero-based"), the base the property maps, the difference the
    /// trait's documentation carries, never a silent cast. `None` is
    /// the honest absence (the ADR-0087 presence rule at the tier): a
    /// member that declared nothing announces no place — the unplaced
    /// row the AT read before the tier, never a guessed `0`. The
    /// collection members override this; every other role inherits the
    /// default.
    fn position_in_set(&self) -> Option<usize> {
        None
    }

    /// The set's current item (M114, ADR-0118): the current tier's
    /// declared word — AccessKit's `AriaCurrent` property, ARIA's
    /// `aria-current`, mapped by `alacer-a11y` in the same snapshot
    /// every other property rides, the kind ([`CurrentKind`]) riding
    /// beside the word. Through M113 the tree said where a member sits
    /// in its grid (the M6 slot's zero-based row and column), the
    /// form's ask and the form's rejection, the collection's pick,
    /// capacity and disclosure, the field's suggestion contract, the
    /// sorted column's order, the nested member's depth — and the word
    /// every navigation and wizard vocabulary speaks about which item
    /// the reader stands in ("this one is the page we're on, that step
    /// is where we are") had no tier: the AT announced the name, the
    /// position, the form facts — and never the currency — the
    /// declared-but-unreported gap, the M94 pattern at the current tier.
    ///
    /// The word prices the word and nothing else: a current item's
    /// doors answer exactly as they always did — the word names the
    /// currency, it walls nothing, the item's press and focus routes
    /// the widget's own (a tree that refused on the word's behalf would
    /// price a contract it does not own).
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// tier): an item that declared nothing announces no currency —
    /// ARIA's `false` (the attribute's absent default) the absence
    /// spelled by `None`, never a guessed flavor. The item widgets
    /// override this; every other role inherits the default.
    fn current(&self) -> Option<CurrentKind> {
        None
    }

    /// The target the control opens (M115, ADR-0119): the has-popup
    /// tier's declared word — AccessKit's `HasPopup` property, ARIA's
    /// `aria-haspopup`, mapped by `alacer-a11y` in the same snapshot
    /// every other property rides, the kind ([`HasPopupKind`]) riding
    /// beside the word. Through M114 the tree said what a control
    /// *does*, what an empty field *hints*, the span a value rides, the
    /// form's ask and the form's rejection, the collection's pick,
    /// capacity and disclosure, the field's suggestion contract, the
    /// sorted column's order, the nested member's depth, the set's
    /// current item — and the word every menu-and-picker vocabulary
    /// speaks about what a control opens ("this one opens a menu, that
    /// one a dialog") had no tier: the AT announced the name, the
    /// currency, the form facts — and never the target — the
    /// declared-but-unreported gap, the M94 pattern at the has-popup
    /// tier.
    ///
    /// The word prices the word and nothing else: a popup opener's
    /// doors answer exactly as they always did — the word names the
    /// target, it walls nothing, the opener's press and focus routes
    /// the widget's own (a tree that refused on the word's behalf would
    /// price a contract it does not own).
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// tier): a control that opens nothing announces no target — the
    /// attribute the AT read before the tier, never a guessed one. The
    /// opener widgets override this; every other role inherits the
    /// default.
    fn has_popup(&self) -> Option<HasPopupKind> {
        None
    }

    /// The role a custom control is *read as* (M116, ADR-0120): the
    /// role-description tier's declared word — AccessKit's
    /// `RoleDescription` property, ARIA's `aria-roledescription`, mapped
    /// by `alacer-a11y` in the same snapshot every other property rides.
    /// Through M115 the tree said what a control *does* (the description,
    /// ADR-0099), what it opens (the has-popup tier, ADR-0119), the form
    /// facts, the collection facts — and the word every custom-control
    /// vocabulary speaks about what a widget is read as ("this slider is
    /// a star rating, that canvas is a chart") had no tier: the AT
    /// announced the generic role — "slider", "button" — and never the
    /// specialized one — the declared-but-unreported gap, the M94
    /// pattern at the role-description tier.
    ///
    /// The word prices the word and nothing else: a custom control's
    /// doors answer exactly as they always did — the word names the
    /// read-as role, it walls nothing, the control's press and focus
    /// routes the widget's own (a tree that refused on the word's behalf
    /// would price a contract it does not own).
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// tier): a control that declared nothing is read as the generic
    /// role it already was — the attribute the AT read before the tier,
    /// never a guessed specialization. The custom-control widgets
    /// override this; every other role inherits the default.
    fn role_description(&self) -> Option<&str> {
        None
    }

    /// The chord the control is invoked with (M121, ADR-0125): the
    /// shortcut tier's declared word — AccessKit's `KeyboardShortcut`
    /// property, ARIA's `aria-keyshortcuts`, mapped by `alacer-a11y` in
    /// the same snapshot every other property rides. Through M120 the
    /// tree said what a control *does*, what it opens, how it is read
    /// as, the form facts, the collection facts, the declared live
    /// region — and the word every menu-and-toolbar vocabulary speaks
    /// about how a control is invoked from the keyboard ("Save —
    /// Control+S") had no tier: the keymap named its bindings to the
    /// causal trail (the M37 consult, ADR-0043) and the platform menus
    /// carried their accelerators, while the control the AT walked to
    /// announced its name and never its chord — the
    /// declared-but-unreported gap, the M94 pattern at the shortcut
    /// tier.
    ///
    /// The word prices the word and nothing else: a worded control's
    /// doors answer exactly as they always did — the word names the
    /// chord, it walls nothing, the control's press and focus routes
    /// the widget's own (a tree that refused on the word's behalf would
    /// price a contract it does not own), and the keymap's dispatch is
    /// the dispatch it always was — the word says the chord the app
    /// named, it never binds a key (the keymap seam owns the chords,
    /// ADR-0043).
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// tier): a control that declared no chord announces none — the
    /// attribute the AT read before the tier, never a guessed binding.
    /// The actionable widgets override this; every other role inherits
    /// the default.
    fn keyboard_shortcut(&self) -> Option<&str> {
        None
    }

    /// The language the widget's text is written in (M123, ADR-0127):
    /// the language tier's declared word — AccessKit's `Language`
    /// property (the BCP 47 tag the platform AT voices the text with),
    /// mapped by `alacer-a11y` in the same snapshot every other property
    /// rides. Through M122 the tree said what a control *does*, what an
    /// empty field *hints*, the field's line capacity, the form facts,
    /// the collection facts, the declared live region, the chord the
    /// control is invoked with — and the word every multilingual
    /// vocabulary speaks about *what language the text speaks* ("this
    /// label is French, that one Japanese") had no tier: the AT
    /// announced the text in whatever voice the platform guessed — the
    /// declared-but-unreported gap, the M94 pattern at the language
    /// tier.
    ///
    /// The word prices the word and nothing else: a worded widget's
    /// doors answer exactly as they always did — the word names the
    /// language, it walls nothing, the text's read routes the widget's
    /// own (a tree that refused on the word's behalf would price a
    /// contract it does not own — the word says the text's language, it
    /// never translates the text). The tag is the string the app
    /// declared — the property is a string, the ADR-0099 description
    /// precedent at the string tier, one branch with no translation.
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// tier): a widget that declared no language announces none — the
    /// text the AT voiced before the tier, never a guessed tag. The
    /// text widgets override this; every other role inherits the
    /// default.
    fn language(&self) -> Option<&str> {
        None
    }

    /// The widget's toggle state for the a11y tree (M6): checkboxes report
    /// their checked state so screen readers announce "checked"/"not
    /// checked". Non-toggle roles return `None`.
    fn toggled(&self) -> Option<ToggleState> {
        None
    }

    /// The widget's position inside a data table (M6): the table, its
    /// headers, and its cells report their geometry so the AccessKit update
    /// carries navigable grid attributes. Non-table roles return `None`.
    fn table_slot(&self) -> Option<TableSlot> {
        None
    }

    /// The widget's text selection for the a11y tree (M11, ADR-0017):
    /// editable widgets report the focused selection — anchor and caret, in
    /// UTF-16 units — so a screen reader reads the range, not just the
    /// value. `None` when the widget exposes none (not focused, composing,
    /// or a valueless role); a focused caret reports as the degenerate
    /// selection.
    fn selection(&self) -> Option<TextSelection> {
        None
    }

    /// The asks the control's door answers (M156, ADR-0161): the action
    /// tier's word — the set the AT reads before it asks, AccessKit's
    /// action set mapped by `alacer-a11y` in the same snapshot every
    /// other property rides. Through M155 the tree said what a control
    /// *is*, its name, its value, whether it is available — and the asks
    /// its door answered had no tier: the platform mapped ten AccessKit
    /// actions onto [`Self::on_a11y_action`]'s vocabulary and every canon
    /// control answered or refused each ask honestly, while the node the
    /// AT walked to advertised none of them — the
    /// declared-but-unreported gap, the M94 pattern at the action tier.
    ///
    /// The word is the door's own answer said before the ask: a control
    /// advertises exactly the asks its `on_a11y_action` consumes in the
    /// state it is in — an unavailable control advertises none (the M22
    /// rule reaching the word: a closed door is no capability), a
    /// read-only field the selection ask alone (M105). The word says the
    /// door and opens nothing: an ask still travels the route it always
    /// travelled (a tree that answered on the word's behalf would price
    /// a contract it does not own), and the door still answers each ask
    /// on its own merits — a malformed payload is still refused, still
    /// named.
    ///
    /// The empty set is the honest absence (the ADR-0087 presence rule at
    /// the tier): a control whose door answers nothing advertises none —
    /// the node the AT read before the tier, never a role-guessed action.
    /// A widget that overrides `on_a11y_action` overrides this too: the
    /// two are one contract, the word said before the ask and the answer
    /// after it, and the m156 sweep pins them equal across the canon.
    fn a11y_actions(&self) -> A11yActions {
        A11yActions::empty()
    }

    /// Handles an accessibility action targeting this widget. Returns `true`
    /// when consumed. Making widgets *actionable* completes the contract in
    /// §11.5: reachable, named, and actionable. Since M156 (ADR-0161) the
    /// tree says which asks this door answers — [`Self::a11y_actions`], the
    /// word the AT reads before it asks.
    fn on_a11y_action(&mut self, action: A11yAction) -> bool {
        let _ = action;
        false
    }

    /// Whether this widget accepts direct pointer hits (scroll containers do;
    /// decorations do not).
    fn accepts_pointer(&self) -> bool {
        true
    }

    /// The hover shape this widget declares for the pointer over its face
    /// (M152, ADR-0156): the [`crate::cursor::CursorStyle`] the platform
    /// applies while the deepest accepting hit under the pointer is this
    /// widget. The default is the platform's arrow — a widget that
    /// overrides nothing declares nothing, and the honest answer is the
    /// absence, never a guessed affordance (the ADR-5 discipline, at the
    /// cursor tier). Override for the faces the vocabulary names: a link
    /// reads as the pointer, a text field as the caret, a canvas grab
    /// handle as the open hand — the pipeline resolves the deepest hit's
    /// word and the platform applies the delta to the window.
    fn cursor_style(&self) -> crate::cursor::CursorStyle {
        crate::cursor::CursorStyle::Default
    }

    /// Whether this widget can hold keyboard focus (§7.2 pass 2 recomputes
    /// the candidates from this predicate after structure changes).
    ///
    /// The default derives from the semantic role — actionable inputs are
    /// focusable — *and* from the availability tier (M22, ADR-0028): an
    /// unavailable control is never a focus candidate, so a widget that
    /// overrides [`Self::enabled`] needs no `focusable` override of its
    /// own. Override for custom controls.
    fn focusable(&self) -> bool {
        matches!(
            self.role(),
            Role::Button
                | Role::TextInput
                | Role::CheckBox
                | Role::Slider
                | Role::Radio
                | Role::Link
                | Role::Tab
                | Role::SpinButton
                | Role::Switch
                | Role::MenuItem
        ) && self.enabled()
    }

    /// The `Any` escape hatch for in-place patching (§6.1: view diffs mutate
    /// retained widgets in place; concrete patchers downcast through this,
    /// the same discipline Masonry's `WidgetMut` machinery uses). Implement
    /// as `{ self }`.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

#[cfg(test)]
mod tests {
    use super::{A11yAction, A11yActions, ScrollUnit, TextSelection};

    /// The routing vocabulary, one well-formed ask per variant.
    fn every_ask() -> Vec<A11yAction> {
        vec![
            A11yAction::DefaultPress,
            A11yAction::Increment,
            A11yAction::Decrement,
            A11yAction::ScrollUp(ScrollUnit::Item),
            A11yAction::ScrollDown(ScrollUnit::Item),
            A11yAction::ScrollLeft(ScrollUnit::Item),
            A11yAction::ScrollRight(ScrollUnit::Item),
            A11yAction::SetValue("42".to_owned()),
            A11yAction::ReplaceSelectedText("x".to_owned()),
            A11yAction::SetTextSelection(TextSelection::new(0, 0)),
        ]
    }

    #[test]
    fn every_ask_advertises_exactly_its_own_word() {
        // M156, ADR-0161: the routing vocabulary and the tree's word are
        // one vocabulary seen twice — every ask is one kind, every kind
        // is one ask's, and no bit is reachable from two asks.
        let mut said = A11yActions::empty();
        for ask in every_ask() {
            let word = ask.advertises();
            assert_eq!(
                word.bits().count_ones(),
                1,
                "{ask:?} advertises exactly one kind"
            );
            assert!(
                !said.intersects(word),
                "{ask:?} advertises a kind no other ask does"
            );
            said |= word;
        }
        assert_eq!(
            said,
            A11yActions::all(),
            "the asks tile the vocabulary: no bit is unreachable from an ask"
        );
    }

    #[test]
    fn the_word_names_the_kind_and_never_the_payload() {
        // The lengths-not-payload rule at the tier: the word says *which
        // ask*, never the value the ask would carry.
        assert_eq!(
            A11yAction::ScrollUp(ScrollUnit::Item).advertises(),
            A11yAction::ScrollUp(ScrollUnit::Page).advertises(),
            "the unit never reaches the word"
        );
        assert_eq!(
            A11yAction::SetValue("1".to_owned()).advertises(),
            A11yAction::SetValue("a secret".to_owned()).advertises(),
            "the value never reaches the word"
        );
        assert_eq!(
            A11yAction::SetTextSelection(TextSelection::new(0, 0)).advertises(),
            A11yAction::SetTextSelection(TextSelection::new(2, 5)).advertises(),
            "the selection never reaches the word"
        );
    }
}
