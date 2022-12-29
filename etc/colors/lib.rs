use lazy_static::lazy_static;
use std::collections::HashMap;

include!(concat!(env!("OUT_DIR"), "/colors.rs"));

lazy_static! {
    pub static ref COLOR_MAP: HashMap<&'static str, (u8, u8, u8)> = init_color();
}
