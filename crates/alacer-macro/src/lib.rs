//! The `rsx!` view DSL compiler (§13.1, ADR-0000: the proc-macro lands with
//! M1 on top of the reconciliation algorithm) and the `Widget` derive
//! (§11.1, ADR-0096: the a11y contract's required pair generated at the
//! type).
//!
//! Grammar (M1 subset, documented in ADR-0007):
//!
//! ```text
//! rsx!    = element
//! element = ident "{" body "}"
//! body    = (prop | element | for | if | text)*      commas optional between
//! prop   = "key" ":" expr          → the node's reconciliation key
//!        | ident ":" expr          → a prop; handlers are `move` closures
//! text   = "literal" | "with {expr} interpolation"
//! for    = "for" pattern "in" expr "{" element "}"
//! if     = "if" expr "{" element "}" "else" ( if | "{" element "}" )
//! ```
//!
//! Expansion is a `::alacer_view::ViewNode` builder tree. This crate is a
//! pure leaf of the crate DAG: generated code references `::alacer_view`
//! paths at the call site (the dioxus-rsx pattern) — no internal dependency,
//! so the macro cannot drift from the view layer it compiles to.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};

/// Compiles the `rsx!` view DSL into a `::alacer_view::ViewNode` expression.
///
/// # Panics
///
/// Never panics: malformed input is reported as a compile error at the
/// offending span.
#[proc_macro]
pub fn rsx(input: TokenStream) -> TokenStream {
    let tokens: TokenStream2 = input.into();
    let expanded = compile_root(&tokens).unwrap_or_else(syn::Error::into_compile_error);
    expanded.into()
}

/// Generates the `Widget` contract's required pair —
/// the semantic role and the accessible name — plus the `Any` upcast, from
/// two `#[widget(...)]` attributes on the struct (§11.1, ADR-0096: the
/// `#[derive(Widget)]` form architecture §19 Q5 pre-registered as the
/// ergonomics option):
///
/// ```ignore
/// #[derive(Widget)]
/// #[widget(role = "button", label = "Save")]
/// struct SaveButton;
///
/// #[derive(Widget)]
/// #[widget(role = "container", label = from_children)]
/// struct Panel;
/// ```
///
/// `role` speaks `Role::name`'s vocabulary — the words the whole framework
/// prints (`button`, `container`, `text_input`, `scroll_area`, …) — and is
/// required: a derived widget never guesses a semantic. `label` is required
/// too, in one of two honest forms: a string (`LabelSource::text`), or the
/// `from_children` keyword (`LabelSource::FromChildren`, the container
/// aggregation contract Column and Table declare themselves). Every other
/// trait method rides its default, so a derived widget is exactly as honest
/// as the hand-written one beside it — reachable, named, and refused by the
/// same answers.
///
/// # Panics
///
/// Never panics: malformed input is reported as a compile error at the
/// offending span (the ADR-0025 diagnostic species — the error names what
/// stood where, and the fix rides the message).
#[proc_macro_derive(Widget, attributes(widget))]
pub fn widget(input: TokenStream) -> TokenStream {
    let tokens: TokenStream2 = input.into();
    let expanded = syn::parse2::<syn::DeriveInput>(tokens)
        .and_then(|input| expand_widget_derive(&input))
        .unwrap_or_else(syn::Error::into_compile_error);
    expanded.into()
}

/// One `#[widget(...)]` attribute's parsed answer: the role word and the
/// name form, carried with the spans the errors quote.
#[derive(Default)]
struct WidgetArgs {
    /// `role = "…"` — the variant word and the span it stood at.
    role: Option<(String, proc_macro2::Span)>,
    /// `label = "…"` or `label = from_children`, with the span it stood at.
    label: Option<(LabelForm, proc_macro2::Span)>,
}

/// The two honest name forms the derive speaks.
enum LabelForm {
    /// A fixed-text source (`LabelSource::text`).
    Text(String),
    /// The container aggregation contract (`LabelSource::FromChildren`).
    FromChildren,
}

