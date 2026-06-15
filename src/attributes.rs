use std::marker::PhantomData;

use crate::op_builder::{RawOp, RawTape};
use crate::traits::{Raw, View};

pub trait AttrName {
    const NAME: &'static str;
}

macro_rules! attr {
    ($n:ident, $k:literal, $v:literal) => {
        pub struct $n;
        impl $crate::traits::Raw for $n {
            const RAW: $crate::op_builder::RawTape = $crate::op_builder::RawTape::new()
                .push($crate::op_builder::RawOp::Attr)
                .push($crate::op_builder::RawOp::Text($k))
                .push($crate::op_builder::RawOp::Text($v));
        }
        impl $crate::traits::View for $n {}
    };
}
pub(crate) use attr;

macro_rules! attr_name {
    ($n:ident, $s:literal) => {
        pub struct $n;
        impl $crate::attributes::AttrName for $n {
            const NAME: &'static str = $s;
        }
    };
}
pub(crate) use attr_name;

// Dynamic attribute value.
pub struct DynAttr<Name>(pub String, PhantomData<Name>);

pub fn attr_dyn<Name>(v: impl Into<String>) -> DynAttr<Name> {
    DynAttr(v.into(), PhantomData)
}

impl<Name: AttrName> Raw for DynAttr<Name> {
    const RAW: RawTape = RawTape::new()
        .push(RawOp::Attr)
        .push(RawOp::Text(Name::NAME))
        .push(RawOp::Dyn);
}

impl<Name: AttrName> View for DynAttr<Name> {
    fn push(self, _dynamic: &mut crate::traits::DynamicValues)
    where
        Self: Sized,
    {
    }
}
