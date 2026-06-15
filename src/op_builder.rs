use crate::const_vec::ConstVec;
use crate::hash::xxh64;
use crate::string_interner::{StaticStringInterner, StringInterner};
use crate::traits::TemplatePath;

// This is the largest CAP that fits in Op's 15-bit enter skip encoding.
pub(crate) const CAP: usize = 16_383;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Op(u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DecodedOp {
    /// Enter a new element
    Enter {
        /// How many ops to skip when exiting
        skip: u16,
        /// If the next op is the namespace
        ns: bool,
    },
    /// The next op is an attribute
    Attr,
    /// The next op is text
    Text,
    /// Static text value
    Static(u16),
    /// Dynamic value
    Dyn,
}

impl Op {
    const MAX_CAP: usize = CAP;
    const ENTER_MAX_CODE: u16 = 0x7fff;
    const ATTR_CODE: u16 = 0x8000;
    const TEXT_CODE: u16 = 0x8001;
    const DYN_CODE: u16 = 0x8002;
    const STATIC_BASE: u16 = 0x8003;

    const fn enter(skip: u16, ns: bool) -> Self {
        if skip as usize > Self::MAX_CAP {
            panic!("op skip exceeds packed op capacity");
        }
        Self((skip << 1) | ns as u16)
    }

    const fn attr() -> Self {
        Self(Self::ATTR_CODE)
    }

    const fn text() -> Self {
        Self(Self::TEXT_CODE)
    }

    const fn static_text(id: u16) -> Self {
        if id as usize >= Self::MAX_CAP {
            panic!("static op id exceeds packed op capacity");
        }
        Self(Self::STATIC_BASE + id)
    }

    const fn dynamic() -> Self {
        Self(Self::DYN_CODE)
    }

    const fn decode(self) -> DecodedOp {
        if self.0 <= Self::ENTER_MAX_CODE {
            DecodedOp::Enter {
                skip: self.0 >> 1,
                ns: self.0 & 1 == 1,
            }
        } else if self.0 == Self::ATTR_CODE {
            DecodedOp::Attr
        } else if self.0 == Self::TEXT_CODE {
            DecodedOp::Text
        } else if self.0 == Self::DYN_CODE {
            DecodedOp::Dyn
        } else {
            DecodedOp::Static(self.0 - Self::STATIC_BASE)
        }
    }
}

impl std::fmt::Debug for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.decode() {
            DecodedOp::Enter { skip, ns } => f
                .debug_struct("Enter")
                .field("skip", &skip)
                .field("ns", &ns)
                .finish(),
            DecodedOp::Attr => f.write_str("Attr"),
            DecodedOp::Text => f.write_str("Text"),
            DecodedOp::Static(id) => f.debug_tuple("Static").field(&id).finish(),
            DecodedOp::Dyn => f.write_str("Dyn"),
        }
    }
}

const _: () = assert!(CAP == Op::MAX_CAP);

