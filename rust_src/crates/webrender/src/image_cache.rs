use super::color::color_to_rgba;
use super::image::ImageExt;
use super::image::ImageRef;
use super::image::WrPixmapRef;
use crate::font_db::FontDB;
use emacs::bindings::Fequal;
use emacs::bindings::EMACS_UINT;
use emacs::globals::Qnil;
use emacs::globals::Qsvg;
use emacs::lisp::LispObject;
use image_::{
    codecs::{
        gif::GifDecoder,
        pnm::{PnmDecoder, PnmSubtype},
        webp::WebPDecoder,
    },
    error::{ImageFormatHint, UnsupportedError, UnsupportedErrorKind},
    imageops::FilterType,
    io::Reader,
    AnimationDecoder, DynamicImage, ImageError, ImageResult, Rgba,
};
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::{
    io::{BufRead, Cursor, Seek},
    time::Duration,
};
use webrender::api::{ImageData, ImageDescriptor, ImageDescriptorFlags, ImageFormat};

pub type ImageId = isize;
pub type ImageHash = EMACS_UINT;
pub enum WrPixmapData {
    Animation(Vec<image_::Frame>), // FIXME Per frame image are with different id, means they are different struct image. Use image.hash to create global hash. Maybe use thread/async decoding
    // servo/components/net/image_cache.rs
    // servo/components/net_traits/image_cache.rs
    Static(DynamicImage),
}

pub type WrPixmapDataDecoded = (
    Option<(ImageDescriptor, ImageData)>,
    Option<(usize, Duration)>,
);
impl WrPixmapData {
    /// TODO Remove resize/rotate code and transform arg when we add transform support using
    /// WebRender when drawing
    fn prepare(&self, frame_index: usize, transform: WrPixmapRef) -> WrPixmapDataDecoded {
        let (image, meta) = match self {
            Self::Animation(frames) => {
                let frame = frames[frame_index].clone();
                let delay = frame.delay();
                (
                    DynamicImage::ImageRgba8(frame.into_buffer()),
                    Some((frames.len(), delay.into())),
                )
            }
            Self::Static(image) => (image.clone(), None),
        };

        let image = if let Some(size) = transform.size {
            image
                .clone()
                .resize_exact(size.width as u32, size.height as u32, FilterType::Lanczos3)
        } else {
            image
        };

        let image = match transform.rotation as u32 {
            90 => image.clone().rotate90(),
            180 => image.clone().rotate180(),
            270 => image.clone().rotate270(),
            _ => image,
        };

        let descriptor = ImageDescriptor::new(
            image.width() as i32,
            image.height() as i32,
            ImageFormat::RGBA8,
            ImageDescriptorFlags::empty(),
        );
        let data = ImageData::new(image.to_rgba8().to_vec());

        (Some((descriptor, data)), meta)
    }
}

pub enum ImageCacheResult {
    Available(WrPixmapData),
    Error(ImageId),
    Pending(ImageId),
}

impl fmt::Debug for ImageCacheResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Available(_) => write!(f, "Available"),
            Self::Error(id) => write!(f, "Error {}", id),
            Self::Pending(id) => write!(f, "Pending {}", id),
        }
    }
}

#[derive(Copy, Clone)]
pub enum ImageSource {
    File(LispObject),
    Data(LispObject),
}

impl fmt::Debug for ImageSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(file) => write!(f, "File {:?}", file),
            Self::Data(data) => write!(f, "Data {:?}", data),
        }
    }
}

impl Hash for ImageSource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let lobj = match self {
            Self::File(file) => {
                state.write_u8(1);
                file
            }
            Self::Data(data) => {
                state.write_u8(2);
                data
            }
        };
        let val = unsafe { emacs::bindings::sxhash(*lobj) };
        val.hash(state);
    }
}

impl PartialEq for ImageSource {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::File(f1), Self::File(f2)) => unsafe { Fequal(*f1, *f2).is_t() },
            (Self::Data(d1), Self::Data(d2)) => unsafe { Fequal(*d1, *d2).is_t() },
            _ => false,
        }
    }
}

impl Eq for ImageSource {}

