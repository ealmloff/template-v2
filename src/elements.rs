use std::marker::PhantomData;

use crate::op_builder::{RawOp, RawTape};
use crate::traits::{Raw, View};

pub trait TagName {
    const NAME: &'static str;
}

macro_rules! tag {
    ($n:ident, $s:literal) => {
        pub struct $n;
        impl $crate::elements::TagName for $n {
            const NAME: &'static str = $s;
        }
    };
}
pub(crate) use tag;

pub struct El<Tag, At, Ch> {
    attrs: At,
    children: Ch,
    _t: PhantomData<Tag>,
}

pub fn el<Tag>() -> El<Tag, (), ()> {
    El {
        attrs: (),
        children: (),
        _t: PhantomData,
    }
}

impl<Tag, At, Ch> El<Tag, At, Ch> {
    pub fn attr<A>(self, a: A) -> El<Tag, (At, A), Ch> {
        El {
            attrs: (self.attrs, a),
            children: self.children,
            _t: PhantomData,
        }
    }

    pub fn child<C>(self, c: C) -> El<Tag, At, (Ch, C)> {
        El {
            attrs: self.attrs,
            children: (self.children, c),
            _t: PhantomData,
        }
    }
}

impl<Tag: TagName, At: Raw, Ch: Raw> Raw for El<Tag, At, Ch> {
    const RAW: RawTape = RawTape::new()
        .push(RawOp::Open(Tag::NAME))
        .concat(At::RAW)
        .concat(Ch::RAW)
        .push(RawOp::Close);
}

impl<Tag: TagName, At: View, Ch: View> View for El<Tag, At, Ch> {
    fn push(self, _dynamic: &mut crate::traits::DynamicValues)
    where
        Self: Sized,
    {
        self.attrs.push(_dynamic);
        self.children.push(_dynamic);
    }
}
