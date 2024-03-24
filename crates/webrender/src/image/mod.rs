pub mod cache;
use cache::ImageCache;
use cache::ImageHash;
use cache::ImageSource;
use emacs_sys::bindings::EMACS_UINT;
use std::time::Duration;
use webrender::api::ImageKey;

use emacs_sys::bindings::image;
use emacs_sys::bindings::make_float;
use emacs_sys::bindings::plist_get;
use emacs_sys::definitions::EmacsInt;
use emacs_sys::frame::FrameRef;
use emacs_sys::globals::*;
use emacs_sys::lisp::ExternalPtr;
use emacs_sys::lisp::LispObject;
use libc::c_void;
use webrender::api::units::DeviceIntSize;
use webrender::api::ColorF;
use webrender::api::ImageData;
use webrender::api::ImageDescriptor;

use crate::frame::FrameExtWrCommon;

use emacs_sys::color::lookup_color_by_name_or_hex;
use emacs_sys::color::pixel_to_color;

pub type ImageRef = ExternalPtr<image>;

pub trait ImageExt {
    fn spec(self) -> LispObject;
    fn ltype(self) -> LispObject;
    fn source(self) -> Option<ImageSource>;
    fn hash(self) -> ImageHash;
    fn frame_index(self) -> usize;
    fn max_width(self) -> u32;
    fn max_height(self) -> u32;
    fn foreground_color(self) -> ColorF;
    fn background_color(self) -> ColorF;
    fn specified_file(self) -> LispObject;
    fn specified_data(self) -> LispObject;
    fn cache_available_p(self) -> bool;
    fn load(self, frame: FrameRef) -> bool;
    fn transform(self, frame: FrameRef, width: i32, height: i32, rotation: f64);
    fn pixmap(self) -> WrPixmapRef;
    fn data(&self, wr_pixmap: WrPixmapRef) -> (ImageDescriptor, ImageData);
    fn meta(&self, frame: FrameRef) -> Option<(ImageKey, ImageDescriptor)>;
}

impl ImageExt for ImageRef {
    fn spec(self) -> LispObject {
        self.spec.as_cons().unwrap().cdr()
    }
    fn ltype(self) -> LispObject {
        unsafe { plist_get(self.spec(), QCtype) }
    }
    fn source(self) -> Option<ImageSource> {
        let specified_data = self.specified_data();
        let specified_file = self.specified_file();
        if specified_data.is_nil() {
            return Some(ImageSource::File(specified_file));
        } else {
            return Some(ImageSource::Data(specified_data));
        }
    }
    fn frame_index(self) -> usize {
        let lindex = unsafe { plist_get(self.spec(), QCindex) };
        lindex.as_fixnum().unwrap_or(0) as usize
    }
    fn max_width(self) -> u32 {
        let max_width = unsafe { plist_get(self.spec(), QCmax_width) };
        max_width.as_fixnum().unwrap_or(0) as u32
    }
    fn max_height(self) -> u32 {
        let max_height = unsafe { plist_get(self.spec(), QCmax_height) };
        max_height.as_fixnum().unwrap_or(0) as u32
    }
    fn foreground_color(self) -> ColorF {
        let foreground_color = unsafe { plist_get(self.spec(), QCforeground) };
        foreground_color
            .as_string()
            .and_then(|s| {
                let s = s.to_string();
                lookup_color_by_name_or_hex(&s)
            })
            .unwrap_or_else(|| pixel_to_color(self.face_foreground))
    }
    fn background_color(self) -> ColorF {
        let background_color = unsafe { plist_get(self.spec(), QCbackground) };
        background_color
            .as_string()
            .and_then(|s| {
                let s = s.to_string();
                lookup_color_by_name_or_hex(&s)
            })
            .unwrap_or_else(|| pixel_to_color(self.face_background))
    }
    fn specified_file(self) -> LispObject {
        unsafe { plist_get(self.spec(), QCfile) }
    }
    fn specified_data(self) -> LispObject {
        unsafe { plist_get(self.spec(), QCdata) }
    }

    fn cache_available_p(self) -> bool {
        let hash = self.hash();
        ImageCache::available_p(&hash)
    }

    // Source hash
    fn hash(self) -> ImageHash {
        self.pixmap().source_hash
    }

