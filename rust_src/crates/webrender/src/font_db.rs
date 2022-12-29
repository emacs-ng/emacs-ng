use fontdb::{FaceInfo, Family, Query, Stretch, Style, Weight};

use std::str;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FontDescriptor {
    PostScript(String),
    Properties {
        family: String,
        weight: Weight,
        slant: Style,
        stretch: Stretch,
    },
}

pub struct FontDB {
    pub db: fontdb::Database,
}

impl FontDB {
    pub fn new() -> FontDB {
        let mut db = fontdb::Database::new();

        db.load_system_fonts();

        FontDB { db }
    }

    pub fn select_postscript(&self, postscript_name: &str) -> Option<&FaceInfo> {
        self.db
            .faces()
            .iter()
            .filter(|f| f.post_script_name == postscript_name)
            .next()
    }

    pub fn query(&self, query: &Query<'_>) -> Option<&FaceInfo> {
        self.db.query(query).and_then(|id| self.db.face(id))
    }

    pub fn all_fonts(&self) -> Vec<&FaceInfo> {
        self.db.faces().iter().collect::<Vec<&FaceInfo>>()
    }

    pub fn fonts_by_family(&self, family: &Family) -> Vec<&FaceInfo> {
        Some(self.db.family_name(family))
            .map(|name| {
                self.db
                    .faces()
                    .iter()
                    .filter(|face| {
                        face.families
                            .iter()
                            .find(|family| family.0 == name)
                            .is_some()
                    })
                    .collect::<Vec<&FaceInfo>>()
            })
            .unwrap_or_else(|| Vec::new())
    }

    pub fn family_name(family_name: &str) -> Family {
        match family_name.clone().to_lowercase().as_str() {
            "serif" => Family::Serif,
            "sans-serif" => Family::SansSerif,
            "sans serif" => Family::SansSerif,
            "monospace" => Family::Monospace,
            "cursive" => Family::Cursive,
            "fantasy" => Family::Fantasy,
            _ => Family::Name(family_name),
        }
    }

    pub fn font_from_desc(&self, desc: FontDescriptor) -> Option<&FaceInfo> {
        match desc {
            FontDescriptor::PostScript(ref name) => self.select_postscript(name),
            FontDescriptor::Properties {
                ref family,
                weight,
                slant,
                stretch,
            } => self.query(&Query {
                families: &[Self::family_name(family)],
                stretch,
                weight,
                style: slant,
            }),
        }
    }
}
