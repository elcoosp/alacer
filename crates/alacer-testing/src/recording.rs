//! The record/replay format (§13.4, ADR-0012): user interactions recorded
//! to `.alc` files and replayed in `cargo test` — bug reports that are also
//! regression tests (Iced's `.ice` precedent, adopted as ALGO).
//!
//! Steps are **semantic by design**: a `press` names a role and an
//! accessible name, not a pixel, so a recording survives layout drift that
//! a coordinate click would not. Replay routes through the harness's real
//! input methods — the same pipeline, the same settle discipline — so a
//! replayed recording *is* the regression test.

use alacer_core::Role;
use alacer_core::events::Key;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The recording format version. Bumping it is a breaking change for
/// committed recordings; the replay refuses unknown schemas loudly.
pub const RECORDING_SCHEMA: u32 = 1;

/// The `serde_json` failure behind a recording surface: the `io::Error`'s
/// `Display` names the surface ("malformed .alc"), and — because the
/// custom repr delegates `source()` to the boxed error, and this wrapper
/// delegates to its cause — the `io::Error`'s own `source()` reaches the
/// real parse or serialize error (ADR-0023, the chain `PlatformError`
/// walks, one seam over; the replay's panic-on-drift semantics are a
/// separate, deliberate contract). The wrapper stays private on purpose:
/// the `io::Error` is the surface type, the chain is the diagnostic.
#[derive(Debug)]
struct RecordingCause {
    surface: &'static str,
    cause: serde_json::Error,
}

impl std::fmt::Display for RecordingCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.surface, self.cause)
    }
}

impl std::error::Error for RecordingCause {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}

/// One recorded interaction step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Step {
    /// The default a11y action ("press") on the first node with `role`
    /// named `name` — the semantic click.
    Press {
        /// The target's semantic role (e.g. `"checkbox"`).
        role: String,
        /// Substring of the target's accessible name.
        name: String,
    },
    /// A typed-text burst into the focused widget (character presses).
    TypeText {
        /// The characters, in order.
        text: String,
    },
    /// One special key press to the focused widget (enter, tab, space, …).
    PressKey {
        /// The key's logical name (see [`key_name`]).
        key: String,
    },
    /// A wheel event at an absolute position.
    Wheel {
        /// The pointer position, `[x, y]`.
        position: [f32; 2],
        /// The wheel delta, `[dx, dy]`.
        delta: [f32; 2],
    },
    /// A raw click at an absolute position (the escape hatch for
    /// decoration-level interactions).
    Click {
        /// The click position, `[x, y]`.
        position: [f32; 2],
    },
}

/// A recorded session: the header (format, viewport, when) plus the ordered
/// steps. Save it as `<name>.alc`; load it anywhere and replay.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Recording {
    /// The format version ([`RECORDING_SCHEMA`]).
    pub schema: u32,
    /// The harness viewport the session ran on, `[width, height]`.
    pub viewport: [f32; 2],
    /// When the session was recorded (unix seconds; metadata, never load-
    /// bearing).
    pub created_unix: u64,
    /// The steps, in order.
    pub steps: Vec<Step>,
}

/// A recording session: runs interactions through the harness and logs the
/// semantic steps in one pass — what you did is what gets saved.
pub struct Session<'h> {
    harness: &'h mut TestHarness,
    steps: Vec<Step>,
}

impl<'h> Session<'h> {
    /// Begins a recording session on `harness`.
    #[must_use]
    pub fn begin(harness: &'h mut TestHarness) -> Self {
        Self {
            harness,
            steps: Vec::new(),
        }
    }

    /// The default a11y action on `role` named `name` (recorded + executed).
    pub fn press(&mut self, role: Role, name: &str) {
        self.steps.push(Step::Press {
            role: role.name().to_string(),
            name: name.to_string(),
        });
        self.harness.press(role, name);
    }

    /// Types `text` into the focused widget (recorded + executed).
    pub fn type_text(&mut self, text: &str) {
        self.steps.push(Step::TypeText {
            text: text.to_string(),
        });
        self.harness.type_text(text);
    }

    /// Presses one special key (recorded + executed).
    pub fn press_key(&mut self, key: Key) {
        self.steps.push(Step::PressKey {
            key: key_name(&key),
        });
        self.harness.press_key(key);
    }

