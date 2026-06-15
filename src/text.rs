use crate::op_builder::{RawOp, RawTape};
use crate::traits::{Raw, View};
use dioxus_core::{DynamicNode, VText};

pub trait TextNode {
    const TEXT: &'static str;
}

macro_rules! text {
    ($n:ident, $s:literal) => {
        pub struct $n;
        impl $crate::traits::Raw for $n {
            const RAW: $crate::op_builder::RawTape = {
                let mut raw = $crate::op_builder::RawTape::new();
                raw.push($crate::op_builder::RawOp::TextNode);
                raw.push($crate::op_builder::RawOp::Text($s));
                raw
            };
        }
        impl $crate::traits::View for $n {}
    };
}
pub(crate) use text;

// Dynamic text child.
pub struct Dynamic(pub String);

pub fn dynamic(s: impl Into<String>) -> Dynamic {
    Dynamic(s.into())
}

impl Raw for Dynamic {
    const RAW: RawTape = {
        let mut raw = RawTape::new();
        raw.push(RawOp::TextNode);
        raw.push(RawOp::Dyn);
        raw
    };
}

impl View for Dynamic {
    fn push(self, dynamic: &mut crate::traits::DynamicValues)
    where
        Self: Sized,
    {
        dynamic.push_dynamic_node(DynamicNode::Text(VText::new(self.0)));
    }
}
