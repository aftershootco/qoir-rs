#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/qoir_bindings.rs"));

impl qoir_pixel_configuration {
    pub fn zero() -> Self {
        Self {
            pixfmt: 0,
            width_in_pixels: 0,
            height_in_pixels: 0,
        }
    }
}

impl qoir_pixel_buffer {
    pub fn zero() -> Self {
        Self {
            pixcfg: qoir_pixel_configuration::zero(),
            data: std::ptr::null_mut(),
            stride_in_bytes: 0,
        }
    }
}

impl qoir_rectangle {
    pub fn zero() -> Self {
        Self {
            x0: 0,
            y0: 0,
            x1: 0,
            y1: 0,
        }
    }
}

impl Default for qoir_decode_options {
    fn default() -> Self {
        Self {
            contextual_free_func: None,
            contextual_malloc_func: None,
            memory_func_context: std::ptr::null_mut(),
            decbuf: std::ptr::null_mut(),
            pixbuf: qoir_pixel_buffer_struct::zero(),
            pixfmt: QOIR_PIXEL_FORMAT__RGBA_NONPREMUL,
            dst_clip_rectangle: qoir_rectangle::zero(),
            use_dst_clip_rectangle: false,
            src_clip_rectangle: qoir_rectangle::zero(),
            use_src_clip_rectangle: false,
            offset_x: 0,
            offset_y: 0,
        }
    }
}

impl Default for qoir_encode_options {
    fn default() -> Self {
        Self {
            contextual_free_func: None,
            contextual_malloc_func: None,
            memory_func_context: std::ptr::null_mut(),
            encbuf: std::ptr::null_mut(),
            metadata_cicp_len: 0,
            metadata_cicp_ptr: std::ptr::null_mut(),
            metadata_iccp_len: 0,
            metadata_iccp_ptr: std::ptr::null_mut(),
            metadata_exif_len: 0,
            metadata_exif_ptr: std::ptr::null_mut(),
            metadata_xmp_len: 0,
            metadata_xmp_ptr: std::ptr::null_mut(),
            lossiness: 0,
            dither: false,
        }
    }
}
