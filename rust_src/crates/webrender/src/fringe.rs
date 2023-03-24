use crate::frame::LispFrameExt;
use bit_vec::BitVec;
use emacs::frame::LispFrameRef;
use image_::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use std::sync::Arc;

use emacs::bindings::draw_fringe_bitmap_params;
use webrender::api::{ImageData, ImageDescriptor, ImageDescriptorFlags, ImageFormat, ImageKey};

use crate::output::CanvasRef;

#[derive(Clone)]
pub struct FringeBitmap {
    pub image_key: ImageKey,

    pub width: u32,
    pub height: u32,
}

pub fn get_or_create_fringe_bitmap(
    frame: LispFrameRef,
    which: i32,
    p: *mut draw_fringe_bitmap_params,
) -> Option<FringeBitmap> {
    if which <= 0 {
        return None;
    }

    let mut display_info = frame.display_info().get_inner();

    if let Some(bitmap) = display_info.fringe_bitmap_caches.get(&which) {
        return Some(bitmap.clone());
    }

    let bitmap = create_fringe_bitmap(frame.canvas(), p);

    // add bitmap to cache
    display_info
        .fringe_bitmap_caches
        .insert(which, bitmap.clone());

    return Some(bitmap);
}

fn create_fringe_bitmap(mut canvas: CanvasRef, p: *mut draw_fringe_bitmap_params) -> FringeBitmap {
    let image_buffer = create_fringe_bitmap_image_buffer(p);

    let (width, height) = image_buffer.dimensions();
    let descriptor = ImageDescriptor::new(
        width as i32,
        height as i32,
        ImageFormat::RGBA8,
        ImageDescriptorFlags::empty(),
    );

    let data = ImageData::Raw(Arc::new(image_buffer.to_rgba8().to_vec()));

    let image_key = canvas.add_image(descriptor, data);

    FringeBitmap {
        image_key,
        width,
        height,
    }
}

fn create_fringe_bitmap_image_buffer(p: *mut draw_fringe_bitmap_params) -> DynamicImage {
    let height = unsafe { (*p).h };

    let bitmap_width = 8 as u32;
    let bitmap_height = (height + unsafe { (*p).dh }) as u32;

    let bits = unsafe { std::slice::from_raw_parts((*p).bits, (8 * bitmap_height) as usize) };

    // convert unsigned short array into u8 array
    let bits: Vec<u8> = bits.iter().map(|v| *v as u8).collect();

    let bits = BitVec::from_bytes(&bits);

    let white_pixel = Rgba([255, 255, 255, 255]);
    let transparent_pixel = Rgba([0, 0, 0, 0]);

    let image_buffer = RgbaImage::from_fn(bitmap_width, bitmap_height, |x, y| {
        let index = (y * bitmap_width + x) as usize;

        if bits
            .get(index)
            .expect("RgbaImage construction: out of index.")
            == true
        {
            white_pixel
        } else {
            transparent_pixel
        }
    });

    DynamicImage::ImageRgba8(image_buffer)
}
