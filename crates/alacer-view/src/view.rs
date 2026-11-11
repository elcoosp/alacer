//! The view tree: the ephemeral, typed description of a UI frame (§6.1).
//!
//! Views are cheap value trees produced by `rsx!` (the proc-macro, M1) or by
//! hand. The retained widget tree is never rebuilt from them: the
//! reconciler diffs view against view and applies the difference as in-place
//! widget mutations ([`crate::mount`]).

use alacer_core::color::Color;
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

/// One view node: an element name, its props, its key, its children.
///
/// `element` names a registry entry ("label", "button", "column", …). A
/// `key` makes the node identity-stable under reordering (§6.3: the
/// reconciler routes keyed children through the LIS diff).
#[derive(Debug, Clone)]
pub struct ViewNode {
    /// The stable identity under keyed reconciliation.
    pub key: Option<u64>,
    /// The registry element name.
    pub element: Cow<'static, str>,
    /// The element's properties, in declaration order.
    pub props: Vec<(Cow<'static, str>, PropValue)>,
    /// The element's children, in declaration order.
    pub children: Vec<ViewNode>,
}

impl ViewNode {
    /// An element node with no props and no children.
    #[must_use]
    pub fn element(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            key: None,
            element: name.into(),
            props: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Sets the node's reconciliation key.
    #[must_use]
    pub fn key(mut self, key: impl Into<u64>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Sets one property.
    #[must_use]
    pub fn prop(mut self, name: impl Into<Cow<'static, str>>, value: impl Into<PropValue>) -> Self {
        self.props.push((name.into(), value.into()));
        self
    }

    /// Appends one child.
    #[must_use]
    pub fn child(mut self, child: ViewNode) -> Self {
        self.children.push(child);
        self
    }

    /// Appends children from an iterator (`for` loops in `rsx!`).
    #[must_use]
    pub fn children(mut self, children: impl IntoIterator<Item = ViewNode>) -> Self {
        self.children.extend(children);
        self
    }

    /// Replaces the children with an empty list.
    pub fn clear_children(&mut self) {
        self.children.clear();
    }

    /// The prop values by name (the registry patcher's lookup form).
    #[must_use]
    pub fn props_map(&self) -> FxHashMap<&str, &PropValue> {
        self.props.iter().map(|(k, v)| (k.as_ref(), v)).collect()
    }
}

/// A shared text-carrying handler cell (the `PropValue::TextHandler` slot).
pub type SharedTextHandler = Rc<RefCell<dyn FnMut(&str)>>;
/// A shared state-carrying handler cell (the `PropValue::FlagHandler` slot).
pub type SharedFlagHandler = Rc<RefCell<dyn FnMut(bool)>>;
/// A shared value-carrying handler cell (the `PropValue::NumberHandler`
/// slot) — the value tier's change handlers (`on_change` on a slider).
pub type SharedNumberHandler = Rc<RefCell<dyn FnMut(f32)>>;

/// A property value: the small, typed vocabulary `rsx!` props compile to.
#[derive(Clone)]
pub enum PropValue {
    /// Text content (`text`, `label`, string interpolation).
    Text(String),
    /// A numeric value (`gap`, `height`, `font_size`, …).
    Number(f32),
    /// A color (`color`, panel fills).
    Color(Color),
    /// A boolean flag.
    Flag(bool),
    /// An event handler (`on_click`, …). Shared so re-renders can reuse it.
    Handler(Rc<RefCell<dyn FnMut()>>),
    /// A text-carrying handler (`on_submit`, `on_changed` — M6): the event's
    /// payload is the committed text.
    TextHandler(SharedTextHandler),
    /// A state-carrying handler (`on_toggle` — M6): the event's payload is
    /// the widget's new state.
    FlagHandler(SharedFlagHandler),
    /// A value-carrying handler (`on_change` — M32): the event's payload
    /// is the widget's new value (the funnel's f32, clamped and snapped
    /// inside the widget).
    NumberHandler(SharedNumberHandler),
    /// A reactive text binding (`binding: my_binding`) — the label re-renders
    /// when the bound signal changes, no view re-run required (§6.1).
    Binding(alacer_core::flux::RenderBinding<String>),
    /// A reactive value binding (`binding: my_f32_binding` — M32): the
    /// value tier's reported value re-renders when the bound signal
    /// changes, no view re-run required — the progress bridge's door,
    /// typed by the value it carries.
    NumberBinding(alacer_core::flux::RenderBinding<f32>),
}

impl std::fmt::Debug for PropValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(text) => f.debug_tuple("Text").field(text).finish(),
            Self::Number(n) => f.debug_tuple("Number").field(n).finish(),
            Self::Color(c) => f.debug_tuple("Color").field(c).finish(),
            Self::Flag(b) => f.debug_tuple("Flag").field(b).finish(),
            Self::Handler(_) => f.debug_tuple("Handler").field(&"<fn>").finish(),
            Self::TextHandler(_) => f.debug_tuple("TextHandler").field(&"<fn>").finish(),
            Self::FlagHandler(_) => f.debug_tuple("FlagHandler").field(&"<fn>").finish(),
            Self::NumberHandler(_) => f.debug_tuple("NumberHandler").field(&"<fn>").finish(),
            Self::Binding(_) => f.debug_tuple("Binding").field(&"<bound>").finish(),
            Self::NumberBinding(_) => f.debug_tuple("NumberBinding").field(&"<bound>").finish(),
        }
    }
}

