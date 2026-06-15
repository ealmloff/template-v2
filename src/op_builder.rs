use crate::const_vec::ConstVec;
use crate::string_interner::StringInterner;
use crate::traits::TemplatePath;

pub(crate) const CAP: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
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
    Dyn(u16),
}

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

    pub const fn concat(mut self, o: RawTape) -> Self {
        let mut i = 0;
        while i < o.ops.len() {
            self.ops = self.ops.push(o.ops.at(i));
            i += 1;
        }
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FlatTemplate {
    pub(crate) ops: ConstVec<Op, CAP>,
    pub(crate) strings: StringInterner<CAP>,
    pub(crate) dyns: ConstVec<TemplatePath, CAP>,
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

    const fn push_dyn(mut self, path: TemplatePath) -> Self {
        let id = self.dyns.len() as u16;
        self.dyns = self.dyns.push(path);
        self.push(Op::Dyn(id))
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

pub(crate) const fn drive(raw: RawTape) -> FlatTemplate {
    let mut t = FlatTemplate::empty();
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
                t = t.push(Op::Enter { skip: 0, ns: false });
                t = t.push_static(tag);
            }
            RawOp::Close => {
                sp -= 1;
                let o = stack[sp];
                let skip = (t.ops.len() - o) as u16;
                if let Op::Enter { ns, .. } = t.ops.at(o) {
                    t.ops = t.ops.set(o, Op::Enter { skip, ns });
                }
            }
            RawOp::Attr => {
                pending_dyn = PendingDynPath::Attr {
                    path: element_paths[sp - 1],
                    remaining_static_text: 2,
                };
                t = t.push(Op::Attr);
            }
            RawOp::TextNode => {
                let path = next_paths[sp];
                next_paths[sp] = path.next_sibling();
                pending_dyn = PendingDynPath::Node(path);
                t = t.push(Op::Text);
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
                t = t.push_static(s);
            }
            RawOp::Dyn => {
                let path = match pending_dyn {
                    PendingDynPath::Attr { path, .. } | PendingDynPath::Node(path) => path,
                    PendingDynPath::None => panic!("dynamic op without a template path"),
                };
                pending_dyn = PendingDynPath::None;
                t = t.push_dyn(path);
            }
        }
        k += 1;
    }
    t
}
