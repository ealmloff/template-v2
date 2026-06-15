use crate::op_builder::{RawOp, RawTape};
use crate::traits::{Raw, View};

pub trait TextNode {
    const TEXT: &'static str;
}

macro_rules! text {
    ($n:ident, $s:literal) => {
        pub struct $n;
        impl $crate::traits::Raw for $n {
            const RAW: $crate::op_builder::RawTape = $crate::op_builder::RawTape::new()
                .push($crate::op_builder::RawOp::TextNode)
                .push($crate::op_builder::RawOp::Text($s));
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
    const RAW: RawTape = RawTape::new().push(RawOp::TextNode).push(RawOp::Dyn);
}

impl View for Dynamic {}
