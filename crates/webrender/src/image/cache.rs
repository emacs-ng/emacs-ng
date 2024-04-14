use super::ImageExt;
use super::ImageRef;
use super::WrPixmapRef;
use emacs_sys::bindings::Fequal;
use emacs_sys::bindings::EMACS_UINT;
use emacs_sys::color::color_to_rgba;
use emacs_sys::globals::Qnil;
use emacs_sys::globals::Qsvg;
use emacs_sys::lisp::LispObject;
use image::codecs::gif::GifDecoder;
use image::codecs::pnm::PnmDecoder;
use image::codecs::pnm::PnmSubtype;
use image::codecs::webp::WebPDecoder;
use image::error::ImageFormatHint;
use image::error::UnsupportedError;
use image::error::UnsupportedErrorKind;
use image::imageops::FilterType;
use image::io::Reader;
use image::AnimationDecoder;
use image::DynamicImage;
use image::ImageError;
use image::ImageResult;
use image::Rgba;
use parking_lot::Mutex;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::BufRead;
use std::io::Cursor;
use std::io::Seek;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;
use webrender::api::ImageData;
use webrender::api::ImageDescriptor;
use webrender::api::ImageDescriptorFlags;
use webrender::api::ImageFormat;
use webrender::FastHashMap;

pub type ImageId = isize;
pub type ImageHash = EMACS_UINT;
pub enum WrPixmapData {
    Animation(Vec<image::Frame>), // FIXME Per frame image are with different id, means they are different struct image. Use image.hash to create global hash. Maybe use thread/async decoding
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
        let val = unsafe { emacs_sys::bindings::sxhash(*lobj) };
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
    pub fn with_slice<P, T>(self, p: P) -> Option<T>
    where
        P: FnOnce(&[u8]) -> Option<T>,
    {
        match self {
            Self::File(file) => {
                let file: LispObject = unsafe { emacs_sys::bindings::image_find_image_file(file) };
                if !file.is_string() || file.is_nil() {
                    image_error!("Cannot find image file: {:?}", file);
                    return None;
                }
                let filename = String::from(file);
                match std::fs::read(&filename) {
                    Ok(result) => p(result.as_slice()),
                    Err(e) => {
                        image_error!("Error open image file: {:?} {e:?}", file);
                        return None;
                    }
                }
            }
            Self::Data(data) => {
                if !data.is_string() {
                    image_error!("Invalid image data: {:?}", data);
                    return None;
                }
                let data = data.as_string().unwrap();
                p(data.as_slice())
            }
        }
    }

    pub fn with_svg_slice<P, T>(self, ltype: LispObject, p: P) -> Option<T>
    where
        P: FnOnce(&[u8]) -> Option<T>,
    {
        if ltype != Qsvg {
            return self.with_slice(|data| p(data));
        }
        None
        // self.with_slice(|data| {
        //     let mut opt = usvg::Options::default();
        //     match self {
        //         Self::File(file) => {
        //             let filename = String::from(file);
        //             // Get file's absolute directory.
        //             opt.resources_dir = std::fs::canonicalize(&filename)
        //                 .ok()
        //                 .and_then(|p| p.parent().map(|p| p.to_path_buf()));
        //         }
        //         Self::Data(_) => {}
        //     };

        //     match Self::svg_to_png(data, &opt) {
        //         Some(bytes) => p(bytes.as_slice()),
        //         None => {
        //             let file = match self {
        //                 Self::File(file) => file,
        //                 Self::Data(data) => unsafe {
        //                     Fmake_temp_file_internal(
        //                         "invalid_resvg".into(),
        //                         Qnil,
        //                         ".svg".into(),
        //                         data,
        //                     )
        //                 },
        //             };
        //             image_error!("Error reading svg file: {:?}", file);
        //             return None;
        //         }
        //     }
        // })
    }
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
                        image_error!("Error decoding image: {:?} {e:?}", def);
                        return None;
                    }
                }
            };

        self.with_svg_slice(ltype, |data| {
            let reader = Reader::new(Cursor::new(data));
            let result = self.decode_from_reader(reader, foreground_color, background_color);
            handle_result(result, Qnil)
        })
    }

    fn decode_from_reader<R: BufRead + Seek>(
        self,
        reader: image::io::Reader<R>,
        foreground_color: Rgba<u8>,
        background_color: Rgba<u8>,
    ) -> ImageResult<WrPixmapData> {
        let reader = reader.with_guessed_format()?;
        if let Some(format) = reader.format() {
            match format {
                // load animationed images
                image::ImageFormat::Gif => {
                    let gif_decoder = GifDecoder::new(reader.into_inner())?;
                    let frames = gif_decoder.into_frames().collect_frames()?;
                    return Ok(WrPixmapData::Animation(frames));
                }
                image::ImageFormat::Pnm => {
                    return Self::decode_pnm_image_from_reader(
                        reader.into_inner(),
                        foreground_color,
                        background_color,
                    );
                }
                image::ImageFormat::WebP => {
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
pub struct ImageCache(FastHashMap<ImageHash, ImageCacheResult>);

static IMAGE_CACHE: LazyLock<Arc<Mutex<ImageCache>>> =
    LazyLock::new(|| Arc::new(Mutex::new(ImageCache(FastHashMap::default()))));
impl ImageCache {
    pub fn global() -> &'static Arc<Mutex<ImageCache>> {
        &IMAGE_CACHE
    }
    fn cache_mut<P, T>(p: P) -> Option<T>
    where
        P: FnOnce(&mut ImageCache) -> T,
    {
        match Self::global().try_lock() {
            Some(mut cache) => Some(p(&mut cache)),
            None => {
                image_error!("Image cache not available...");
                None
            }
        }
    }

    fn cache<P, T>(p: P) -> Option<T>
    where
        P: FnOnce(&ImageCache) -> T,
    {
        match Self::global().try_lock() {
            Some(cache) => Some(p(&cache)),
            None => {
                image_error!("Image cache not available...");
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
            image_error!("Invalid image source: {:?}", image.spec());
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