impl Parse for WidgetArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let key: proc_macro2::Ident = input.parse().map_err(|_| {
                syn::Error::new(
                    input.span(),
                    "the `#[widget]` attribute takes `role = \"...\"` and \
                     `label = \"...\" | from_children` entries, comma-separated; \
                     help: every entry is a `key = value` pair",
                )
            })?;
            let _eq: syn::Token![=] = input.parse()?;
            match key.to_string().as_str() {
                "role" => {
                    if args.role.is_some() {
                        return Err(syn::Error::new(
                            key.span(),
                            "one `role` per widget: the semantic is declared once",
                        ));
                    }
                    let word: syn::LitStr = input.parse().map_err(|_| {
                        syn::Error::new(
                            key.span(),
                            "the role is a string literal naming `Role::name`'s \
                                 vocabulary; help: `#[widget(role = \"button\")]`",
                        )
                    })?;
                    args.role = Some((word.value(), word.span()));
                }
                "label" => {
                    if args.label.is_some() {
                        return Err(syn::Error::new(
                            key.span(),
                            "one `label` per widget: the name is declared once",
                        ));
                    }
                    if input.peek(syn::LitStr) {
                        let text: syn::LitStr = input.parse()?;
                        args.label = Some((LabelForm::Text(text.value()), text.span()));
                    } else if input.peek(syn::Ident) {
                        let keyword: proc_macro2::Ident = input.parse()?;
                        if keyword == "from_children" {
                            args.label = Some((LabelForm::FromChildren, keyword.span()));
                        } else {
                            return Err(syn::Error::new(
                                keyword.span(),
                                "the label is a string literal or the `from_children` \
                                 keyword; help: containers aggregate their descendants' \
                                 names with `#[widget(label = from_children)]`",
                            ));
                        }
                    } else {
                        return Err(syn::Error::new(
                            key.span(),
                            "the label is a string literal or the `from_children` keyword",
                        ));
                    }
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown `#[widget]` entry `{other}`; \
                             help: the attribute takes `role` and `label`"
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                let _: syn::Token![,] = input.parse()?;
            }
        }
        Ok(args)
    }
}

/// The attribute word → the [`Role`](::alacer_core::Role) variant: the
/// vocabulary is `Role::name`'s own — the words the snapshot's role field,
/// the CI walker's diagnostics, and the whole framework print.
fn role_variant(word: &str, span: proc_macro2::Span) -> syn::Result<proc_macro2::Ident> {
    let variant = match word {
        "decoration" => "Decoration",
        "container" => "Container",
        "label" => "Label",
        "button" => "Button",
        "text_input" => "TextInput",
        "scroll_area" => "ScrollArea",
        "list_item" => "ListItem",
        "list" => "List",
        "checkbox" => "CheckBox",
        "slider" => "Slider",
        "progress_indicator" => "ProgressIndicator",
        "table" => "Table",
        "table_row" => "TableRow",
        "table_cell" => "TableCell",
        "column_header" => "ColumnHeader",
        "canvas" => "Canvas",
        "radio" => "Radio",
        "radio_group" => "RadioGroup",
        "link" => "Link",
        "heading" => "Heading",
        "tab" => "Tab",
        "tab_list" => "TabList",
        "spin_button" => "SpinButton",
        // The pipeline's reserved region is never a widget's role: the
        // pipeline builds the node itself (§11.6, ADR-0024) — the refusal
        // names the rule, not just the word.
        "live_region" => {
            return Err(syn::Error::new(
                span,
                "`live_region` is never a widget's role: the pipeline builds the \
                 live region itself (ADR-0024); help: name the widget's own \
                 semantic — the vocabulary is `Role::name`'s",
            ));
        }
        other => {
            return Err(syn::Error::new(
                span,
                format!(
                    "unknown role `{other}`; help: the derive speaks `Role::name`'s \
                     vocabulary — decoration, container, label, button, text_input, \
                     scroll_area, list_item, list, checkbox, slider, \
                     progress_indicator, table, table_row, table_cell, \
                     column_header, canvas, radio, radio_group, link, heading, \
                     tab, tab_list, spin_button"
                ),
            ));
        }
    };
    Ok(proc_macro2::Ident::new(variant, span))
}

