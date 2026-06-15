use crate::op_builder::{RawOp, RawTape};
use crate::traits::{Raw, View};
use dioxus_core::Attribute;

pub trait AttrName {
    const NAME: &'static str;
}

macro_rules! attr {
    ($n:ident, $k:literal, $v:literal) => {
        pub struct $n;
        impl $crate::traits::Raw for $n {
            const RAW: $crate::op_builder::RawTape = {
                let mut raw = $crate::op_builder::RawTape::new();
                raw.push($crate::op_builder::RawOp::Attr);
                raw.push($crate::op_builder::RawOp::Text($k));
                raw.push($crate::op_builder::RawOp::Text($v));
                raw
            };
        }
        impl $crate::traits::View for $n {}
    };
}
pub(crate) use attr;

#[allow(unused_macros)]
macro_rules! attr_name {
    ($n:ident, $s:literal) => {
        pub struct $n;
        impl $crate::attributes::AttrName for $n {
            const NAME: &'static str = $s;
        }
    };
}
#[allow(unused_imports)]
pub(crate) use attr_name;

// Dynamic attribute name and value.
pub struct DynAttr {
    name: &'static str,
    value: String,
}

pub fn attr_dyn(name: &'static str, value: impl Into<String>) -> DynAttr {
    DynAttr {
        name,
        value: value.into(),
    }
}

impl Raw for DynAttr {
    const RAW: RawTape = {
        let mut raw = RawTape::new();
        raw.push(RawOp::DynAttr);
        raw
    };
}

impl View for DynAttr {
    fn push(self, dynamic: &mut crate::traits::DynamicValues)
    where
        Self: Sized,
    {
        dynamic.push_attribute(Attribute::new(self.name, self.value, None, false));
    }
}
