use std::fmt::Debug;

use crate::op_builder::{FlatTemplate, FlatTemplateStorage, RawTape, drive};
use dioxus_core::{Attribute, DynamicNode};

pub trait Raw {
    const RAW: RawTape;
}

impl Raw for () {
    const RAW: RawTape = RawTape::new();
}

macro_rules! impl_raw_tuple {
    ($($name:ident),+ $(,)?) => {
        impl<$($name: Raw),+> Raw for ($($name,)+) {
            const RAW: RawTape = {
                let mut raw = RawTape::new();
                $(raw.concat(&$name::RAW);)+
                raw
            };
        }
    };
}

impl_raw_tuple!(A);
impl_raw_tuple!(A, B);
impl_raw_tuple!(A, B, C);
impl_raw_tuple!(A, B, C, D);
impl_raw_tuple!(A, B, C, D, E);
impl_raw_tuple!(A, B, C, D, E, F);
impl_raw_tuple!(A, B, C, D, E, F, G);
impl_raw_tuple!(A, B, C, D, E, F, G, H);
impl_raw_tuple!(A, B, C, D, E, F, G, H, I);
impl_raw_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_raw_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_raw_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_raw_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_raw_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_raw_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_raw_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

/// Type-indexed compile-time promotion.
///
/// Implementations should build `STATIC` from const-evaluable data and borrow
/// the result so Rust promotes it to static storage.
pub trait ConstStatic<T: ?Sized + 'static> {
    const STATIC: &'static T;
}

impl<V: Raw> ConstStatic<FlatTemplateStorage> for V {
    const STATIC: &'static FlatTemplateStorage = &drive(Self::RAW);
}

impl<V: Raw> ConstStatic<FlatTemplate> for V {
    const STATIC: &'static FlatTemplate =
        &<Self as ConstStatic<FlatTemplateStorage>>::STATIC.as_template();
}

pub trait Built: Raw + ConstStatic<FlatTemplate> {
    const TEMPLATE: &'static FlatTemplate = <Self as ConstStatic<FlatTemplate>>::STATIC;
}

impl<V: Raw> Built for V {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TemplatePathSegment {
    Sibling = 0,
    Child = 1,
}

/// A compact path from a template root to a dynamic node or dynamic attribute.
///
/// Paths are encoded as a sequence of cursor moves through the static template:
/// `Child` means "move to the first child" and `Sibling` means "move to the
/// next sibling". Each move is stored as one bit in `path`, so this
/// representation can encode up to 128 traversal operations.
///
/// The 128-op budget is intended for normal authored templates, where dynamic
/// holes are usually reached after only a handful of cursor moves. Extremely
/// wide generated templates can exceed this limit, especially when many dynamic
/// holes sit late in a long sibling list.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TemplatePath {
    path: u128,
}

impl TemplatePath {
    pub const fn empty() -> Self {
        Self { path: 0 }
    }

    pub const fn next_child(self) -> Self {
        let path = self.path << 1 | TemplatePathSegment::Child as u128;
        Self { path }
    }

    pub const fn next_sibling(self) -> Self {
        let path = self.path << 1 | TemplatePathSegment::Sibling as u128;
        Self { path }
    }

    pub const fn parent(self) -> Self {
        let path = self.path >> 1;
        Self { path }
    }

    pub(crate) const fn bits(&self) -> u128 {
        self.path
    }

    pub const fn pop_front(&mut self) -> TemplatePathSegment {
        let segment = if self.path & 1 == 1 {
            TemplatePathSegment::Child
        } else {
            TemplatePathSegment::Sibling
        };
        self.path >>= 1;
        segment
    }

    pub fn iter(&self) -> impl Iterator<Item = TemplatePathSegment> {
        std::iter::successors(Some(self.path), |&path| {
            if path == 0 { None } else { Some(path >> 1) }
        })
        .map(|path| {
            if path & 1 == 1 {
                TemplatePathSegment::Child
            } else {
                TemplatePathSegment::Sibling
            }
        })
    }
}

impl Debug for TemplatePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TemplatePath")
            .field("path", &self.iter().collect::<Vec<_>>())
            .finish()
    }
}

enum DynamicValue {
    Attributes(Box<[Attribute]>),
    DynamicNode(DynamicNode),
}

impl Debug for DynamicValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Attributes(attrs) => f.debug_tuple("Attributes").field(attrs).finish(),
            Self::DynamicNode(node) => f.debug_tuple("DynamicNode").field(node).finish(),
        }
    }
}

#[derive(Debug)]
pub struct DynamicValues {
    key: Option<String>,
    values: Vec<DynamicValue>,
}