    fn load(mut self, frame: FrameRef) -> bool {
        let source = self.source();
        let frame_index = self.frame_index();

        if !self.cache_available_p() || source.is_none() {
            ImageCache::load(self);
            return false;
        } else {
            let hash = self.hash();
            let transform = self.pixmap();
            ImageCache::with_image_data(&hash, frame_index, transform, |(data, meta)| {
                if data.is_none() {
                    return false;
                }
                let (descriptor, data) = data.unwrap();
                let size = descriptor.size;
                self.width = size.width;
                self.height = size.height;
                frame
                    .gl_renderer()
                    .add_or_update_image(&hash, descriptor, data);
                let lisp_data = animation_frame_meta_to_lisp_data(meta);
                self.lisp_data = lisp_data;
                return true;
            })
            .unwrap()
        }
    }
    // TODO: transform using WebRender LayoutTranform?
    // image.c using transform2D transform3D too
    fn transform(mut self, frame: FrameRef, width: i32, height: i32, rotation: f64) {
        let size = DeviceIntSize::new(width, height);
        self.pixmap().size = Some(size);
        self.pixmap().rotation = rotation;
        log::trace!("image transform width {width:?}, height {height:?}, rotation: {rotation:?}");
        let (descriptor, data) = self.data(self.pixmap());
        // update WebRender resource
        let hash = self.hash();
        frame
            .gl_renderer()
            .add_or_update_image(&hash, descriptor, data);
        // store transformed props
        let size = descriptor.size;
        self.width = size.width;
        self.height = size.height;
        // lisp_data are currently not used
        let lisp_data = self.lisp_data;
        self.lisp_data = lisp_data;
    }
    fn pixmap(mut self) -> WrPixmapRef {
        if self.pixmap.is_null() {
            let wr_pixmap = Box::new(WrPixmap::new(self.source().expect("No source for image")));
            let pixmap_ptr = Box::into_raw(wr_pixmap);
            self.pixmap = pixmap_ptr as *mut c_void;
        }
        (self.pixmap as *mut WrPixmap).into()
    }
    fn data(&self, wr_pixmap: WrPixmapRef) -> (ImageDescriptor, ImageData) {
        let hash = self.hash();
        let frame_index = self.frame_index();
        ImageCache::with_image_data(&hash, frame_index, wr_pixmap, |(data, _)| data.unwrap())
            .unwrap()
    }
    fn meta(&self, frame: FrameRef) -> Option<(ImageKey, ImageDescriptor)> {
        let hash = self.hash();
        frame.gl_renderer().image_key(&hash)
    }
}

pub struct WrPixmap {
    pub size: Option<DeviceIntSize>,
    pub rotation: f64,
    // Source hash
    pub source_hash: EMACS_UINT,
}

impl WrPixmap {
    fn new(source: ImageSource) -> Self {
        let lobj = match source {
            ImageSource::File(file) => file,
            ImageSource::Data(data) => data,
        };
        let source_hash = unsafe { emacs_sys::bindings::sxhash(lobj) };

        WrPixmap {
            size: None,
            rotation: 0.0,
            source_hash,
        }
    }
}

pub type WrPixmapRef = ExternalPtr<WrPixmap>;

pub fn can_use_native_image_api(image_type: LispObject) -> bool {
    match image_type {
        // TODO Qxbm (supported by GNU Emacs but not image-rs )
        // TODO Qavif (Non-default, even in `avif`. Requires stable Rust and native dependency libdav1d.)
        Qnative_image | Qpng | Qjpeg | Qgif | Qtiff | Qpbm | Qsvg | Qwebp | Qpnm | Qtga | Qdds
        | Qbmp | Qico | Qhdr | Qopen_exr | Qfarbfeld => true,
        _ => false,
    }
}

fn animation_frame_meta_to_lisp_data(animation_meta: Option<(usize, Duration)>) -> LispObject {
    match animation_meta {
        Some((frame_count, delay)) => {
            let mut lisp_data = Qnil;

            if frame_count > 0 {
                lisp_data = LispObject::cons(
                    Qcount,
                    LispObject::cons(LispObject::from_fixnum(frame_count as EmacsInt), lisp_data),
                );
            }

            let delay = delay.as_secs_f64();

            if delay > 0.0 {
                lisp_data = LispObject::cons(
                    Qdelay,
                    LispObject::cons(unsafe { make_float(delay) }, lisp_data),
                );
            }

            lisp_data
        }
        None => Qnil,
    }
}