    /// Wheels at `position` by `delta` (recorded + executed).
    pub fn wheel(&mut self, position: alacer_core::geom::Point, delta: (f32, f32)) {
        self.steps.push(Step::Wheel {
            position: [position.x, position.y],
            delta: [delta.0, delta.1],
        });
        self.harness.wheel(position, delta);
    }

    /// Clicks at `position` (recorded + executed).
    pub fn click(&mut self, position: alacer_core::geom::Point) {
        self.steps.push(Step::Click {
            position: [position.x, position.y],
        });
        self.harness.click(position);
    }

    /// Ends the session: the recording (viewport and timestamp filled in).
    #[must_use]
    pub fn finish(self) -> Recording {
        Recording {
            schema: RECORDING_SCHEMA,
            viewport: [
                self.harness.viewport().width,
                self.harness.viewport().height,
            ],
            created_unix: unix_seconds(),
            steps: self.steps,
        }
    }
}

impl Recording {
    /// A fresh, empty recording for `viewport` (programmatic construction).
    #[must_use]
    pub fn new(viewport: (f32, f32)) -> Self {
        Self {
            schema: RECORDING_SCHEMA,
            viewport: [viewport.0, viewport.1],
            created_unix: unix_seconds(),
            steps: Vec::new(),
        }
    }

    /// Appends a step (programmatic construction).
    pub fn push(&mut self, step: Step) {
        self.steps.push(step);
    }

    /// Saves the recording as `.alc` (versioned JSON).
    ///
    /// # Errors
    ///
    /// Propagates I/O and serialization errors to the caller.
    pub fn save(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(|error| {
            std::io::Error::other(RecordingCause {
                surface: "recording serialization failed",
                cause: error,
            })
        })?;
        std::fs::write(path, json)
    }

    /// Loads a `.alc` recording.
    ///
    /// # Errors
    ///
    /// Errors on unknown schemas, unreadable files, and malformed JSON — a
    /// recording that cannot be replayed honestly must not silently pass.
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let recording: Recording = serde_json::from_str(&text).map_err(|error| {
            std::io::Error::other(RecordingCause {
                surface: "malformed .alc",
                cause: error,
            })
        })?;
        if recording.schema != RECORDING_SCHEMA {
            return Err(std::io::Error::other(format!(
                "unsupported recording schema {} (this harness speaks {RECORDING_SCHEMA})",
                recording.schema
            )));
        }
        Ok(recording)
    }

    /// Replays every step on `harness`, in order, through the real input
    /// methods (each step settles — the idle contract unchanged). After each
    /// step, `on_step` runs — the hook apps with an explicit re-render seam
    /// need (the M6 canon's scoped re-render rides exactly this hook).
    ///
    /// # Panics
    ///
    /// Panics when a semantic step cannot resolve (role/name parse failure
    /// or a missing a11y node): a recording that no longer describes reality
    /// is a failing regression test, by design.
    pub fn replay_with(
        &self,
        harness: &mut TestHarness,
        on_step: &mut dyn FnMut(&mut TestHarness, &Step),
    ) {
        self.check_viewport(harness);
        for step in &self.steps {
            match step {
                Step::Press { role, name } => {
                    let role = role_from_name(role)
                        .unwrap_or_else(|| panic!("recording names an unknown role {role:?}"));
                    harness.press(role, name);
                }
                Step::TypeText { text } => harness.type_text(text),
                Step::PressKey { key } => {
                    let key = key_from_name(key)
                        .unwrap_or_else(|| panic!("recording names an unknown key {key:?}"));
                    harness.press_key(key);
                }
                Step::Wheel { position, delta } => harness.wheel(
                    alacer_core::geom::Point::new(position[0], position[1]),
                    (delta[0], delta[1]),
                ),
                Step::Click { position } => {
                    harness.click(alacer_core::geom::Point::new(position[0], position[1]));
                }
            }
            on_step(harness, step);
        }
    }

    /// Replays every step with no per-step hook (the pure input path).
    ///
    /// # Panics
    ///
    /// Same as [`Self::replay_with`].
    pub fn replay(&self, harness: &mut TestHarness) {
        self.replay_with(harness, &mut |_, _| {});
    }

    /// The viewport determinism guard (shared by both replay entry points).
    fn check_viewport(&self, harness: &TestHarness) {
        assert_eq!(
            harness.viewport().width,
            self.viewport[0],
            "recording viewport width does not match the harness"
        );
        assert_eq!(
            harness.viewport().height,
            self.viewport[1],
            "recording viewport height does not match the harness"
        );
    }

    /// The ordered steps (test introspection).
    #[must_use]
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }
}

