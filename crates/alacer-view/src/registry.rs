//! The element registry: view element names → retained widget factories and
//! patchers (§6.1: view diff emits widget mutations; widgets are never
//! re-created).
//!
//! The machinery lives here (the foundation layer names no widget types);
//! concrete element sets are registered by crates that own widgets — the
//! M0 set ships as `alacer_widgets::default_registry`. Applications add
//! their own elements with [`WidgetRegistry::register`]: typed closures, one
//! downcast per entry, none at the call site.

use crate::view::{PropValue, ViewNode};
use alacer_core::color::Color;
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::rc::Rc;

/// The property list of one view node, with lookup helpers (the patcher's
/// view of [`ViewNode::props`]).
pub struct ElementProps<'a> {
    props: FxHashMap<&'a str, &'a PropValue>,
}

impl<'a> ElementProps<'a> {
    /// Builds from a view node (call sites pass the node being created or
    /// patched).
    #[must_use]
    pub fn of(node: &'a ViewNode) -> Self {
        Self {
            props: node.props_map(),
        }
    }

    /// A text prop.
    #[must_use]
    pub fn text(&self, name: &str) -> Option<&str> {
        match self.props.get(name)? {
            PropValue::Text(text) => Some(text),
            _ => None,
        }
    }

    /// A numeric prop.
    #[must_use]
    pub fn number(&self, name: &str) -> Option<f32> {
        match self.props.get(name)? {
            PropValue::Number(number) => Some(*number),
            _ => None,
        }
    }

    /// A color prop.
    #[must_use]
    pub fn color(&self, name: &str) -> Option<Color> {
        match self.props.get(name)? {
            PropValue::Color(color) => Some(*color),
            _ => None,
        }
    }

    /// A boolean prop.
    #[must_use]
    pub fn flag(&self, name: &str) -> Option<bool> {
        match self.props.get(name)? {
            PropValue::Flag(flag) => Some(*flag),
            _ => None,
        }
    }

    /// A reactive text-binding prop (`binding: my_binding`).
    #[must_use]
    pub fn binding(&self) -> Option<alacer_core::flux::RenderBinding<String>> {
        match self.props.get("binding")? {
            crate::view::PropValue::Binding(binding) => Some(binding.clone()),
            _ => None,
        }
    }

    /// A handler prop, wrapped for a widget's `Box<dyn FnMut()>` slot: the
    /// shared cell is called through `RefCell` so re-renders re-hang the same
    /// handler identity without moving out of the `Rc`.
    #[must_use]
    pub fn handler(&self, name: &str) -> Option<Box<dyn FnMut()>> {
        match self.props.get(name)? {
            PropValue::Handler(handler) => {
                let handler = Rc::clone(handler);
                Some(Box::new(move || (handler.borrow_mut())()))
            }
            _ => None,
        }
    }

    /// A text-carrying handler prop (`on_submit`, `on_changed` — M6): the
    /// widget calls it with the committed text.
    #[must_use]
    pub fn text_handler(&self, name: &str) -> Option<BoxedTextHandler> {
        match self.props.get(name)? {
            PropValue::TextHandler(handler) => {
                let handler = Rc::clone(handler);
                Some(Box::new(move |text: &str| (handler.borrow_mut())(text)))
            }
            _ => None,
        }
    }

    /// A state-carrying handler prop (`on_toggle` — M6): the widget calls it
    /// with its new state.
    #[must_use]
    pub fn flag_handler(&self, name: &str) -> Option<BoxedFlagHandler> {
        match self.props.get(name)? {
            PropValue::FlagHandler(handler) => {
                let handler = Rc::clone(handler);
                Some(Box::new(move |checked: bool| {
                    (handler.borrow_mut())(checked);
                }))
            }
            _ => None,
        }
    }
}

/// A boxed text-carrying handler, ready to hang on a widget.
pub type BoxedTextHandler = Box<dyn FnMut(&str)>;
/// A boxed state-carrying handler, ready to hang on a widget.
pub type BoxedFlagHandler = Box<dyn FnMut(bool)>;

/// A boxed widget factory over a view node.
type CreateFn = Box<dyn Fn(&ViewNode) -> Box<dyn alacer_core::Widget>>;
/// A boxed in-place patcher over a retained widget; returns whether a
/// visual prop applied.
type PatchFn = Box<dyn Fn(&mut dyn alacer_core::Widget, &ViewNode) -> bool>;

/// One registry entry: create a widget from props, patch one in place.
pub(crate) struct ElementEntry {
    pub(crate) create: CreateFn,
    pub(crate) patch: PatchFn,
}

/// The element registry: names → typed create/patch entries.
#[derive(Default)]
pub struct WidgetRegistry {
    elements: FxHashMap<Cow<'static, str>, Rc<ElementEntry>>,
}