impl ImageSource {
    fn decode(
        self,
        foreground_color: Rgba<u8>,
        background_color: Rgba<u8>,
        ltype: LispObject,
        _max_width: u32,
        _max_height: u32,
    ) -> Option<WrPixmapData> {
        let handle_result =
            |result: ImageResult<WrPixmapData>, def: LispObject| -> Option<WrPixmapData> {
                match result {
                    Ok(data) => Some(data),
                    Err(e) => {
                        image_error!("Error decoding image {:?} {e:?}", def);
                        return None;
                    }
                }
            };

        let bytes = match self {
            Self::File(file) => {
                let filename = String::from(file);
                match std::fs::read(&filename) {
                    Ok(result) => result,
                    Err(e) => {
                        image_error!("Error open image file {:?} {e:?}", file);
                        return None;
                    }
                }
            }
            Self::Data(data) => {
                let data: String = data.into();
                data.as_bytes().to_vec()
            }
        };

        let bytes = if ltype == Qsvg {
            let mut opt = usvg::Options::default();
            match self {
                Self::File(file) => {
                    let filename = String::from(file);
                    // Get file's absolute directory.
                    opt.resources_dir = std::fs::canonicalize(&filename)
                        .ok()
                        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
                }
                Self::Data(_) => {}
            };

            match Self::svg_to_png(&bytes, &opt) {
                Some(bytes) => bytes,
                None => {
                    return None;
                }
            }
        } else {
            bytes
        };

        let reader = Reader::new(Cursor::new(bytes.as_slice()));
        let result = self.decode_from_reader(reader, foreground_color, background_color);
        handle_result(result, Qnil)
    }

    // directly draw svg using webrender
    fn svg_to_png(contents: &[u8], opt: &usvg::Options) -> Option<Vec<u8>> {
        use resvg::usvg_text_layout::TreeTextToPath;

        let font_db = FontDB::global();
        let fontdb = font_db.db();

        // let mut fontdb = fontdb::Database::new();
        // fontdb.load_system_fonts();
        let result = usvg::Tree::from_data(contents, opt);
        let mut tree = match result {
            Ok(result) => result,
            Err(e) => {
                image_error!("Failed to parse svg {e:?}");
                return None;
            }
        };

        tree.convert_text(fontdb);
        let pixmap_size = tree.size.to_screen_size();
        let result = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height());
        if result.is_none() {
            image_error!("Failed to create tiny_skia pixmap");
            return None;
        }
        let mut pixmap = result.unwrap();
        match resvg::render(
            &tree,
            usvg::FitTo::Original,
            tiny_skia::Transform::default(),
            pixmap.as_mut(),
        ) {
            None => {
                image_error!("Failed to render svg using resvg");
                return None;
            }
            _ => {}
        }

        match pixmap.encode_png() {
            Ok(bytes) => Some(bytes),
            Err(error) => {
                image_error!("Failed to encode svg: {error:?}");
                None
            }
        }
    }

    fn decode_from_reader<R: BufRead + Seek>(
        self,
        reader: image_::io::Reader<R>,
        foreground_color: Rgba<u8>,
        background_color: Rgba<u8>,
    ) -> ImageResult<WrPixmapData> {
        let reader = reader.with_guessed_format()?;
        if let Some(format) = reader.format() {
            match format {
                // load animationed images
                image_::ImageFormat::Gif => {
                    let gif_decoder = GifDecoder::new(reader.into_inner())?;
                    let frames = gif_decoder.into_frames().collect_frames()?;
                    return Ok(WrPixmapData::Animation(frames));
                }
                image_::ImageFormat::Pnm => {
                    return Self::decode_pnm_image_from_reader(
                        reader.into_inner(),
                        foreground_color,
                        background_color,
                    );
                }
                image_::ImageFormat::WebP => {
                    let decoder = WebPDecoder::new(reader.into_inner())?;
                    if decoder.has_animation() {
                        let frames = decoder.into_frames().collect_frames()?;
                        return Ok(WrPixmapData::Animation(frames));
                    } else {
                        let image = DynamicImage::from_decoder(decoder)?;
                        return Ok(WrPixmapData::Static(image));
                    };
                }
                _ => {
                    let image_result = reader.decode()?;
                    return Ok(WrPixmapData::Static(image_result));
                }
            }
        }
        return Err(ImageError::Unsupported(
            UnsupportedError::from_format_and_kind(
                ImageFormatHint::Unknown,
                UnsupportedErrorKind::GenericFeature(String::from("Unable to guess image format!")),
            ),
        ));
    }

    fn decode_pnm_image_from_reader<R: BufRead + Seek>(
        reader: R,
        foreground_color: Rgba<u8>,
        background_color: Rgba<u8>,
    ) -> ImageResult<WrPixmapData> {
        let pnm_decoder = PnmDecoder::new(reader)?;

        let pnm_type = pnm_decoder.subtype();

        let image = DynamicImage::from_decoder(pnm_decoder)?;

        let black_pixel = Rgba([0, 0, 0, 255]);
        let white_pixel = Rgba([255, 255, 255, 255]);

        let image = match pnm_type {
            PnmSubtype::Bitmap(_) => {
                // Apply foreground and background to mono PBM images.
                let mut rgba = image.into_rgba8();

                rgba.pixels_mut().for_each(|p| {
                    if *p == black_pixel {
                        *p = foreground_color;
                    } else if *p == white_pixel {
                        *p = background_color;
                    }
                });

                DynamicImage::ImageRgba8(rgba)
            }
            _ => image,
        };
        Ok(WrPixmapData::Static(image))
    }
}