/// The derive's expansion: the required pair (`role`, `label`) and the `Any`
/// upcast in its documented `{ self }` shape — and nothing else, every other
/// method riding the trait's own default.
fn expand_widget_derive(input: &syn::DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    if !matches!(input.data, syn::Data::Struct(_)) {
        return Err(syn::Error::new(
            name.span(),
            "the Widget derive takes a struct, found an enum or union; \
             help: name the widget's fields on a struct",
        ));
    }
    if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
        return Err(syn::Error::new(
            name.span(),
            "the Widget derive takes a plain struct — no generics yet: the \
             generic impl would need bounds the attribute cannot express; \
             help: derive on the concrete type",
        ));
    }

    // Fold every `#[widget(...)]` attribute's entries; duplicates refuse
    // with the second occurrence's span (declared once, named once).
    let mut args = WidgetArgs::default();
    for attribute in &input.attrs {
        if !attribute.path().is_ident("widget") {
            continue;
        }
        let parsed: WidgetArgs = attribute.parse_args()?;
        if let Some(role) = parsed.role {
            if args.role.is_some() {
                return Err(syn::Error::new(
                    role.1,
                    "one `role` per widget: the semantic is declared once",
                ));
            }
            args.role = Some(role);
        }
        if let Some(label) = parsed.label {
            if args.label.is_some() {
                return Err(syn::Error::new(
                    label.1,
                    "one `label` per widget: the name is declared once",
                ));
            }
            args.label = Some(label);
        }
    }

    let Some((role_word, role_span)) = args.role else {
        return Err(syn::Error::new(
            name.span(),
            "the Widget derive needs the widget's semantic role: add \
             `#[widget(role = \"...\")]`; help: the vocabulary is `Role::name`'s \
             — button, container, label, text_input, scroll_area, …",
        ));
    };
    let role = role_variant(&role_word, role_span)?;

    let label = match args.label {
        Some((LabelForm::Text(text), _)) => quote! { ::alacer_core::LabelSource::text(#text) },
        Some((LabelForm::FromChildren, _)) => quote! { ::alacer_core::LabelSource::FromChildren },
        None => {
            return Err(syn::Error::new(
                name.span(),
                "the Widget derive needs the widget's accessible name: add \
                 `#[widget(label = \"...\")]`, or `#[widget(label = from_children)]` \
                 when the container's name aggregates its descendants; help: an \
                 unnamed widget is the a11y walker's offender (§11.1)",
            ));
        }
    };

    Ok(quote! {
        impl ::alacer_core::Widget for #name {
            fn role(&self) -> ::alacer_core::Role {
                ::alacer_core::Role::#role
            }
            fn label(&self) -> ::alacer_core::LabelSource {
                #label
            }
            fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
                self
            }
        }
    })
}

/// One compiled body item: a prop, a child, or accumulated text content.
enum Item {
    /// `key: expr` — handled separately from ordinary props.
    Key(syn::Expr),
    /// `name: expr`.
    Prop(proc_macro2::Ident, syn::Expr),
    /// A compiled child expression (`.child(...)` fragment).
    Child(TokenStream2),
    /// A `for` loop (`.children(...)` fragment).
    ForEach(TokenStream2),
    /// Text content: an expression evaluating to the text (plain literal or
    /// `format!` — views are cheap trees, rebuilt per frame, so interpolation
    /// is evaluated when the view re-runs, §6.1).
    Text(TokenStream2),
}

/// The element body parser: props, children, `for`, `if`, text literals.
struct Body {
    items: Vec<Item>,
}