impl PartialEq for PropValue {
    /// Handlers compare by identity, not by behavior (re-hanging the same
    /// closure is not a visual change).
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text(a), Self::Text(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::Color(a), Self::Color(b)) => a == b,
            (Self::Flag(a), Self::Flag(b)) => a == b,
            (Self::Handler(a), Self::Handler(b)) => Rc::ptr_eq(a, b),
            (Self::TextHandler(a), Self::TextHandler(b)) => Rc::ptr_eq(a, b),
            (Self::FlagHandler(a), Self::FlagHandler(b)) => Rc::ptr_eq(a, b),
            (Self::NumberHandler(a), Self::NumberHandler(b)) => Rc::ptr_eq(a, b),
            // Bindings never compare equal — nor does any other pairing:
            // a rebuild re-hangs the same binding slot, and the patcher
            // re-hangs idempotently.
            _ => false,
        }
    }
}

impl From<&str> for PropValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<String> for PropValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<f32> for PropValue {
    fn from(value: f32) -> Self {
        Self::Number(value)
    }
}

impl From<i32> for PropValue {
    fn from(value: i32) -> Self {
        Self::Number(alacer_core::num::i32_to_f32(value))
    }
}

impl From<usize> for PropValue {
    fn from(value: usize) -> Self {
        Self::Number(alacer_core::num::usize_to_f32(value))
    }
}

impl From<Color> for PropValue {
    fn from(value: Color) -> Self {
        Self::Color(value)
    }
}

impl From<bool> for PropValue {
    fn from(value: bool) -> Self {
        Self::Flag(value)
    }
}

impl From<alacer_core::flux::RenderBinding<String>> for PropValue {
    fn from(value: alacer_core::flux::RenderBinding<String>) -> Self {
        Self::Binding(value)
    }
}

impl From<alacer_core::flux::RenderBinding<f32>> for PropValue {
    fn from(value: alacer_core::flux::RenderBinding<f32>) -> Self {
        Self::NumberBinding(value)
    }
}

impl<F: FnMut() + 'static> From<F> for PropValue {
    fn from(value: F) -> Self {
        Self::Handler(Rc::new(RefCell::new(value)))
    }
}

#[cfg(test)]
mod tests {
    use super::{PropValue, ViewNode};

    #[test]
    fn builders_compose_into_the_declared_shape() {
        let tree = ViewNode::element("column")
            .prop("gap", 8.0_f32)
            .child(ViewNode::element("label").prop("text", "hello"))
            .children((0_u64..3).map(|i| {
                ViewNode::element("button")
                    .key(i)
                    .prop("text", format!("B{i}"))
            }));

        assert_eq!(tree.element, "column");
        assert_eq!(tree.children.len(), 4);
        assert_eq!(tree.children[1].key, Some(0));
        assert_eq!(tree.children[2].key, Some(1));
        assert_eq!(
            tree.children[1].props_map().get("text"),
            Some(&&PropValue::Text("B0".to_string()))
        );
    }

    #[test]
    fn prop_values_convert_from_the_usual_vocabulary() {
        assert_eq!(PropValue::from("hi"), PropValue::Text("hi".into()));
        assert_eq!(PropValue::from(12.5_f32), PropValue::Number(12.5));
        assert_eq!(PropValue::from(7_i32), PropValue::Number(7.0));
        assert_eq!(PropValue::from(true), PropValue::Flag(true));
        assert_ne!(PropValue::Text("a".into()), PropValue::Number(0.0));
    }

    #[test]
    fn handlers_compare_by_identity() {
        let a: PropValue = PropValue::from(|| {});
        let b = a.clone();
        let c: PropValue = PropValue::from(|| {});
        assert_eq!(a, b, "a clone is the same handler");
        assert_ne!(a, c, "two closures are different handlers");
    }
}