impl WidgetRegistry {
    /// An empty registry — register elements with [`Self::register`], or
    /// start from `alacer_widgets::default_registry()`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers (or replaces) an element under `name`.
    ///
    /// `create` builds a fresh widget from a node's props; `patch` mutates a
    /// retained one in place (returning whether a *visual* prop applied —
    /// handlers are not visual). The patch closure downcasts once through
    /// [`alacer_core::Widget::as_any_mut`], inside the entry.
    pub fn register<W: alacer_core::Widget>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        create: fn(&ElementProps<'_>) -> W,
        patch: fn(&mut W, &ElementProps<'_>) -> bool,
    ) {
        let entry = ElementEntry {
            create: Box::new(move |node| Box::new(create(&ElementProps::of(node)))),
            patch: Box::new(move |widget, node| {
                widget
                    .as_any_mut()
                    .downcast_mut::<W>()
                    .is_some_and(|typed| patch(typed, &ElementProps::of(node)))
            }),
        };
        self.elements.insert(name.into(), Rc::new(entry));
    }

    /// Whether `name` is registered.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.elements.contains_key(name)
    }

    /// Creates the widget for `node`'s element.
    ///
    /// # Panics
    ///
    /// Panics naming the unknown element: an `rsx!` typo must fail loudly at
    /// mount, not silently produce an empty widget.
    #[must_use]
    pub fn create(&self, node: &ViewNode) -> Box<dyn alacer_core::Widget> {
        (self.entry(&node.element).create)(node)
    }

    /// Patches `widget` (known to be `node`'s element's widget) in place.
    /// Returns whether any visual prop was applied.
    pub fn patch(&self, widget: &mut dyn alacer_core::Widget, node: &ViewNode) -> bool {
        (self.entry(&node.element).patch)(widget, node)
    }

    fn entry(&self, name: &str) -> &Rc<ElementEntry> {
        self.elements.get(name).unwrap_or_else(|| {
            panic!(
                "unknown element {name:?} — register it in the WidgetRegistry \
                 (the M0 set ships as `alacer_widgets::default_registry`)"
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::WidgetRegistry;
    use crate::view::ViewNode;

    /// A minimal probe widget (the machinery test needs no widget crate).
    struct TextProbe {
        text: String,
    }

    impl alacer_core::Widget for TextProbe {
        fn role(&self) -> alacer_core::Role {
            alacer_core::Role::Label
        }

        fn label(&self) -> alacer_core::LabelSource {
            alacer_core::LabelSource::text(self.text.clone())
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    impl TextProbe {
        fn set_text(&mut self, text: &str) {
            self.text = text.to_string();
        }
    }

    #[test]
    fn unknown_elements_fail_loudly_at_create() {
        let registry = WidgetRegistry::new();
        let node = ViewNode::element("video");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = registry.create(&node);
        }));
        assert!(result.is_err(), "an unregistered element must panic");
        assert!(!registry.contains("video"));
    }

    #[test]
    fn registered_elements_create_and_patch_in_place() {
        fn create(props: &super::ElementProps<'_>) -> TextProbe {
            TextProbe {
                text: props.text("text").unwrap_or_default().to_string(),
            }
        }

        fn patch(probe: &mut TextProbe, props: &super::ElementProps<'_>) -> bool {
            match props.text("text") {
                Some(text) => {
                    probe.set_text(text);
                    true
                }
                None => false,
            }
        }

        let mut registry = WidgetRegistry::new();
        registry.register("probe", create, patch);
        assert!(registry.contains("probe"));

        let node = ViewNode::element("probe").prop("text", "before");
        let mut widget = registry.create(&node);
        assert_eq!(widget.label().resolve(), "before");

        let updated = ViewNode::element("probe").prop("text", "after");
        assert!(registry.patch(widget.as_mut(), &updated));
        assert_eq!(widget.label().resolve(), "after", "patched in place");
    }

    #[test]
    fn patch_reports_non_visual_changes_as_silent() {
        fn create(_props: &super::ElementProps<'_>) -> TextProbe {
            TextProbe {
                text: String::new(),
            }
        }

        fn patch(_probe: &mut TextProbe, _props: &super::ElementProps<'_>) -> bool {
            false // only handlers were set
        }

        let mut registry = WidgetRegistry::new();
        registry.register("probe", create, patch);
        let node = ViewNode::element("probe").prop("on_click", || {});
        let mut widget = registry.create(&node);
        let updated = ViewNode::element("probe").prop("on_click", || {});
        assert!(
            !registry.patch(widget.as_mut(), &updated),
            "handler-only patches are silent"
        );
    }
}
