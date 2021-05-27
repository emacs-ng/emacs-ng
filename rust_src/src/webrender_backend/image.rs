use std::{
    ffi::CString,
    io::{BufRead, Cursor, Seek},
    ptr,
    sync::Arc,
    time::Duration,
};

use emacs::{
    bindings::{add_to_log, image as Emacs_Image, make_float, Fplist_get},
    definitions::EmacsInt,
    frame::LispFrameRef,
    globals::{QCindex, Qcount, Qdelay, Qgif, Qjpeg, Qnative_image, Qnil, Qpbm, Qpng, Qtiff},
    lisp::LispObject,
};
use image::{
    codecs::gif::GifDecoder, imageops::FilterType, io::Reader, AnimationDecoder, DynamicImage,
    GenericImageView, ImageFormat, ImageResult,
};
use libc::c_void;
use webrender::api::ImageKey;

use super::output::OutputRef;

pub struct WrPixmap {
    pub image_key: ImageKey,
    pub image_buffer: DynamicImage,
}

pub fn can_use_native_image_api(image_type: LispObject) -> bool {
    match image_type {
        Qnative_image | Qpng | Qjpeg | Qgif | Qtiff | Qpbm => true,
        _ => false,
    }
}

fn open_image(
    spec_file: LispObject,
    spec_data: LispObject,
    frame_index: usize,
) -> Option<(DynamicImage, Option<(usize, Duration)>)> {
    if spec_file.is_string() {
        let filename = spec_file.as_string().unwrap().to_string();

        let loaded_image = Reader::open(filename)
            .ok()
            .and_then(|r| decode_image_from_reader(r, frame_index).ok());

        return loaded_image;
    }

    if spec_data.is_string() {
        let data = spec_data.as_string().unwrap();

        let reader = Reader::new(Cursor::new(data.as_slice()));
        let loaded_image = decode_image_from_reader(reader, frame_index).ok();
        return loaded_image;
    }

    return None;
}

fn decode_gif_image_from_reader<R: BufRead + Seek>(
    reader: R,
    frame_index: usize,
) -> ImageResult<(DynamicImage, (usize, Duration))> {
    let gif_decoder = GifDecoder::new(reader)?;
    let frames = gif_decoder.into_frames().collect_frames()?;

    let frame = frames[frame_index].clone();

    let frame_count = frames.len();
    let delay = frame.delay();

    Ok((
        DynamicImage::ImageRgba8(frame.into_buffer()),
        (frame_count, delay.into()),
    ))
}

fn decode_image_from_reader<R: BufRead + Seek>(
    reader: image::io::Reader<R>,
    frame_index: usize,
) -> ImageResult<(DynamicImage, Option<(usize, Duration)>)> {
    let reader = reader.with_guessed_format()?;

    // load animationed images
    if reader.format() == Some(ImageFormat::Gif) {
        return decode_gif_image_from_reader(reader.into_inner(), frame_index)
            .map(|(image, meta)| (image, Some(meta)));
    }

    let image_result = reader.decode()?;

    Ok((image_result, None))
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

fn define_image(frame: LispFrameRef, img: *mut Emacs_Image, image_buffer: DynamicImage) {
    let width = image_buffer.width() as i32;
    let height = image_buffer.height() as i32;

    let output: OutputRef = unsafe { frame.output_data.wr.into() };

    let old_image_key = if unsafe { (*img).pixmap } != ptr::null_mut() {
        let pixmap = unsafe { (*img).pixmap as *mut WrPixmap };

        Some(unsafe { (*pixmap).image_key })
    } else {
        None
    };

    let pixmap = if let Some(image_key) = old_image_key {
        output.update_image(
            image_key,
            width,
            height,
            Arc::new(image_buffer.to_rgba8().into_raw()),
        );

        WrPixmap {
            image_key,
            image_buffer,
        }
    } else {
        let image_key =
            output.add_image(width, height, Arc::new(image_buffer.to_rgba8().into_raw()));

        WrPixmap {
            image_key,
            image_buffer,
        }
    };

    // take back old pixmap, let gc destroy its resource
    unsafe { Box::from_raw((*img).pixmap) };

    let pixmap = Box::new(pixmap);
    let pixmap_ptr = Box::into_raw(pixmap);

    unsafe {
        (*img).width = width;
        (*img).height = height;

        (*img).pixmap = pixmap_ptr as *mut c_void;
    };
}

pub fn load_image(
    frame: LispFrameRef,
    img: *mut Emacs_Image,
    spec_file: LispObject,
    spec_data: LispObject,
) -> bool {
    let spec = unsafe { (*img).spec }.as_cons().unwrap().cdr();
    let lisp_index = unsafe { Fplist_get(spec, QCindex) };
    let frame_index = lisp_index.as_fixnum().unwrap_or(0) as usize;

    let loaded_image = open_image(spec_file, spec_data, frame_index);

    if loaded_image == None {
        let format_str = CString::new("Unable to load image %s").unwrap();
        unsafe { add_to_log(format_str.as_ptr(), (*img).spec) };

        return false;
    }

    let (loaded_image, meta) = loaded_image.unwrap();

    define_image(frame, img, loaded_image);

    let lisp_data = animation_frame_meta_to_lisp_data(meta);
    unsafe { (*img).lisp_data = lisp_data };

    return true;
}

pub fn transform_image(
    frame: LispFrameRef,
    img: *mut Emacs_Image,
    width: i32,
    height: i32,
    rotation: f64,
) {
    let pixmap = unsafe { (*img).pixmap as *mut WrPixmap };

    let image_buffer = unsafe { (*pixmap).image_buffer.clone() };

    let image_buffer = image_buffer.resize_exact(width as u32, height as u32, FilterType::Lanczos3);

    let rotation = rotation as u32;
    let image_buffer = match rotation {
        90 => image_buffer.rotate90(),
        180 => image_buffer.rotate180(),
        270 => image_buffer.rotate270(),
        _ => image_buffer,
    };

    let lisp_data = unsafe { (*img).lisp_data };

    define_image(frame, img, image_buffer);

    unsafe { (*img).lisp_data = lisp_data };
}
