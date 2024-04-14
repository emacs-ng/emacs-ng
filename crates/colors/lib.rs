#![feature(lazy_cell)]
use std::collections::HashMap;
use std::sync::LazyLock;

include!(concat!(env!("OUT_DIR"), "/colors.rs"));

pub static COLOR_MAP: LazyLock<HashMap<&'static str, (u8, u8, u8)>> =
    LazyLock::new(|| init_color());