/// A key's stable recording name (`"enter"`, `"space"`, `"a"`, …).
#[must_use]
pub fn key_name(key: &Key) -> String {
    match key {
        Key::Char(ch) => ch.to_string(),
        Key::Enter => "enter".to_string(),
        Key::Backspace => "backspace".to_string(),
        Key::Delete => "delete".to_string(),
        Key::Tab => "tab".to_string(),
        Key::Space => "space".to_string(),
        Key::Escape => "escape".to_string(),
        Key::ArrowLeft => "arrow_left".to_string(),
        Key::ArrowRight => "arrow_right".to_string(),
        Key::ArrowUp => "arrow_up".to_string(),
        Key::ArrowDown => "arrow_down".to_string(),
        Key::Home => "home".to_string(),
        Key::End => "end".to_string(),
        Key::Unknown => "unknown".to_string(),
    }
}

/// Parses a recording key name back onto [`Key`].
#[must_use]
pub fn key_from_name(name: &str) -> Option<Key> {
    match name {
        "enter" => Some(Key::Enter),
        "backspace" => Some(Key::Backspace),
        "delete" => Some(Key::Delete),
        "tab" => Some(Key::Tab),
        "space" => Some(Key::Space),
        "escape" => Some(Key::Escape),
        "arrow_left" => Some(Key::ArrowLeft),
        "arrow_right" => Some(Key::ArrowRight),
        "arrow_up" => Some(Key::ArrowUp),
        "arrow_down" => Some(Key::ArrowDown),
        "home" => Some(Key::Home),
        "end" => Some(Key::End),
        "unknown" => Some(Key::Unknown),
        other => {
            let mut chars = other.chars();
            let (ch, rest) = (chars.next(), chars.as_str());
            if rest.is_empty() {
                Some(Key::Char(ch?))
            } else {
                None
            }
        }
    }
}

/// Parses a recording role name back onto [`Role`].
#[must_use]
pub fn role_from_name(name: &str) -> Option<Role> {
    match name {
        "decoration" => Some(Role::Decoration),
        "container" => Some(Role::Container),
        "label" => Some(Role::Label),
        "button" => Some(Role::Button),
        "text_input" => Some(Role::TextInput),
        "scroll_area" => Some(Role::ScrollArea),
        "list_item" => Some(Role::ListItem),
        "list" => Some(Role::List),
        "checkbox" => Some(Role::CheckBox),
        "table" => Some(Role::Table),
        "table_row" => Some(Role::TableRow),
        "table_cell" => Some(Role::TableCell),
        "column_header" => Some(Role::ColumnHeader),
        _ => None,
    }
}

fn unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

use crate::TestHarness;

#[cfg(test)]
mod tests {
    use super::{Recording, Session, Step, key_from_name, key_name, role_from_name};
    use alacer_core::Role;
    use alacer_core::events::Key;
    use alacer_widgets::BoxedWidget as _;

    #[test]
    fn keys_round_trip_through_recording_names() {
        for key in [
            Key::Enter,
            Key::Backspace,
            Key::Delete,
            Key::Tab,
            Key::Space,
            Key::Escape,
            Key::ArrowLeft,
            Key::ArrowRight,
            Key::ArrowUp,
            Key::ArrowDown,
            Key::Home,
            Key::End,
            Key::Char('x'),
            Key::Char('é'),
            Key::Unknown,
        ] {
            assert_eq!(
                key_from_name(&key_name(&key)),
                Some(key),
                "{key:?} round-trips"
            );
        }
    }

    #[test]
    fn roles_round_trip_through_recording_names() {
        for role in [
            Role::CheckBox,
            Role::Button,
            Role::TextInput,
            Role::Table,
            Role::TableCell,
            Role::ColumnHeader,
            Role::TableRow,
        ] {
            assert_eq!(
                role_from_name(role.name()),
                Some(role),
                "{role:?} round-trips"
            );
        }
        assert_eq!(role_from_name("video"), None);
    }

