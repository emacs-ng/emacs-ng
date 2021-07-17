use font_kit::{
    family_name::FamilyName,
    source::{Source, SystemSource},
};
use fontdb::{FaceInfo, Family, Query};

pub struct FontDB {
    pub db: fontdb::Database,

    family_serif: Option<String>,
    family_sans_serif: Option<String>,
    family_cursive: Option<String>,
    family_fantasy: Option<String>,
    family_monospace: Option<String>,
}

impl FontDB {
    pub fn new() -> FontDB {
        let mut db = fontdb::Database::new();

        db.load_system_fonts();

        FontDB {
            db,

            family_serif: Self::default_font_family(FamilyName::Serif),
            family_sans_serif: Self::default_font_family(FamilyName::SansSerif),
            family_cursive: Self::default_font_family(FamilyName::Cursive),
            family_fantasy: Self::default_font_family(FamilyName::Fantasy),
            family_monospace: Self::default_font_family(FamilyName::Monospace),
        }
    }

    pub fn select_family(&self, family: &Family) -> Vec<&FaceInfo> {
        let family = self.family_name(family);

        if family.is_none() {
            return Vec::new();
        }

        let family = family.unwrap();

        self.db
            .faces()
            .iter()
            .filter(|f| f.family == family)
            .collect::<Vec<&FaceInfo>>()
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

    pub fn all_families(&self) -> Option<Vec<String>> {
        SystemSource::new().all_families().ok()
    }

    fn default_font_family(default_font_family: FamilyName) -> Option<String> {
        let family = SystemSource::new()
            .select_family_by_generic_name(&default_font_family)
            .ok()?;

        let fonts = family.fonts();

        if fonts.len() == 0 {
            return None;
        }

        let font = fonts[0].load().ok()?;

        Some(font.family_name())
    }

    fn family_name<'a>(&'a self, family: &'a Family) -> Option<&'a str> {
        use std::ops::Deref;
        match family {
            Family::Name(ref name) => Some(name),
            Family::Serif => self.family_serif.as_ref().map(|t| t.deref()),
            Family::SansSerif => self.family_sans_serif.as_ref().map(|t| t.deref()),
            Family::Cursive => self.family_cursive.as_ref().map(|t| t.deref()),
            Family::Fantasy => self.family_fantasy.as_ref().map(|t| t.deref()),
            Family::Monospace => self.family_monospace.as_ref().map(|t| t.deref()),
        }
    }
}
