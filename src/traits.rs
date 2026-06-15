use std::fmt::Debug;

use crate::op_builder::{FlatTemplate, RawTape, drive};
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
            const RAW:  RawTape = RawTape::new()$(.concat($name::RAW))+;
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

pub trait Built: Raw {
    const TEMPLATE: &'static FlatTemplate = &drive(Self::RAW);
}

impl<V: Raw> Built for V {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TemplatePathSegment {
    Sibling = 0,
    Child = 1,
}

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
            if path == 0 {
                None
            } else {
                Some(path >> 1)
            }
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

#[derive(Debug)]
enum DynamicValue {
    Attributes(Box<[Attribute]>),
    DynamicNode(DynamicNode),
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

    pub fn push(&mut self, value: DynamicValue) {
        self.values.push(value);
    }

    pub fn set_key(&mut self, key: String) {
        self.key = Some(key);
    }
}

#[derive(Debug)]
pub struct VNode {
    template: &'static FlatTemplate,
    dynamic: DynamicValues,
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