impl Parse for Body {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            // Separators are optional in rsx!: consume them where present.
            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
                continue;
            }
            // `for` — a keyword child.
            if input.peek(syn::Token![for]) {
                input.parse::<syn::Token![for]>()?;
                let pattern = syn::Pat::parse_single(input)?;
                let _in_token: syn::Token![in] = input.parse()?;
                let iter = expr_until_brace(input)?;
                let body;
                syn::braced!(body in input);
                let parsed: Body = body.parse()?;
                let element = one_element(parsed, "a `for` body", body.span())?;
                let counter = LOOP_DEPTH.with(|cell| {
                    let mut depth = cell.borrow_mut();
                    *depth += 1;
                    format_ident!("__alacer_iter{}", *depth)
                });
                items.push(Item::ForEach(quote! { .children({
                    let #counter = #iter;
                    ::std::iter::Iterator::map(#counter, |#pattern| #element)
                        .collect::<::std::vec::Vec<::alacer_view::ViewNode>>()
                }) }));
                continue;
            }
            // `if` — a keyword child.
            if input.peek(syn::Token![if]) {
                let _if_token: syn::Token![if] = input.parse()?;
                let branch = parse_if_chain(input)?;
                items.push(Item::Child(branch));
                continue;
            }
            // A text literal child.
            if input.peek(syn::LitStr) {
                let literal: syn::LitStr = input.parse()?;
                items.push(Item::Text(interpolate(&literal)?));
                continue;
            }
            // An element: `ident { ... }` — fork-peek the brace so props
            // (`ident : expr`) fall through to the prop branch. The raw
            // element expression lands in `Item::Child`; `compile_element`
            // wraps it in `.child(...)` (the same expression serves as the
            // `for`-loop closure body, which must stay unwrapped).
            if input.peek(syn::Ident) {
                let fork = input.fork();
                let name: proc_macro2::Ident = fork.parse()?;
                if fork.peek(syn::token::Brace) {
                    let _ = input.parse::<proc_macro2::Ident>()?;
                    let body;
                    syn::braced!(body in input);
                    let parsed: Body = body.parse()?;
                    let element = compile_element(&name, parsed)?;
                    items.push(Item::Child(element));
                    continue;
                }
            }
            // A prop: `name : expr` — `key` is special. Anything that got
            // this far is a malformed body: name what stood there (the
            // ADR-0025 diagnostic species — never a bare syn parse error
            // with no rsx context).
            let name: proc_macro2::Ident = input.parse().map_err(|_| {
                let found = input.parse::<proc_macro2::TokenTree>().map_or_else(
                    |_| "the end of the macro body".to_string(),
                    |token| token.to_string(),
                );
                syn::Error::new(
                    input.span(),
                    format!(
                        "rsx! expected an element (`column {{ ... }}`), `for`, `if`, a \
                         text literal, or a `name : value` prop, found {found}"
                    ),
                )
            })?;
            let _colon: syn::Token![:] = input.parse()?;
            let expr: syn::Expr = input.parse()?;
            if name == "key" {
                items.push(Item::Key(expr));
            } else {
                items.push(Item::Prop(name, expr));
            }
        }
        Ok(Self { items })
    }
}