    #[test]
    fn recordings_round_trip_through_files() {
        let mut recording = Recording::new((320.0, 240.0));
        recording.push(Step::Press {
            role: "checkbox".to_string(),
            name: "Ship in bulk".to_string(),
        });
        recording.push(Step::TypeText {
            text: "héllo".to_string(),
        });
        recording.push(Step::PressKey {
            key: "enter".to_string(),
        });
        recording.push(Step::Wheel {
            position: [40.0, 60.0],
            delta: [0.0, -30.0],
        });

        let path = std::env::temp_dir().join("alacer_recording_roundtrip.alc");
        recording.save(&path).expect("save");
        let loaded = Recording::load(&path).expect("load");
        let _ = std::fs::remove_file(&path);

        assert_eq!(loaded, recording, "the loaded recording is byte-identical");
        assert_eq!(loaded.schema, super::RECORDING_SCHEMA);
        assert_eq!(loaded.viewport, [320.0, 240.0]);
        assert_eq!(loaded.steps().len(), 4);
    }

    #[test]
    fn a_malformed_recording_walks_to_its_parse_cause() {
        // ADR-0023: a malformed `.alc` is a *bug report* (ADR-10) — the
        // load path's `source()` chain must reach the real parse error,
        // the way `PlatformError`'s chain reaches the event-loop error
        // (ADR-0022). The message names the surface; the chain carries it.
        let path = std::env::temp_dir().join("alacer_recording_malformed.alc");
        std::fs::write(&path, "{ not json").expect("write");
        let error = Recording::load(&path).expect_err("malformed json refuses");
        let _ = std::fs::remove_file(&path);

        assert!(
            error.to_string().contains("malformed .alc"),
            "the message names the surface: {error}"
        );
        let cause = std::error::Error::source(&error)
            .and_then(|cause| cause.downcast_ref::<serde_json::Error>())
            .expect("the parse cause is reachable as the error's source");
        assert!(
            cause.is_syntax(),
            "the cause is the parse error itself: {cause}"
        );
    }

    #[test]
    fn unknown_schemas_refuse_to_load() {
        let path = std::env::temp_dir().join("alacer_recording_bad_schema.alc");
        std::fs::write(
            &path,
            r#"{"schema": 99, "viewport": [1,1], "created_unix": 0, "steps": []}"#,
        )
        .expect("write");
        let error = Recording::load(&path).expect_err("schema mismatch refuses");
        let _ = std::fs::remove_file(&path);
        assert!(
            error.to_string().contains("unsupported recording schema"),
            "the error names the problem: {error}"
        );
    }

    #[test]
    fn a_recorded_session_round_trips_and_replays() {
        // A checkbox-bearing tree; the session records a semantic press.
        let mut harness = crate::TestHarness::new(320.0, 240.0);
        let toggles = std::rc::Rc::new(std::cell::RefCell::new(0_u32));
        let sink = toggles.clone();
        let _ = harness.mount(
            alacer_widgets::Checkbox::new("Ship in bulk")
                .on_toggle(move |checked| {
                    if checked {
                        *sink.borrow_mut() += 1;
                    } else {
                        *sink.borrow_mut() -= 1;
                    }
                })
                .boxed(),
        );
        harness.settle();

        let mut session = Session::begin(&mut harness);
        session.press(Role::CheckBox, "Ship in bulk");
        let recording = session.finish();
        assert_eq!(recording.steps().len(), 1, "the press is recorded");
        assert_eq!(*toggles.borrow(), 1, "the live interaction ran");

        // Save → load → replay on a fresh, identically-assembled harness.
        let path = std::env::temp_dir().join("alacer_session_replay.alc");
        recording.save(&path).expect("save");
        let loaded = Recording::load(&path).expect("load");
        let _ = std::fs::remove_file(&path);
        assert_eq!(loaded, recording, "round-trip is exact");

        let mut replay = crate::TestHarness::new(320.0, 240.0);
        let sink = toggles.clone();
        let _ = replay.mount(
            alacer_widgets::Checkbox::new("Ship in bulk")
                .on_toggle(move |checked| {
                    if checked {
                        *sink.borrow_mut() += 1;
                    } else {
                        *sink.borrow_mut() -= 1;
                    }
                })
                .boxed(),
        );
        replay.settle();
        loaded.replay(&mut replay);
        assert_eq!(*toggles.borrow(), 2, "the replayed press toggled again");
    }
}
