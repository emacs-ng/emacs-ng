use core::ops::Deref;
#[cfg(free_unix)]
use font_loader::system_fonts;
use fontdb::{FaceInfo, Family, Query, Stretch, Style, Weight};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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

/// A font
pub struct Font<'a> {
    pub info: &'a fontdb::FaceInfo,
    pub data: &'a [u8],
    pub face: rustybuzz::Face<'a>,
    // pub face: ttf_parser::Face<'a>,
}

impl<'a> Font<'a> {
    pub fn new(info: &'a fontdb::FaceInfo) -> Option<Self> {
        let data = match &info.source {
            fontdb::Source::Binary(data) => data.deref().as_ref(),
            // #[cfg(feature = "std")]
            fontdb::Source::File(path) => {
                log::warn!("Unsupported fontdb Source::File('{}')", path.display());
                return None;
            }
            // #[cfg(feature = "std")]
            fontdb::Source::SharedFile(_path, data) => data.deref().as_ref(),
        };

        Some(Self {
            info,
            data,
            face: rustybuzz::Face::from_slice(data, info.index)?,
        })
    }
}

#[ouroboros::self_referencing]
pub struct FontDBInner {
    language: fontdb::Language,
    /// there should be a fontdb::Database::language
    db: fontdb::Database,
    #[borrows(db)]
    #[not_covariant]
    font_cache: Mutex<HashMap<fontdb::ID, Option<Arc<Font<'this>>>>>,
    font_matches_cache: Mutex<HashMap<FontDescriptor, Option<fontdb::ID>>>,
}

/// Access system fonts
pub struct FontDB(FontDBInner);

impl FontDB {
    pub fn new() -> FontDB {
        let mut db = fontdb::Database::new();
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::Instant::now();

        #[cfg(not(free_unix))]
        db.load_system_fonts();

        #[cfg(free_unix)]
        {
            let dirs = system_fonts::get_font_dirs();
            for dir in &dirs {
                log::trace!("Load fonts dir: {:?}", dir);
                db.load_fonts_dir(dir);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        log::info!(
            "Parsed {} font faces in {}ms.",
            db.len(),
            now.elapsed().as_millis()
        );

        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::Instant::now();

        //TODO only do this on demand!
        for i in 0..db.faces().len() {
            let id = db.faces()[i].id;
            unsafe {
                db.make_shared_face_data(id);
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        log::info!(
            "Mapped {} font faces in {}ms.",
            db.len(),
            now.elapsed().as_millis()
        );

        Self(
            FontDBInnerBuilder {
                language: fontdb::Language::English_UnitedStates,
                db,
                font_cache_builder: |_| Mutex::new(HashMap::new()),
                font_matches_cache: Mutex::new(HashMap::new()),
            }
            .build(),
        )
    }

    pub fn db(&self) -> &fontdb::Database {
        self.0.borrow_db()
    }

    pub fn select_postscript(&self, postscript_name: &str) -> Option<&FaceInfo> {
        self.db()
            .faces()
            .iter()
            .filter(|f| f.post_script_name == postscript_name)
            .next()
    }

    pub fn query(&self, query: &Query<'_>) -> Option<&FaceInfo> {
        self.db().query(query).and_then(|id| self.db().face(id))
    }

    pub fn all_fonts(&self) -> Vec<&FaceInfo> {
        self.db().faces().iter().collect::<Vec<&FaceInfo>>()
    }

    pub fn fonts_by_family(&self, family: &Family) -> Vec<&FaceInfo> {
        Some(self.db().family_name(family))
            .map(|name| {
                self.db()
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
            "default" => Family::Monospace, // emacs reports default
            "serif" => Family::Serif,
            "sans-serif" => Family::SansSerif,
            "sans serif" => Family::SansSerif,
            "monospace" => Family::Monospace,
            "cursive" => Family::Cursive,
            "fantasy" => Family::Fantasy,
            _ => Family::Name(family_name),
        }
    }

    pub fn face_info_from_desc(&self, desc: FontDescriptor) -> Option<&FaceInfo> {
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

    #[cfg(free_unix)]
    pub fn fc_family_name(name: &str) -> String {
        match name.clone().to_lowercase().as_str() {
            "default" => {
                let mut property = system_fonts::FontPropertyBuilder::new().monospace().build();
                let sysfonts = system_fonts::query_specific(&mut property);
                if let Some(family_name) = &sysfonts.get(0) {
                    log::trace!("Query: {} Name: {}", name, family_name,);
                    return family_name.to_string();
                }
                log::trace!("Query: {} monospace(default) not found", name);
                return name.to_string();
            }
            _ => {
                if let Some(family_name) = system_fonts::family_name(name) {
                    log::trace!("Query: {} Name: {}", name, family_name,);
                    return family_name;
                }
                log::trace!("Query: {} not found", name);
                return name.to_string();
            }
        }
    }

    // Clippy false positive
    #[allow(clippy::needless_lifetimes)]
    pub fn get_font<'a>(&'a self, id: fontdb::ID) -> Option<Arc<Font<'a>>> {
        self.0.with(|fields| get_font(&fields, id))
    }

    pub fn get_font_matches<'a>(&'a self, desc: FontDescriptor) -> Option<Arc<Font<'a>>> {
        let face_id = self.0.with(|fields| {
            let mut font_matches_cache = fields
                .font_matches_cache
                .try_lock()
                .expect("failed to lock font matches cache");

            font_matches_cache
                .entry(desc.clone())
                .or_insert_with(|| {
                    #[cfg(not(target_arch = "wasm32"))]
                    let now = std::time::Instant::now();

                    let mut face_id = None;
                    if let Some(face) = self.face_info_from_desc(desc.clone()) {
                        face_id = Some(face.id);
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let elapsed = now.elapsed();
                        log::debug!("font matches for {:?} in {:?}", desc, elapsed);
                    }
                    face_id
                })
                .clone()
        });

        if let Some(id) = face_id {
            return self.get_font(id);
        }

        None
    }
}

#[allow(dead_code)]
fn get_font<'b>(
    fields: &ouroboros_impl_font_db_inner::BorrowedFields<'_, 'b>,
    id: fontdb::ID,
) -> Option<Arc<Font<'b>>> {
    fields
        .font_cache
        .try_lock()
        .expect("failed to lock font cache")
        .entry(id)
        .or_insert_with(|| {
            let face = fields.db.face(id)?;
            match Font::new(face) {
                Some(font) => Some(Arc::new(font)),
                None => {
                    log::warn!("failed to load font '{}'", face.post_script_name);
                    None
                }
            }
        })
        .clone()
}