/// Parses `else` / `else if` chains after an `if` branch.
fn parse_if_chain(input: ParseStream<'_>) -> syn::Result<TokenStream2> {
    let cond = expr_until_brace(input)?;
    let then_body;
    syn::braced!(then_body in input);
    let then_parsed: Body = then_body.parse()?;
    let then_element = one_element(then_parsed, "an `if` branch", then_body.span())?;

    if !input.peek(syn::Token![else]) {
        return Err(syn::Error::new(
            then_body.span(),
            "`if` in rsx! needs an `else` branch (conditional absence lands with \
             scoped re-renders); write both outcomes explicitly",
        ));
    }
    let _else_token: syn::Token![else] = input.parse()?;

    if input.peek(syn::Token![if]) {
        let _nested_if: syn::Token![if] = input.parse()?;
        let nested = parse_if_chain(input)?;
        return Ok(quote! { if #cond { #then_element } else { #nested } });
    }
    let else_body;
    syn::braced!(else_body in input);
    let else_parsed: Body = else_body.parse()?;
    let else_element = one_element(else_parsed, "an `else` branch", else_body.span())?;
    Ok(quote! { if #cond { #then_element } else { #else_element } })
}

/// Collects an expression terminated by the body's brace group (`if cond {`,
/// `for … in iter {`). Token-level collection sidesteps the ambiguity where
/// syn's expression parser would read `ident { … }` as a struct literal.
fn expr_until_brace(input: ParseStream<'_>) -> syn::Result<syn::Expr> {
    let mut tokens = TokenStream2::new();
    let mut depth = 0_usize;
    while !input.is_empty() {
        if depth == 0 && input.peek(syn::token::Brace) {
            break;
        }
        let tree: proc_macro2::TokenTree = input.parse()?;
        match &tree {
            proc_macro2::TokenTree::Punct(p) if "([{".contains(p.as_char()) => depth += 1,
            proc_macro2::TokenTree::Punct(p) if ")]}".contains(p.as_char()) => depth -= 1,
            proc_macro2::TokenTree::Group(_)
            | proc_macro2::TokenTree::Punct(_)
            | proc_macro2::TokenTree::Literal(_)
            | proc_macro2::TokenTree::Ident(_) => {}
        }
        tokens.extend(std::iter::once(tree));
    }
    syn::parse2(tokens)
}

thread_local! {
    /// The `for`-loop counter id supply (unique per expansion).
    static LOOP_DEPTH: std::cell::RefCell<usize> = const { std::cell::RefCell::new(0) };
}

/// Exactly one compiled element from a `for`/`if` body (the body contract).
fn one_element(body: Body, what: &str, span: proc_macro2::Span) -> syn::Result<TokenStream2> {
    let found = body.items.len();
    let mut items = body.items;
    match (items.pop(), items.is_empty()) {
        (Some(Item::Child(element)), true) => Ok(element),
        // The ADR-0025 diagnostic species: the cardinality is named —
        // "found 0" is a missing element, "found 2" a collision — with
        // the miette-class `help:` the fix suggests.
        _ => Err(syn::Error::new(
            span,
            format!(
                "{what} must contain exactly one element, found {found}; \
                 help: a body holds one element like `label {{ ... }}` — \
                 wrap siblings in a container (column or row)"
            ),
        )),
    }
}

/// Compiles `name { items }` into the `ViewNode` builder chain.
fn compile_element(name: &proc_macro2::Ident, body: Body) -> syn::Result<TokenStream2> {
    let name_lit = name.to_string();
    let mut out = quote! { ::alacer_view::ViewNode::element(#name_lit) };

    // Text content lands on the element's `text` prop (§5.2's
    // `footer { "{remaining} items left" }` shape). Multiple parts
    // concatenate; an explicit `text:` prop plus string children is a
    // contradiction — reject it.
    let text_parts: Vec<TokenStream2> = body
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Text(expr) => Some(expr.clone()),
            _ => None,
        })
        .collect();
    let explicit_text = body
        .items
        .iter()
        .any(|item| matches!(item, Item::Prop(ident, _) if ident == "text"));
    if !text_parts.is_empty() && explicit_text {
        return Err(syn::Error::new(
            name.span(),
            "either a `text:` prop or string children, not both",
        ));
    }
    match text_parts.len() {
        0 => {}
        1 => {
            let part = &text_parts[0];
            out.extend(quote! { .prop("text", #part) });
        }
        _ => {
            let pushes = text_parts
                .iter()
                .map(|part| quote! { __text.push_str(&#part); });
            out.extend(quote! { .prop("text", {
                let mut __text = ::std::string::String::new();
                #(#pushes)*
                __text
            }) });
        }
    }

    for item in body.items {
        match item {
            Item::Key(expr) => out.extend(quote! { .key(#expr) }),
            Item::Prop(ident, expr) => {
                let prop_name = ident.to_string();
                let value = compile_prop_value(expr)?;
                out.extend(quote! { .prop(#prop_name, #value) });
            }
            Item::Child(element) => out.extend(quote! { .child(#element) }),
            Item::ForEach(fragment) => out.extend(fragment),
            Item::Text(_) => {}
        }
    }
    Ok(out)
}

/// A prop value: string literals interpolate (`text: "Clicked {n} times"` —
/// §5.2); every other expression passes through verbatim (`impl
/// Into<PropValue>` does the typing).
fn compile_prop_value(expr: syn::Expr) -> syn::Result<syn::Expr> {
    if let syn::Expr::Lit(literal) = &expr
        && let syn::Lit::Str(text) = &literal.lit
    {
        let expanded = interpolate(text)?;
        return syn::parse2(expanded);
    }
    Ok(expr)
}

/// The top-level contract: exactly one root element.
fn compile_root(tokens: &TokenStream2) -> syn::Result<TokenStream2> {
    let element: ElementRoot = syn::parse2(tokens.clone())?;
    Ok(element.expanded)
}

/// The single-root wrapper parser.
struct ElementRoot {
    expanded: TokenStream2,
}

impl Parse for ElementRoot {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let name: proc_macro2::Ident = input.parse().map_err(|_| {
            // The weak-error cleanup (ADR-0025): the discarded parse cause
            // never told the user what stood where the root should be —
            // name it, with the help text the fix suggests.
            let found = input.parse::<proc_macro2::TokenTree>().map_or_else(
                |_| "the end of the macro body".to_string(),
                |token| token.to_string(),
            );
            syn::Error::new(
                input.span(),
                format!(
                    "rsx! expects one root element, found {found}; \
                     a root is an element like `column {{ ... }}`"
                ),
            )
        })?;
        let body;
        syn::braced!(body in input);
        let parsed: Body = body.parse()?;
        if !input.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "rsx! takes exactly one root element; \
                 help: wrap siblings in a container (column or row)",
            ));
        }
        Ok(Self {
            expanded: compile_element(&name, parsed)?,
        })
    }
}

/// One string literal → a text-content expression: plain literals are string
/// literals; `{expr}` interpolations compile to `format!` (§5.2). Views are
/// cheap trees re-built per frame, so interpolation re-evaluates when the
/// view re-runs — the Dioxus model, applied to widget-world reconciliation.
fn interpolate(literal: &syn::LitStr) -> syn::Result<TokenStream2> {
    let value = literal.value();
    let mut format_string = String::new();
    let mut args: Vec<TokenStream2> = Vec::new();
    let mut rest = value.as_str();
    let mut interpolating = false;
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix("{{") {
            format_string.push_str("{{");
            rest = after;
            continue;
        }
        if let Some(after) = rest.strip_prefix("}}") {
            format_string.push_str("}}");
            rest = after;
            continue;
        }
        if let Some(after) = rest.strip_prefix('{') {
            let end = after.find('}').ok_or_else(|| {
                // The ADR-0025 diagnostic species: the error names the fix
                // the sibling escape hatch ships — `{{` is a literal brace
                // in a text literal, an unclosed `{` is a broken
                // interpolation.
                syn::Error::new(
                    literal.span(),
                    "unterminated `{` in a text literal; \
                     write `{{` for a literal brace, or close the \
                     interpolation before the literal ends",
                )
            })?;
            let inner = &after[..end];
            let expr: syn::Expr = syn::parse_str(inner).map_err(|error| {
                syn::Error::new(
                    literal.span(),
                    format!("`{inner}` inside a text literal is not an expression: {error}"),
                )
            })?;
            format_string.push_str("{}");
            args.push(quote! { #expr });
            interpolating = true;
            rest = &after[end + 1..];
            continue;
        }
        let next_brace = rest.find(['{', '}']).unwrap_or(rest.len());
        format_string.push_str(&rest[..next_brace]);
        rest = &rest[next_brace..];
    }

    if interpolating {
        let format_lit = syn::LitStr::new(&format_string, literal.span());
        Ok(quote! { ::std::format!(#format_lit, #(#args),*) })
    } else {
        // Unescape braces for the plain literal (format! semantics).
        let plain = value.replace("{{", "{").replace("}}", "}");
        Ok(quote! { #plain })
    }
}

#[cfg(test)]
mod tests {
    use super::interpolate;

    fn lit(source: &str) -> syn::LitStr {
        syn::parse_str::<syn::LitStr>(source).expect("test literal")
    }

    #[test]
    fn plain_literals_stay_plain() {
        let expanded = interpolate(&lit("\"hello\"")).expect("plain").to_string();
        assert!(expanded.contains("hello"), "got {expanded}");
        assert!(!expanded.contains("format"), "got {expanded}");
    }

    #[test]
    fn interpolation_compiles_to_a_runtime_format() {
        let expanded = interpolate(&lit("\"Clicked {count} times\""))
            .expect("interpolated")
            .to_string();
        assert!(expanded.contains("format"), "got {expanded}");
        assert!(expanded.contains("count"), "got {expanded}");
    }

    #[test]
    fn doubled_braces_are_escapes() {
        let expanded = interpolate(&lit("\"{{ literal }}\""))
            .expect("escaped")
            .to_string();
        assert!(expanded.contains("literal"), "got {expanded}");
        assert!(!expanded.contains("format"), "got {expanded}");
    }

    #[test]
    fn unterminated_interpolation_is_an_error() {
        assert!(interpolate(&lit("\"{count\"")).is_err());
    }
}

#[cfg(test)]
mod expansion_debug {
    use super::compile_root;

    fn expand(source: &str) -> String {
        let tokens: proc_macro2::TokenStream = syn::parse_str(source).expect("tokens");
        compile_root(&tokens).expect("expansion").to_string()
    }

    #[test]
    fn expansions_are_well_formed_builder_chains() {
        let nested = expand("column { gap: 8.0, label { text: \"a\" } }");
        assert!(
            nested.contains("element (\"column\")"),
            "root element compiles: {nested}"
        );
        assert!(
            nested.contains(". child ("),
            "children wrap in .child: {nested}"
        );
        // Exactly one `.child(` per declared child — no double wrapping.
        assert_eq!(nested.matches(". child (").count(), 1);

        let if_tree =
            expand("column { if show { button { text: \"a\" } } else { label { text: \"b\" } } }");
        assert_eq!(
            if_tree.matches(". child (").count(),
            1,
            "the if chain is one child"
        );
        assert!(if_tree.contains("if show"), "the condition survives");
    }
}

#[cfg(test)]
mod derive_expansion_debug {
    use super::expand_widget_derive;

    fn expand(source: &str) -> String {
        let input: syn::DeriveInput = syn::parse_str(source).expect("derive input parses");
        expand_widget_derive(&input).expect("expansion").to_string()
    }

    #[test]
    fn the_pair_expands_to_the_required_trait_methods() {
        let expanded = expand("#[widget(role = \"button\", label = \"Save\")] struct SaveButton;")
            .replace(' ', "");
        assert!(
            expanded.contains("impl::alacer_core::WidgetforSaveButton"),
            "the impl targets the named struct: {expanded}"
        );
        assert!(
            expanded.contains("->::alacer_core::Role{::alacer_core::Role::Button}"),
            "the role lands verbatim from the vocabulary: {expanded}"
        );
        assert!(
            expanded.contains(
                "->::alacer_core::LabelSource{::alacer_core::LabelSource::text(\"Save\")}"
            ),
            "a string label rides LabelSource::text: {expanded}"
        );
        assert!(
            expanded.contains("fnas_any_mut(&mutself)->&mutdyn::std::any::Any{self}"),
            "the Any hatch rides the trait's documented identity shape: {expanded}"
        );
    }

    #[test]
    fn every_role_word_maps_to_its_variant() {
        let words = [
            ("decoration", "Decoration"),
            ("container", "Container"),
            ("label", "Label"),
            ("button", "Button"),
            ("text_input", "TextInput"),
            ("scroll_area", "ScrollArea"),
            ("list_item", "ListItem"),
            ("list", "List"),
            ("checkbox", "CheckBox"),
            ("slider", "Slider"),
            ("progress_indicator", "ProgressIndicator"),
            ("table", "Table"),
            ("table_row", "TableRow"),
            ("table_cell", "TableCell"),
            ("column_header", "ColumnHeader"),
            ("canvas", "Canvas"),
            ("radio", "Radio"),
            ("radio_group", "RadioGroup"),
            ("link", "Link"),
            ("heading", "Heading"),
            ("tab", "Tab"),
            ("tab_list", "TabList"),
            ("spin_button", "SpinButton"),
        ];
        for (word, variant) in words {
            let source = format!("#[widget(role = \"{word}\", label = \"x\")] struct W;");
            let expanded = expand(&source).replace(' ', "");
            assert!(
                expanded.contains(&format!("::alacer_core::Role::{variant}")),
                "`{word}` maps to Role::{variant}: {expanded}"
            );
        }
    }

    #[test]
    fn from_children_rides_the_container_contract() {
        let expanded =
            expand("#[widget(role = \"container\", label = from_children)] struct Panel;")
                .replace(' ', "");
        assert!(
            expanded.contains("::alacer_core::LabelSource::FromChildren"),
            "the keyword emits the container aggregation contract: {expanded}"
        );
    }

    #[test]
    fn unit_structs_and_tuple_structs_take_the_pair() {
        for source in [
            "#[widget(role = \"label\", label = \"n\")] struct Unit;",
            "#[widget(role = \"label\", label = \"n\")] struct Tuple(f32);",
            "#[widget(role = \"label\", label = \"n\")] struct Named { size: f32 }",
        ] {
            let expanded = expand(source).replace(' ', "");
            assert!(
                expanded.contains("::alacer_core::Role::Label"),
                "every struct shape expands: {source}"
            );
        }
    }
}

#[cfg(test)]
mod derive_tests {
    use super::expand_widget_derive;

    fn expand(source: &str) -> Result<String, String> {
        let input: syn::DeriveInput = syn::parse_str(source).expect("derive input parses");
        expand_widget_derive(&input)
            .map(|tokens| tokens.to_string())
            .map_err(|error| error.to_string())
    }

    fn expect_err(result: Result<String, String>) -> String {
        match result {
            Ok(tokens) => panic!("expected an error, got an expansion: {tokens}"),
            Err(error) => error,
        }
    }

    #[test]
    fn a_missing_role_names_the_attribute() {
        let error = expect_err(expand("#[widget(label = \"x\")] struct W;"));
        assert!(
            error.contains("role"),
            "the error names the attribute: {error}"
        );
        assert!(error.contains("help:"), "the fix rides the error: {error}");
    }

    #[test]
    fn a_missing_label_names_the_two_honest_forms() {
        let error = expect_err(expand("#[widget(role = \"container\")] struct W;"));
        assert!(
            error.contains("label"),
            "the error names the attribute: {error}"
        );
        assert!(
            error.contains("from_children"),
            "the container form is offered: {error}"
        );
    }

    #[test]
    fn an_unknown_role_names_the_vocabulary() {
        let error = expect_err(expand(
            "#[widget(role = \"toolbar\", label = \"x\")] struct W;",
        ));
        assert!(
            error.contains("toolbar"),
            "the error quotes the word: {error}"
        );
        assert!(
            error.contains("button"),
            "the vocabulary rides the error: {error}"
        );
    }

    #[test]
    fn live_region_is_refused_with_its_reason() {
        let error = expect_err(expand(
            "#[widget(role = \"live_region\", label = \"x\")] struct W;",
        ));
        assert!(
            error.contains("live region"),
            "the reason is the ADR-0024 rule: {error}"
        );
    }

    #[test]
    fn a_duplicate_role_is_named() {
        let error = expect_err(expand(
            "#[widget(role = \"button\", label = \"x\")] #[widget(role = \"slider\")] struct W;",
        ));
        assert!(
            error.contains("one `role`"),
            "the duplicate is named: {error}"
        );
    }

    #[test]
    fn an_unknown_widget_key_is_named() {
        let error = expect_err(expand(
            "#[widget(role = \"button\", label = \"x\", paint = \"y\")] struct W;",
        ));
        assert!(
            error.contains("paint"),
            "the unknown key is quoted: {error}"
        );
    }

    #[test]
    fn a_non_string_role_names_the_species() {
        let error = expect_err(expand("#[widget(role = button, label = \"x\")] struct W;"));
        assert!(
            error.contains("string literal"),
            "the expected species is named: {error}"
        );
    }

    #[test]
    fn a_non_string_label_names_the_species() {
        let error = expect_err(expand("#[widget(role = \"button\", label = 42)] struct W;"));
        assert!(
            error.contains("string literal") || error.contains("from_children"),
            "the expected species is named: {error}"
        );
    }

    #[test]
    fn enums_and_unions_are_refused() {
        let error = expect_err(expand(
            "#[widget(role = \"button\", label = \"x\")] enum W { A }",
        ));
        assert!(error.contains("struct"), "the species is named: {error}");
    }

    #[test]
    fn generics_are_refused_with_the_limitation_named() {
        let error = expect_err(expand(
            "#[widget(role = \"button\", label = \"x\")] struct W<T> { value: T }",
        ));
        assert!(
            error.contains("generic"),
            "the limitation is named, not stumbled into: {error}"
        );
    }
}