/// We cache image by its source while image.c caches image by its spec
/// images with same source might have different spec
/// TODO We plan implement spec(rotate/scale) using WebRender transform
pub struct ImageCache(HashMap<ImageHash, ImageCacheResult>);

static mut IMAGE_CACHE: OnceLock<Arc<Mutex<ImageCache>>> = OnceLock::new();
impl ImageCache {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self(HashMap::new())))
    }
    pub fn global() -> &'static Arc<Mutex<ImageCache>> {
        unsafe {
            IMAGE_CACHE.get_or_init(|| {
                log::trace!("image cache is being created...");
                Self::new()
            })
        }
    }
    fn cache_mut<P, T>(p: P) -> Option<T>
    where
        P: FnOnce(&mut ImageCache) -> T,
    {
        match Self::global().try_lock() {
            Ok(mut cache) => Some(p(&mut cache)),
            Err(e) => {
                image_error!("Image cache not available... {e:?}");
                None
            }
        }
    }

    fn cache<P, T>(p: P) -> Option<T>
    where
        P: FnOnce(&ImageCache) -> T,
    {
        match Self::global().try_lock() {
            Ok(cache) => Some(p(&cache)),
            Err(e) => {
                image_error!("Image cache not available... {e:?}");
                None
            }
        }
    }

    pub fn with_image_data<P, T>(
        hash: &ImageHash,
        frame_index: usize,
        transform: WrPixmapRef,
        p: P,
    ) -> Option<T>
    where
        P: FnOnce(WrPixmapDataDecoded) -> T,
    {
        let data = ImageCache::cache(|cache| {
            let result = cache.0.get(hash);
            match result {
                Some(result) => match result {
                    ImageCacheResult::Available(data) => data.prepare(frame_index, transform),
                    _ => (None, None),
                },
                None => (None, None),
            }
        });
        Some(p(data.unwrap()))
    }

    pub fn contains(hash: &ImageHash) -> bool {
        ImageCache::cache(|cache| {
            let result = cache.0.get(hash);
            result.is_some()
        })
        .unwrap_or(false)
    }

    pub fn available_p(hash: &ImageHash) -> bool {
        ImageCache::cache(|cache| {
            let result = cache.0.get(&hash);
            match result {
                Some(result) => match result {
                    ImageCacheResult::Available(_) => true,
                    _ => false,
                },
                None => false,
            }
        })
        .unwrap_or(false)
    }

    pub fn insert(hash: ImageHash, result: ImageCacheResult) -> Option<ImageCacheResult> {
        match ImageCache::cache_mut(|cache| cache.0.insert(hash, result)) {
            Some(result) => result,
            None => None,
        }
    }

    pub fn load(image: ImageRef) {
        let id = image.id;
        let source = image.source();

        if source.is_none() {
            image_error!("Invalid image source {:?}", image.spec());
            return;
        }

        let source = source.unwrap();
        let hash = image.hash();
        if ImageCache::contains(&hash) {
            log::trace!("image {source:?} already in cache");
            return;
        }

        log::trace!("pending image {source:?} {id:?}");
        ImageCache::insert(hash, ImageCacheResult::Pending(id));

        let data = source.decode(
            color_to_rgba(image.foreground_color()),
            color_to_rgba(image.background_color()),
            image.ltype(),
            image.max_width(),
            image.max_height(),
        );
        let result = match data {
            Some(data) => {
                log::trace!("image {source:?} {id:?} loaded");
                ImageCacheResult::Available(data)
            }
            None => {
                log::trace!("image {source:?} {id:?} error");
                ImageCacheResult::Error(id)
            }
        };
        ImageCache::insert(hash, result);
    }
}
