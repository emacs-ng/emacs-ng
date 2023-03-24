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
}

impl<'a> Font<'a> {
    pub fn new(info: &'a fontdb::FaceInfo) -> Option<Self> {
        let data = match &info.source {
            fontdb::Source::Binary(data) => data.deref().as_ref(),
            fontdb::Source::File(path) => {
                log::warn!("Unsupported fontdb Source::File('{}')", path.display());
                return None;
            }
            fontdb::Source::SharedFile(_path, data) => data.deref().as_ref(),
        };

        Some(Self {
            info,
            data,
            face: rustybuzz::Face::from_slice(data, info.index)?,
        })
    }
}

#[allow(dead_code)]
pub struct FontDB<'a> {
    language: fontdb::Language,
    /// there should be a fontdb::Database::language
    db: fontdb::Database,
    font_cache: Mutex<HashMap<fontdb::ID, Option<Arc<Font<'a>>>>>,
    font_matches_cache: Mutex<HashMap<FontDescriptor, Option<fontdb::ID>>>,
}

impl FontDB<'static> {
    pub fn new() -> FontDB<'static> {
        let mut db = fontdb::Database::new();
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::Instant::now();

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

        Self {
            language: fontdb::Language::English_UnitedStates,
            db,
            font_cache: Mutex::new(HashMap::new()),
            font_matches_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn db(&self) -> &fontdb::Database {
        &self.db
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
    pub fn get_font(&'static mut self, id: fontdb::ID) -> Option<Arc<Font<'static>>> {
        let mut cache = self
            .font_cache
            .try_lock()
            .expect("failed to lock font cache");

        if cache.get(&id).is_none() {
            unsafe {
                self.db.make_shared_face_data(id);
            }
        }

        cache
            .entry(id)
            .or_insert_with(|| {
                let face = self.db.face(id)?;
                let font = Font::new(face);
                match font {
                    Some(font) => Some(Arc::new(font)),
                    None => {
                        log::warn!("failed to load font '{}'", face.post_script_name);
                        None
                    }
                }
            })
            .clone()
    }

    pub fn get_font_matches(&'static mut self, desc: FontDescriptor) -> Option<Arc<Font<'static>>> {
        let face_id = {
            let mut font_matches_cache = self
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
        };

        if let Some(id) = face_id {
            return self.get_font(id);
        }

        None
    }
}