impl DynamicValues {
    pub const fn new() -> Self {
        Self {
            key: None,
            values: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub(crate) fn push_attribute(&mut self, value: Attribute) {
        self.values
            .push(DynamicValue::Attributes(Box::new([value])));
    }

    pub(crate) fn push_dynamic_node(&mut self, value: DynamicNode) {
        self.values.push(DynamicValue::DynamicNode(value));
    }

    pub fn set_key(&mut self, key: String) {
        self.key = Some(key);
    }
}

impl Default for DynamicValues {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct VNode {
    template: &'static FlatTemplate,
    dynamic: DynamicValues,
}

impl VNode {
    pub fn template(&self) -> &'static FlatTemplate {
        self.template
    }

    pub fn dynamic(&self) -> &DynamicValues {
        &self.dynamic
    }
}

// View = Raw: everything a composed view contributes to the template. Returned as
// `impl View` so call sites never spell out the nested-tuple type.
pub trait View: Raw + Built + Sized {
    fn push(self, _dynamic: &mut DynamicValues)
    where
        Self: Sized,
    {
    }

    fn into_vnode(self) -> VNode {
        let mut dynamic = DynamicValues::new();
        self.push(&mut dynamic);
        VNode {
            template: Self::TEMPLATE,
            dynamic,
        }
    }
}

impl View for () {}

macro_rules! impl_view_tuple {
    ($(($name:ident, $value:ident)),+ $(,)?) => {
        impl<$($name: View),+> View for ($($name,)+) {
            fn push(self, dynamic: &mut DynamicValues) {
                let ($($value,)+) = self;
                $($value.push(dynamic);)+
            }
        }
    };
}

impl_view_tuple!((A, a));
impl_view_tuple!((A, a), (B, b));
impl_view_tuple!((A, a), (B, b), (C, c));
impl_view_tuple!((A, a), (B, b), (C, c), (D, d));
impl_view_tuple!((A, a), (B, b), (C, c), (D, d), (E, e));
impl_view_tuple!((A, a), (B, b), (C, c), (D, d), (E, e), (F, f));
impl_view_tuple!((A, a), (B, b), (C, c), (D, d), (E, e), (F, f), (G, g));
impl_view_tuple!(
    (A, a),
    (B, b),
    (C, c),
    (D, d),
    (E, e),
    (F, f),
    (G, g),
    (H, h)
);
impl_view_tuple!(
    (A, a),
    (B, b),
    (C, c),
    (D, d),
    (E, e),
    (F, f),
    (G, g),
    (H, h),
    (I, i)
);
impl_view_tuple!(
    (A, a),
    (B, b),
    (C, c),
    (D, d),
    (E, e),
    (F, f),
    (G, g),
    (H, h),
    (I, i),
    (J, j)
);
impl_view_tuple!(
    (A, a),
    (B, b),
    (C, c),
    (D, d),
    (E, e),
    (F, f),
    (G, g),
    (H, h),
    (I, i),
    (J, j),
    (K, k)
);
impl_view_tuple!(
    (A, a),
    (B, b),
    (C, c),
    (D, d),
    (E, e),
    (F, f),
    (G, g),
    (H, h),
    (I, i),
    (J, j),
    (K, k),
    (L, l)
);
impl_view_tuple!(
    (A, a),
    (B, b),
    (C, c),
    (D, d),
    (E, e),
    (F, f),
    (G, g),
    (H, h),
    (I, i),
    (J, j),
    (K, k),
    (L, l),
    (M, m)
);
impl_view_tuple!(
    (A, a),
    (B, b),
    (C, c),
    (D, d),
    (E, e),
    (F, f),
    (G, g),
    (H, h),
    (I, i),
    (J, j),
    (K, k),
    (L, l),
    (M, m),
    (N, n)
);
impl_view_tuple!(
    (A, a),
    (B, b),
    (C, c),
    (D, d),
    (E, e),
    (F, f),
    (G, g),
    (H, h),
    (I, i),
    (J, j),
    (K, k),
    (L, l),
    (M, m),
    (N, n),
    (O, o)
);
impl_view_tuple!(
    (A, a),
    (B, b),
    (C, c),
    (D, d),
    (E, e),
    (F, f),
    (G, g),
    (H, h),
    (I, i),
    (J, j),
    (K, k),
    (L, l),
    (M, m),
    (N, n),
    (O, o),
    (P, p)
);

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus_core::{AttributeValue, DynamicNode};

    struct Div;

    impl crate::elements::TagName for Div {
        const NAME: &'static str = "div";
    }

    #[test]
    fn dynamic_text_pushes_runtime_value() {
        let vnode = crate::text::dynamic("Ada").into_vnode();

        assert_eq!(vnode.template.dyns.len(), 1);
        assert_eq!(vnode.dynamic.len(), 1);
        match &vnode.dynamic.values[0] {
            DynamicValue::DynamicNode(DynamicNode::Text(text)) => {
                assert_eq!(text.value, "Ada");
            }
            other => panic!("expected dynamic text node, got {other:?}"),
        }
    }

    #[test]
    fn dynamic_attribute_pushes_runtime_value() {
        let vnode = crate::elements::el::<Div>()
            .attr(crate::attributes::attr_dyn("style", "color: crimson"))
            .into_vnode();

        assert_eq!(vnode.template.dyns.len(), 1);
        assert_eq!(vnode.dynamic.len(), 1);
        match &vnode.dynamic.values[0] {
            DynamicValue::Attributes(attrs) => {
                assert_eq!(attrs.len(), 1);
                assert_eq!(attrs[0].name, "style");
                assert_eq!(attrs[0].namespace, None);
                assert!(!attrs[0].volatile);
                assert_eq!(
                    attrs[0].value,
                    AttributeValue::Text("color: crimson".to_string())
                );
            }
            other => panic!("expected dynamic attribute, got {other:?}"),
        }
    }

    #[test]
    fn composed_view_pushes_values_in_template_order() {
        let vnode = crate::card("color: crimson", "Welcome", "Ada").into_vnode();

        assert_eq!(vnode.template.dyns.len(), 3);
        assert_eq!(vnode.dynamic.len(), 3);
        match &vnode.dynamic.values[..] {
            [
                DynamicValue::Attributes(attrs),
                DynamicValue::DynamicNode(DynamicNode::Text(title)),
                DynamicValue::DynamicNode(DynamicNode::Text(name)),
            ] => {
                assert_eq!(attrs[0].name, "style");
                assert_eq!(
                    attrs[0].value,
                    AttributeValue::Text("color: crimson".to_string())
                );
                assert_eq!(title.value, "Welcome");
                assert_eq!(name.value, "Ada");
            }
            other => panic!("unexpected dynamic values: {other:?}"),
        }
    }
}
