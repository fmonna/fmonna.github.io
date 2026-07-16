// Runtime types for the generated content module. build.rs emits
// `$OUT_DIR/content_gen.rs` referencing these (Entry, LStr, Lang); we include it
// here so `content_gen::EXPERIENCE` etc. are ordinary statics.
//
// The bilingual model is "EN is the source of truth; FR falls back to EN when the
// FR field is empty". A `fr_draft` flag marks entries whose French text is an
// unreviewed machine-style draft (rendered with a discreet banner).

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Lang {
    En,
    Fr,
}

/// A string with an EN and FR form. `fr` may be empty (=> fall back to `en`).
#[derive(Clone, Copy, PartialEq)]
pub struct LStr {
    pub en: &'static str,
    pub fr: &'static str,
}

impl LStr {
    /// Return the string for `lang`, falling back to EN when FR is empty.
    pub fn pick(self, lang: Lang) -> &'static str {
        match lang {
            Lang::En => self.en,
            Lang::Fr => {
                if self.fr.is_empty() {
                    self.en
                } else {
                    self.fr
                }
            }
        }
    }
    pub fn fr_available(self) -> bool {
        !self.fr.is_empty()
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Entry {
    pub id: &'static str,
    pub order: i64,
    pub heading: LStr,
    pub org: &'static str,
    pub location: LStr,
    pub start: &'static str,
    pub end: &'static str, // "" => current
    pub body_en: &'static str,
    pub body_fr: &'static str, // "" => render EN
    pub fr_draft: bool,
}

impl Entry {
    pub fn body(self, lang: Lang) -> &'static str {
        match lang {
            Lang::En => self.body_en,
            Lang::Fr => {
                if self.body_fr.is_empty() {
                    self.body_en
                } else {
                    self.body_fr
                }
            }
        }
    }
    pub fn fr_available(self) -> bool {
        !self.body_fr.is_empty()
    }
}

pub mod gen {
    include!(concat!(env!("OUT_DIR"), "/content_gen.rs"));
}

pub use gen::{EDUCATION, EXPERIENCE, PROFILE, PUBLICATIONS, PUBLICATIONS_DRAFT, SKILLS, SKILLS_DRAFT};
