use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

const RGB_TXT_PATH: &str = "../rgb.txt";

fn main() {
    let file = BufReader::new(File::open(RGB_TXT_PATH).unwrap());
    let color = file
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| !line.trim().is_empty())
        .filter(|line| !line.starts_with('#'))
        .map(|line| {
            let result = line
                .trim()
                .split("\t\t")
                .map(|str| str.to_owned())
                .collect::<Vec<String>>();

            let color = result[0]
                .split_whitespace()
                .map(|str| str.to_owned())
                .collect::<Vec<String>>();

            let name = result[1].trim().to_lowercase();

            let red = color[0].clone();
            let green = color[1].clone();
            let blue = color[2].clone();

            (name, (red, green, blue))
        });

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("colors.rs");

    let color_function_body = format!(
        "let mut color_map: HashMap<&'static str, (u8, u8, u8)> = HashMap::new(); {} color_map",
        color
            .map(|(name, (red, green, blue))| format!(
                "color_map.insert(\"{}\", ({}, {}, {}));\n",
                name, red, green, blue
            ))
            .collect::<Vec<String>>()
            .concat()
    );

    let color_fun_source = format!(
        "fn init_color() -> HashMap<&'static str, (u8, u8, u8)> {{ {} }}",
        color_function_body
    );

    let mut file = File::create(out_path).unwrap();
    file.write_all(color_fun_source.as_bytes()).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", RGB_TXT_PATH);
}