#[derive(Clone, Copy)]
pub enum RawOp {
    Open(&'static str),
    Close,
    Attr,
    DynAttr,
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

    pub const fn push(&mut self, op: RawOp) {
        self.ops.push(op);
    }

    pub const fn concat(&mut self, o: &RawTape) {
        let mut i = 0;
        while i < o.ops.len() {
            self.ops.push(o.ops.at(i));
            i += 1;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FlatTemplate {
    pub(crate) ops: &'static [Op],
    pub(crate) strings: StaticStringInterner,
    pub(crate) dyns: &'static [TemplatePath],
    hash: u64,
}

impl FlatTemplate {
    pub const fn hash(&self) -> u64 {
        self.hash
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FlatTemplateStorage {
    pub(crate) ops: ConstVec<Op, CAP>,
    pub(crate) strings: StringInterner<CAP>,
    pub(crate) dyns: ConstVec<TemplatePath, CAP>,
    hash: u64,
}

impl FlatTemplateStorage {
    const fn empty() -> Self {
        Self {
            ops: ConstVec::new_with_max_size(),
            strings: StringInterner::new(),
            dyns: ConstVec::new_with_max_size(),
            hash: 0,
        }
    }

    pub(crate) const fn as_template(&'static self) -> FlatTemplate {
        FlatTemplate {
            ops: self.ops.as_ref(),
            strings: self.strings.as_static(),
            dyns: self.dyns.as_ref(),
            hash: self.hash,
        }
    }

    const fn push(&mut self, op: Op) {
        self.ops.push(op);
    }

    const fn push_static(&mut self, s: &str) {
        let (strings, i) = self.strings.intern(s);
        self.strings = strings;
        self.push(Op::static_text(i));
    }

    const fn push_dyn(&mut self, path: TemplatePath) {
        self.dyns.push(path);
        self.push(Op::dynamic());
    }

    const fn with_hash(mut self) -> Self {
        self.hash = self.compute_hash();
        self
    }

    const fn compute_hash(&self) -> u64 {
        let mut hash = 0u64;

        hash = xxh64(&[0xB0], hash);
        let mut i = 0;
        while i < self.ops.len() {
            hash = self.hash_op(self.ops.at(i), hash);
            i += 1;
        }

        hash = xxh64(&[0xB1], hash);
        let mut i = 0;
        while i < self.dyns.len() {
            hash = xxh64(&self.dyns.at(i).bits().to_le_bytes(), hash);
            i += 1;
        }

        hash
    }

    const fn hash_op(&self, op: Op, seed: u64) -> u64 {
        match op.decode() {
            DecodedOp::Enter { skip, ns } => {
                let mut hash = xxh64(&[0x01], seed);
                hash = xxh64(&skip.to_le_bytes(), hash);
                xxh64(&[ns as u8], hash)
            }
            DecodedOp::Attr => xxh64(&[0x02], seed),
            DecodedOp::Text => xxh64(&[0x03], seed),
            DecodedOp::Static(id) => {
                let hash = xxh64(&[0x04], seed);
                self.strings.hash_at(id, hash)
            }
            DecodedOp::Dyn => xxh64(&[0x05], seed),
        }
    }
}

impl std::hash::Hash for FlatTemplate {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for FlatTemplate {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for FlatTemplate {}

const EMPTY_TEMPLATE_STORAGE: FlatTemplateStorage = FlatTemplateStorage::empty().with_hash();

impl Default for FlatTemplate {
    fn default() -> Self {
        EMPTY_TEMPLATE_STORAGE.as_template()
    }
}

#[derive(Clone, Copy)]
enum PendingDynPath {
    None,
    Attr {
        path: TemplatePath,
        remaining_static_text: u8,
    },
    Node(TemplatePath),
}

pub(crate) const fn drive(raw: RawTape) -> FlatTemplateStorage {
    let mut t = FlatTemplateStorage::empty();
    let mut stack = [0usize; CAP];
    let mut element_paths = [TemplatePath::empty(); CAP];
    let mut next_paths = [TemplatePath::empty(); CAP];
    next_paths[0] = TemplatePath::empty().next_child();
    let mut sp = 0;
    let mut pending_dyn = PendingDynPath::None;
    let mut k = 0;
    while k < raw.ops.len() {
        match raw.ops.at(k) {
            RawOp::Open(tag) => {
                let path = next_paths[sp];
                next_paths[sp] = path.next_sibling();
                element_paths[sp] = path;
                next_paths[sp + 1] = path.next_child();
                stack[sp] = t.ops.len();
                sp += 1;
                t.push(Op::enter(0, false));
                t.push_static(tag);
            }
            RawOp::Close => {
                sp -= 1;
                let o = stack[sp];
                let skip = (t.ops.len() - o) as u16;
                if let DecodedOp::Enter { ns, .. } = t.ops.at(o).decode() {
                    t.ops.set(o, Op::enter(skip, ns));
                }
            }
            RawOp::Attr => {
                pending_dyn = PendingDynPath::Attr {
                    path: element_paths[sp - 1],
                    remaining_static_text: 2,
                };
                t.push(Op::attr());
            }
            RawOp::DynAttr => {
                t.push_dyn(element_paths[sp - 1]);
            }
            RawOp::TextNode => {
                let path = next_paths[sp];
                next_paths[sp] = path.next_sibling();
                pending_dyn = PendingDynPath::Node(path);
                t.push(Op::text());
            }
            RawOp::Text(s) => {
                pending_dyn = match pending_dyn {
                    PendingDynPath::Attr {
                        path,
                        remaining_static_text,
                    } => {
                        if remaining_static_text > 1 {
                            PendingDynPath::Attr {
                                path,
                                remaining_static_text: remaining_static_text - 1,
                            }
                        } else {
                            PendingDynPath::None
                        }
                    }
                    PendingDynPath::Node(_) => PendingDynPath::None,
                    PendingDynPath::None => PendingDynPath::None,
                };
                t.push_static(s);
            }
            RawOp::Dyn => {
                let path = match pending_dyn {
                    PendingDynPath::Attr { path, .. } | PendingDynPath::Node(path) => path,
                    PendingDynPath::None => panic!("dynamic op without a template path"),
                };
                pending_dyn = PendingDynPath::None;
                t.push_dyn(path);
            }
        }
        k += 1;
    }
    t.with_hash()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const fn simple_template_storage(text: &'static str) -> FlatTemplateStorage {
        let mut raw = RawTape::new();
        raw.push(RawOp::Open("div"));
        raw.push(RawOp::TextNode);
        raw.push(RawOp::Text(text));
        raw.push(RawOp::Close);
        drive(raw)
    }

    #[test]
    fn op_is_packed_to_two_bytes() {
        assert_eq!(size_of::<Op>(), size_of::<u16>());
    }

    #[test]
    fn cap_uses_the_largest_encodable_value() {
        assert_eq!(CAP, 16_383);
        assert_eq!(Op::enter(CAP as u16, true).0, Op::ENTER_MAX_CODE);
        assert_eq!(
            Op::static_text((CAP - 1) as u16).decode(),
            DecodedOp::Static((CAP - 1) as u16)
        );
    }

    #[test]
    fn packed_ops_decode_to_logical_variants() {
        assert_eq!(
            Op::enter(CAP as u16, true).decode(),
            DecodedOp::Enter {
                skip: CAP as u16,
                ns: true,
            }
        );
        assert_eq!(
            Op::enter(7, false).decode(),
            DecodedOp::Enter { skip: 7, ns: false }
        );
        assert_eq!(Op::attr().decode(), DecodedOp::Attr);
        assert_eq!(Op::text().decode(), DecodedOp::Text);
        assert_eq!(
            Op::static_text((CAP - 1) as u16).decode(),
            DecodedOp::Static((CAP - 1) as u16)
        );
        assert_eq!(Op::dynamic().decode(), DecodedOp::Dyn);
    }

    #[test]
    fn drive_patches_enter_skip() {
        const STORAGE: FlatTemplateStorage = {
            let mut raw = RawTape::new();
            raw.push(RawOp::Open("div"));
            raw.push(RawOp::Close);
            drive(raw)
        };
        const TEMPLATE: FlatTemplate = STORAGE.as_template();

        assert_eq!(
            TEMPLATE.ops[0].decode(),
            DecodedOp::Enter { skip: 2, ns: false }
        );
    }

    #[test]
    fn dynamic_attr_slot_does_not_staticize_the_attr_name() {
        const STORAGE: FlatTemplateStorage = {
            let mut raw = RawTape::new();
            raw.push(RawOp::Open("div"));
            raw.push(RawOp::DynAttr);
            raw.push(RawOp::Close);
            drive(raw)
        };
        const TEMPLATE: FlatTemplate = STORAGE.as_template();

        assert_eq!(TEMPLATE.dyns.len(), 1);
        assert_eq!(TEMPLATE.ops.len(), 3);
        assert_eq!(TEMPLATE.ops[2].decode(), DecodedOp::Dyn);
    }

    #[test]
    fn hash_is_const_and_content_based() {
        const FIRST_STORAGE: FlatTemplateStorage = simple_template_storage("hello");
        const SECOND_STORAGE: FlatTemplateStorage = simple_template_storage("hello");
        const DIFFERENT_STORAGE: FlatTemplateStorage = simple_template_storage("goodbye");
        const FIRST: FlatTemplate = FIRST_STORAGE.as_template();
        const SECOND: FlatTemplate = SECOND_STORAGE.as_template();
        const DIFFERENT: FlatTemplate = DIFFERENT_STORAGE.as_template();

        assert_eq!(FIRST.hash(), SECOND.hash());
        assert_eq!(FIRST, SECOND);
        assert_ne!(FIRST.hash(), DIFFERENT.hash());
        assert_ne!(FIRST, DIFFERENT);
    }

    #[test]
    fn hash_includes_dynamic_paths() {
        const FIRST_CHILD_STORAGE: FlatTemplateStorage = {
            let mut raw = RawTape::new();
            raw.push(RawOp::Open("div"));
            raw.push(RawOp::TextNode);
            raw.push(RawOp::Dyn);
            raw.push(RawOp::Close);
            drive(raw)
        };
        const SECOND_CHILD_STORAGE: FlatTemplateStorage = {
            let mut raw = RawTape::new();
            raw.push(RawOp::Open("div"));
            raw.push(RawOp::TextNode);
            raw.push(RawOp::Text("static"));
            raw.push(RawOp::TextNode);
            raw.push(RawOp::Dyn);
            raw.push(RawOp::Close);
            drive(raw)
        };
        const FIRST_CHILD: FlatTemplate = FIRST_CHILD_STORAGE.as_template();
        const SECOND_CHILD: FlatTemplate = SECOND_CHILD_STORAGE.as_template();

        assert_ne!(FIRST_CHILD.hash(), SECOND_CHILD.hash());
    }

    #[test]
    fn flat_template_uses_static_slices() {
        const STORAGE: FlatTemplateStorage = simple_template_storage("hello");
        const TEMPLATE: FlatTemplate = STORAGE.as_template();

        let ops: &'static [Op] = TEMPLATE.ops;
        let dyns: &'static [TemplatePath] = TEMPLATE.dyns;

        assert_eq!(ops.len(), 4);
        assert!(dyns.is_empty());
        assert_eq!(TEMPLATE.strings.str_at(0), "div");
        assert_eq!(TEMPLATE.strings.str_at(1), "hello");
    }
}
