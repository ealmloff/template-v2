//! Composable view builder with a compile-time flat template. Each view
//! contributes raw ops, and `drive` lowers those ops into one flat template in
//! const context.

use std::marker::PhantomData;

mod const_vec;
mod string_interner;

use const_vec::ConstVec;
use string_interner::StringInterner;

// ===== efficient flat op template ==========================================
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    Enter { skip: u16, ns: bool },
    Exit,
    Attr,
    Text,
    Static(u16),
    Dyn(u16),
}
const CAP: usize = 64;
#[derive(Clone, Copy)]
pub enum RawOp {
    Open(&'static str),
    Close,
    Attr,
    TextNode,
    Text(&'static str),
    Dyn,
}
#[derive(Clone, Copy)]
pub struct RawTape {
    ops: ConstVec<RawOp, CAP>,
}
impl RawTape {
    pub const fn new() -> Self {
        Self {
            ops: ConstVec::new_with_max_size(),
        }
    }
    pub const fn push(mut self, op: RawOp) -> Self {
        self.ops = self.ops.push(op);
        self
    }
    pub const fn concat(mut self, o: &RawTape) -> Self {
        let mut i = 0;
        while i < o.ops.len() {
            self.ops = self.ops.push(o.ops.at(i));
            i += 1;
        }
        self
    }
}

#[derive(Clone, Copy)]
pub struct FlatTemplate {
    ops: ConstVec<Op, CAP>,
    strings: StringInterner<CAP>,
    dyns: ConstVec<u16, CAP>,
}
impl FlatTemplate {
    const fn empty() -> Self {
        Self {
            ops: ConstVec::new_with_max_size(),
            strings: StringInterner::new(),
            dyns: ConstVec::new_with_max_size(),
        }
    }
    const fn push(mut self, op: Op) -> Self {
        self.ops = self.ops.push(op);
        self
    }
    const fn push_static(mut self, s: &str) -> Self {
        let (strings, i) = self.strings.intern(s);
        self.strings = strings;
        self.push(Op::Static(i))
    }
    const fn push_dyn(mut self) -> Self {
        let id = self.dyns.len() as u16;
        self.dyns = self.dyns.push(self.ops.len() as u16);
        self.push(Op::Dyn(id))
    }
}
const fn drive(raw: &RawTape) -> FlatTemplate {
    let mut t = FlatTemplate::empty();
    let mut stack = [0usize; CAP];
    let mut sp = 0;
    let mut k = 0;
    while k < raw.ops.len() {
        match raw.ops.at(k) {
            RawOp::Open(tag) => {
                stack[sp] = t.ops.len();
                sp += 1;
                t = t.push(Op::Enter { skip: 0, ns: false });
                t = t.push_static(tag);
            }
            RawOp::Close => {
                t = t.push(Op::Exit);
                sp -= 1;
                let o = stack[sp];
                let skip = (t.ops.len() - o) as u16;
                if let Op::Enter { ns, .. } = t.ops.at(o) {
                    t.ops = t.ops.set(o, Op::Enter { skip, ns });
                }
            }
            RawOp::Attr => {
                t = t.push(Op::Attr);
            }
            RawOp::TextNode => {
                t = t.push(Op::Text);
            }
            RawOp::Text(s) => {
                t = t.push_static(s);
            }
            RawOp::Dyn => {
                t = t.push_dyn();
            }
        }
        k += 1;
    }
    t
}

// ===== the three composable halves =========================================
pub trait Raw {
    const RAW: RawTape;
}
pub trait Built {
    const TEMPLATE: FlatTemplate;
}
impl<V: Raw> Built for V {
    const TEMPLATE: FlatTemplate = drive(&V::RAW);
}

// ----- leaves --------------------------------------------------------------
impl Raw for () {
    const RAW: RawTape = RawTape::new();
}

macro_rules! text {
    ($n:ident, $s:literal) => {
        pub struct $n;
        impl Raw for $n {
            const RAW: RawTape = RawTape::new().push(RawOp::TextNode).push(RawOp::Text($s));
        }
    };
}
macro_rules! tag {
    ($n:ident, $s:literal) => {
        pub struct $n;
        impl TagName for $n {
            const NAME: &'static str = $s;
        }
    };
}
macro_rules! attr {
    ($n:ident, $k:literal, $v:literal) => {
        pub struct $n;
        impl Raw for $n {
            const RAW: RawTape = RawTape::new()
                .push(RawOp::Attr)
                .push(RawOp::Text($k))
                .push(RawOp::Text($v));
        }
    };
}
macro_rules! attr_name {
    ($n:ident, $s:literal) => {
        pub struct $n;
        impl AttrName for $n {
            const NAME: &'static str = $s;
        }
    };
}
pub trait TextNode {
    const TEXT: &'static str;
}
pub trait TagName {
    const NAME: &'static str;
}
pub trait AttrName {
    const NAME: &'static str;
}

// dynamic text child
pub struct Dynamic(pub String);
pub fn dynamic(s: impl Into<String>) -> Dynamic {
    Dynamic(s.into())
}
impl Raw for Dynamic {
    const RAW: RawTape = RawTape::new().push(RawOp::TextNode).push(RawOp::Dyn);
}

// dynamic attribute value
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

// ----- tuples --------------------------------------------------------------
impl<A: Raw, B: Raw> Raw for (A, B) {
    const RAW: RawTape = A::RAW.concat(&B::RAW);
}

// ----- element + builder ---------------------------------------------------
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
        .concat(&At::RAW)
        .concat(&Ch::RAW)
        .push(RawOp::Close);
}
// ===========================================================================
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

struct StaticTemplate {

}

struct DynamicValues {
    attributes: Vec<String>,
}

// View = Raw: everything a composed view contributes to the template. Returned as
// `impl View` so call sites never spell out the nested-tuple type.
pub trait View: Raw {
    fn push_values(self, values: &mut Vec<String>);
}
impl<T: Raw> View for T {}

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
fn template_of<V: Raw>(_: &V) -> FlatTemplate {
    <V as Built>::TEMPLATE
}

fn main() {
    let v1 = card("color: crimson", "Welcome", "Ada");
    let t = template_of(&v1);
    println!(
        "template: {} ops, holes at {:?}",
        t.ops.len(),
        t.dyns.as_ref()
    );
}

// proof the template is a compile-time constant: drive() runs in const context
// on a named view type (no composed type spelled out).
const _: () = assert!(
    <Dynamic as Built>::TEMPLATE.dyns.len() == 1 && <Dynamic as Built>::TEMPLATE.ops.len() == 2
);
