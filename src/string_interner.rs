use std::fmt::Debug;

use crate::const_vec::ConstVec;

#[derive(Clone, Copy, Debug)]
struct Span {
    off: u16,
    len: u16,
}

#[derive(Clone, Copy)]
pub(crate) struct StringInterner<const CAP: usize> {
    blob: ConstVec<u8, CAP>,
    spans: ConstVec<Span, CAP>,
}

impl<const CAP: usize> Debug for StringInterner<CAP> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StringInterner")
            .field(
                "values",
                &self
                    .spans
                    .as_ref()
                    .iter()
                    .map(|sp| {
                        let blob = self.blob.as_ref();
                        core::str::from_utf8(
                            &blob[sp.off as usize..sp.off as usize + sp.len as usize],
                        )
                        .unwrap()
                    })
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl<const CAP: usize> StringInterner<CAP> {
    pub(crate) const fn new() -> Self {
        Self {
            blob: ConstVec::new_with_max_size(),
            spans: ConstVec::new_with_max_size(),
        }
    }

    pub(crate) const fn intern(mut self, s: &str) -> (Self, u16) {
        let sb = s.as_bytes();
        let mut k = 0;
        while k < self.spans.len() {
            let sp = self.spans.at(k);
            if sp.len as usize == sb.len() {
                let mut i = 0;
                let mut eq = true;
                while i < sb.len() {
                    if self.blob.at(sp.off as usize + i) != sb[i] {
                        eq = false;
                        break;
                    }
                    i += 1;
                }
                if eq {
                    return (self, k as u16);
                }
            }
            k += 1;
        }

        let off = self.blob.len();
        let mut i = 0;
        while i < sb.len() {
            self.blob = self.blob.push(sb[i]);
            i += 1;
        }

        let idx = self.spans.len();
        self.spans = self.spans.push(Span {
            off: off as u16,
            len: sb.len() as u16,
        });
        (self, idx as u16)
    }

    #[cfg(test)]
    pub(crate) fn str_at(&self, i: u16) -> &str {
        let sp = self.spans.as_ref()[i as usize];
        let blob = self.blob.as_ref();
        core::str::from_utf8(&blob[sp.off as usize..sp.off as usize + sp.len as usize]).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::StringInterner;

    #[test]
    fn deduplicates_strings() {
        const INTERNED: (StringInterner<16>, u16, u16) = {
            let (interner, first) = StringInterner::new().intern("div");
            let (interner, second) = interner.intern("div");
            (interner, first, second)
        };

        assert_eq!(INTERNED.1, INTERNED.2);
        assert_eq!(INTERNED.0.str_at(INTERNED.1), "div");
    }

    #[test]
    fn stores_distinct_strings() {
        const INTERNED: (StringInterner<16>, u16, u16) = {
            let (interner, first) = StringInterner::new().intern("div");
            let (interner, second) = interner.intern("span");
            (interner, first, second)
        };

        assert_ne!(INTERNED.1, INTERNED.2);
        assert_eq!(INTERNED.0.str_at(INTERNED.1), "div");
        assert_eq!(INTERNED.0.str_at(INTERNED.2), "span");
    }
}
