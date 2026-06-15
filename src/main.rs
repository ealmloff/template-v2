//! Composable view builder with a compile-time flat template. Each view
//! contributes raw ops, and `drive` lowers those ops into one flat template in
//! const context.

pub mod attributes;
mod const_vec;
pub mod elements;
mod hash;
mod op_builder;
mod string_interner;
pub mod text;
pub mod traits;

use attributes::{attr, attr_dyn};
use elements::{div, h2, p, span};
use op_builder::FlatTemplate;
use text::{Dynamic, dynamic, text};
use traits::{Built, Raw, View};
attr!(CardClass, "class", "card");
attr!(BadgeClass, "class", "badge");
attr!(TitleRole, "data-role", "title");
text!(TitlePrefix, "Title: ");
text!(HelloPrefix, "Hello, ");

fn badge(content: impl View) -> impl View {
    span().attr(BadgeClass).child(content)
}
// one composable view definition, parameterized by its dynamic values
fn card(style: &str, title: &str, name: &str) -> impl View {
    div()
        .attr(CardClass)
        .attr(attr_dyn("style", style))
        .child(
            h2().attr(TitleRole)
                .child(TitlePrefix)
                .child(dynamic(title)),
        )
        .child(
            p().attr(CardClass)
                .child(HelloPrefix)
                .child(badge(dynamic(name))),
        )
}

// Generic helpers read the template off any view's type, so the call site never
// names that type.
fn template_of<V: Raw>(_: &V) -> &'static FlatTemplate {
    <V as Built>::TEMPLATE
}

fn main() {
    let v1 = card("color: crimson", "Welcome", "Ada");
    let template = template_of(&v1);
    let vnode = v1.into_vnode();
    assert_eq!(template.dyns.len(), vnode.dynamic().len());
    println!("vnode: {:?}", vnode);
}

// proof the template is a compile-time constant: drive() runs in const context
// on a named view type (no composed type spelled out).
const _: () = assert!(
    <Dynamic as Built>::TEMPLATE.dyns.len() == 1 && <Dynamic as Built>::TEMPLATE.ops.len() == 2
);
