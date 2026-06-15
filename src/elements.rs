use std::marker::PhantomData;

use crate::op_builder::{RawOp, RawTape};
use crate::traits::{Raw, View};

pub trait TagName {
    const NAME: &'static str;
}

pub trait ElementTag: TagName + Sized {
    fn new() -> El<Self, (), ()> {
        el::<Self>()
    }
}

impl<Tag: TagName> ElementTag for Tag {}

macro_rules! tag {
    ($n:ident, $s:literal) => {
        pub struct $n;
        impl $crate::elements::TagName for $n {
            const NAME: &'static str = $s;
        }
    };
}
#[allow(unused_imports)]
pub(crate) use tag;

macro_rules! element_helpers {
    ($($type:ident => $helper:ident, $name:literal;)+) => {
        $(
            tag!($type, $name);

            pub fn $helper() -> El<$type, (), ()> {
                <$type as $crate::elements::ElementTag>::new()
            }
        )+
    };
}

element_helpers! {
    Div => div, "div";
    H1 => h1, "h1";
    H2 => h2, "h2";
    H3 => h3, "h3";
    H4 => h4, "h4";
    H5 => h5, "h5";
    H6 => h6, "h6";
    P => p, "p";
    Span => span, "span";
}

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
    const RAW: RawTape = {
        let mut raw = RawTape::new();
        raw.push(RawOp::Open(Tag::NAME));
        raw.concat(&At::RAW);
        raw.concat(&Ch::RAW);
        raw.push(RawOp::Close);
        raw
    };
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
