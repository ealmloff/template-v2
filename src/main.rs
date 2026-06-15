//! Composable view builder with a compile-time flat template. Each view
//! contributes raw ops, and `drive` lowers those ops into one flat template in
//! const context.

pub mod attributes;
mod const_vec;
pub mod elements;
mod op_builder;
mod string_interner;
pub mod text;
pub mod traits;

use attributes::{attr, attr_dyn, attr_name};
use elements::{el, tag};
use op_builder::FlatTemplate;
use text::{Dynamic, dynamic, text};
use traits::{Built, Raw, View};
tag!(Div, "div");
tag!(H2, "h2");
tag!(P, "p");
tag!(SpanTag, "span");
attr!(CardClass, "class", "card");
attr!(BadgeClass, "class", "badge");
attr!(TitleRole, "data-role", "title");
text!(TitlePrefix, "Title: ");
text!(HelloPrefix, "Hello, ");
attr_name!(StyleName, "style");

struct StaticTemplate {}

fn badge(content: impl View) -> impl View {
    el::<SpanTag>().attr(BadgeClass).child(content)
}
// one composable view definition, parameterized by its dynamic values
fn card(style: &str, title: &str, name: &str) -> impl View {
    el::<Div>()
        .attr(CardClass)
        .attr(attr_dyn::<StyleName>(style))
        .child(
            el::<H2>()
                .attr(TitleRole)
                .child(TitlePrefix)
                .child(dynamic(title)),
        )
        .child(
            el::<P>()
                .attr(CardClass)
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
    let vnode = v1.into_vnode();
    println!("vnode: {:?}", vnode);
}

// proof the template is a compile-time constant: drive() runs in const context
// on a named view type (no composed type spelled out).
const _: () = assert!(
    <Dynamic as Built>::TEMPLATE.dyns.len() == 1 && <Dynamic as Built>::TEMPLATE.ops.len() == 2
);
