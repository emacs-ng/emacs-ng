#![feature(lazy_cell)]

mod data;
mod error;
use data::package_targets;
pub use data::packages_source;
use data::target_source;
pub use data::with_enabled_crates;
pub use data::with_enabled_crates_all;
pub use data::with_root_crate;
use data::with_root_crate_checked;
pub use data::Package;
pub use error::BuildError;
use std::sync::LazyLock;

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::fs::create_dir_all;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;
use std::process;

use regex::Regex;

static C_NAME: &str = "c_name = \"";

/// Exit with error $code after printing the $fmtstr to stderr
macro_rules! fail_with_msg {
    ($code:expr, $modname:expr, $lineno:expr, $($arg:expr),*) => {{
        eprintln!("In {} on line {}", $modname, $lineno);
        eprintln!($($arg),*);
        process::exit($code);
    }};
}

#[derive(Debug)]
pub struct LintMsg {
    modname: String,
    lineno: u32,
    msg: String,
}

impl LintMsg {
    fn new(modname: &str, lineno: u32, msg: String) -> Self {
        Self {
            modname: modname.to_string(),
            lineno: lineno,
            msg: msg,
        }
    }

    pub fn fail(self, code: i32) -> ! {
        fail_with_msg!(code, self.modname, self.lineno, "{}", self.msg);
    }
}

#[derive(Clone, Debug)]
struct ModuleInfo {
    pub name: String,
    pub path: PathBuf,
}

impl ModuleInfo {
    pub fn from_path(mod_path: &PathBuf, base: &PathBuf) -> ModuleInfo {
        let root = base.canonicalize().unwrap().parent().unwrap().to_path_buf();
        let mut name = mod_path.strip_prefix(&root).unwrap().to_path_buf();
        name.set_extension("");

        return ModuleInfo {
            path: mod_path.clone(),
            name: name.to_string_lossy().to_string(),
        };
    }
}

struct ModuleData {
    pub info: ModuleInfo,
    pub c_exports: Vec<(Option<String>, String)>,
    pub lisp_fns: Vec<(Option<String>, String)>,
    pub protected_statics: Vec<String>,
}

impl ModuleData {
    pub fn new(info: ModuleInfo) -> Self {
        Self {
            info: info,
            c_exports: Vec::new(),
            lisp_fns: Vec::new(),
            protected_statics: Vec::new(),
        }
    }
}

struct ModuleParser<'a> {
    info: &'a ModuleInfo,
    lineno: u32,
}

impl<'a> ModuleParser<'a> {
    pub fn new(mod_info: &'a ModuleInfo) -> Self {
        ModuleParser {
            info: mod_info,
            lineno: 0,
        }
    }

    pub fn run(&mut self, in_file: impl BufRead) -> Result<ModuleData, BuildError> {
        let mut mod_data = ModuleData::new(self.info.clone());
        let mut reader = in_file.lines();
        let mut has_include = false;
        let mut preceding_cfg: Option<String> = None;

        while let Some(next) = reader.next() {
            let line = next?;
            self.lineno += 1;

            if line.starts_with(' ') {
                continue;
            }

            if line.starts_with("declare_GC_protected_static!") {
                let var = self.parse_gc_protected_static(&line)?;
                mod_data.protected_statics.push(var);
            } else if line.starts_with("#[no_mangle]") {
                if let Some(next) = reader.next() {
                    let line = next?;

                    if let Some(func) = self.parse_c_export(&line, None)? {
                        self.lint_nomangle(&line)?;
                        mod_data.c_exports.push((preceding_cfg, func));
                    }

                    preceding_cfg = None;
                } else {
                    self.fail(1, "unexpected end of file");
                }
            } else if line.starts_with("#[cfg") {
                preceding_cfg = Some(line);
            } else if line.starts_with("#[lisp_fn") {
                let line = if line.ends_with("]") {
                    line.clone()
                } else {
                    let mut line = line.clone();
                    loop {
                        if let Some(next) = reader.next() {
                            let l = next?;
                            if !l.ends_with(")]") {
                                line += &l;
                            } else {
                                line += &l;
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    line
                };

                let name = if let Some(begin) = line.find(C_NAME) {
                    let start = begin + C_NAME.len();
                    let end = line[start..].find('"').unwrap() + start;
                    let name = line[start..end].to_string();
                    if name.starts_with('$') {
                        // Ignore macros, nothing we can do with them
                        continue;
                    }

                    Some(name)
                } else {
                    None
                };

                if let Some(next) = reader.next() {
                    let line = next?;

                    if let Some(func) = self.parse_c_export(&line, name)? {
                        mod_data.lisp_fns.push((preceding_cfg, func));
                    }
                } else {
                    self.fail(1, "unexpected end of file");
                }

                preceding_cfg = None;
            } else if line.starts_with("#[async_stream") {
                if let Some(next) = reader.next() {
                    let line = next?;

                    if let Some(func) = self.parse_c_export(&line, None)? {
                        let mut prefix = String::from("call_");
                        prefix.push_str(&func);
                        mod_data.lisp_fns.push((preceding_cfg, prefix));
                    }
                } else {
                    self.fail(1, "Unexpected end of file");
                }

                preceding_cfg = None;
            } else if line.starts_with("include!(concat!(") {
                has_include = true;
            } else if line.starts_with("/*") && !line.ends_with("*/") {
                while let Some(next) = reader.next() {
                    let line = next?;
                    if line.ends_with("*/") {
                        break;
                    }
                }
            } else {
                preceding_cfg = None;
            }
        }

        if !has_include && !(mod_data.lisp_fns.is_empty() && mod_data.protected_statics.is_empty())
        {
            let msg = format!(
                "{} is missing the required include for protected statics or lisp_fn exports.",
                path_as_str(self.info.path.file_name()).to_string()
            );

            self.fail(2, &msg);
        }

        Ok(mod_data)
    }

    fn fail(&mut self, code: i32, msg: &str) -> ! {
        fail_with_msg!(code, &self.info.name, self.lineno, "{}", msg);
    }

    /// Handle both no_mangle and lisp_fn functions
    fn parse_c_export(
        &mut self,
        line: &str,
        name: Option<String>,
    ) -> Result<Option<String>, LintMsg> {
        let name = self.validate_exported_function(name, line, "function must be public.")?;
        if let Some(func) = name {
            Ok(Some(func))
        } else {
            Ok(None)
        }
    }

    fn parse_gc_protected_static(&mut self, line: &str) -> Result<String, LintMsg> {
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"GC_protected_static!\((.+), .+\);"#).unwrap());

        match RE.captures(line) {
            Some(caps) => {
                let name = caps[1].to_string();
                Ok(name)
            }
            None => Err(LintMsg::new(
                &self.info.name,
                self.lineno,
                "could not parse protected static".to_string(),
            )),
        }
    }

    // Determine if a function is exported correctly and return that function's name or None.
    fn validate_exported_function(
        &mut self,
        name: Option<String>,
        line: &str,
        msg: &str,
    ) -> Result<Option<String>, LintMsg> {
        match name.or_else(|| get_function_name(line)) {
            Some(name) => {
                if line.starts_with("pub ") {
                    Ok(Some(name))
                } else if line.starts_with("fn ") {
                    Err(LintMsg::new(
                        &self.info.name,
                        self.lineno,
                        format!("\n`{}` is not public.\n{}", name, msg),
                    ))
                } else {
                    eprintln!(
                        "Unhandled code in the {} module at line {}",
                        self.info.name, self.lineno
                    );
                    unreachable!();
                }
            }
            None => Ok(None),
        }
    }

    fn lint_nomangle(&mut self, line: &str) -> Result<(), LintMsg> {
        if !(line.starts_with("pub extern \"C\" ") || line.starts_with("pub unsafe extern \"C\" "))
        {
            Err(LintMsg::new(
                &self.info.name,
                self.lineno,
                "'no_mangle' functions exported for C need 'extern \"C\"' too.".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

// Parse the function name out of a line of source
fn get_function_name(line: &str) -> Option<String> {
    if let Some(pos) = line.find('(') {
        if let Some(fnpos) = line.find("fn ") {
            let name = line[(fnpos + 3)..pos].trim();
            return Some(name.to_string());
        }
    }

    None
}

fn handle_target(t: &cargo_metadata::Target) -> Result<Vec<ModuleData>, BuildError> {
    let mut mod_datas: Vec<ModuleData> = Vec::new();
    let files = target_source(t)?;

    for file in files {
        //ignore the root module
        if file == t.src_path {
            continue;
        }
        let mod_data = handle_file(&file, &t.src_path.clone().into_std_path_buf())?;
        mod_datas.push(mod_data);
    }

    Ok(mod_datas)
}

fn handle_file(mod_path: &PathBuf, src_path: &PathBuf) -> Result<ModuleData, BuildError> {
    let mod_info = ModuleInfo::from_path(mod_path, src_path);

    let fp = match File::open(mod_info.path.clone()) {
        Ok(f) => f,
        Err(e) => {
            return Err(io::Error::new(
                e.kind(),
                format!("Failed to open {}: {}", mod_info.path.to_string_lossy(), e),
            )
            .into());
        }
    };

    let mut parser = ModuleParser::new(&mod_info);
    let mod_data = parser.run(BufReader::new(fp))?;
    Ok(mod_data)
}

// Transmute &OsStr to &str
fn path_as_str(path: Option<&OsStr>) -> &str {
    path.and_then(|p| p.to_str())
        .unwrap_or_else(|| panic!("Cannot understand string: {:?}", path))
}

pub fn env_var(name: &str) -> String {
    env::var(name).unwrap_or_else(|e| panic!("Could not find {} in environment: {}", name, e))
}

/// Find modules in PATH which should contain the src directory of a crate
fn find_crate_modules() -> Result<Vec<ModuleData>, BuildError> {
    let mut modules: Vec<ModuleData> = Vec::new();
    with_root_crate_checked(|root| {
        for (_, t) in package_targets(root).iter().enumerate() {
            let mut files = handle_target(t)?;
            modules.append(&mut files);
        }
        Ok(())
    })?;

    Ok(modules)
}

/// Lookup public functions in a crate's modules and add the declarations
/// to the c_exports file that is determined by out_file.
fn generate_crate_c_export_file(
    mut out_file: &File,
    modules: &Vec<ModuleData>,
) -> Result<(), BuildError> {
    for mod_data in modules {
        for (cfg, func) in &mod_data.c_exports {
            if let Some(cfg) = cfg {
                write!(out_file, "{}\n", cfg)?;
            }
            write!(
                out_file,
                "pub use crate::{}::{};\n",
                mod_data.info.name.replace("/", "::"),
                func
            )?;
        }
        for (cfg, func) in &mod_data.lisp_fns {
            if let Some(cfg) = cfg {
                write!(out_file, "{}\n", cfg)?;
            }
            write!(
                out_file,
                "pub use crate::{}::F{};\n",
                mod_data.info.name.replace("/", "::"),
                func
            )?;
        }
    }
    write!(out_file, "\n")?;

    Ok(())
}

/// Create c_exports.rs that holds a crate's generated bindings.
/// We call generate_crate_c_export_file to add regular functions bindings
/// and write_lisp_fns to create the include file for each module which holds
/// the lisp_fns.
pub fn generate_crate_exports() -> Result<(), BuildError> {
    let modules = find_crate_modules()?;
    let out_path: PathBuf = [&env_var("OUT_DIR")].iter().collect();
    let mut out_file = File::create(out_path.join("c_exports.rs"))?;

    generate_crate_c_export_file(&out_file, &modules)?;

    let crate_name = get_crate_name()?;
    write!(
        out_file,
        "#[no_mangle]\npub extern \"C\" fn {}_init_syms() {{\n",
        crate_name
    )?;
    let _ = with_enabled_crates(|packages| {
        for package in packages {
            let crate_name = &package.name.replace('-', "_");
            // Call a crate's init_syms function in the main c_exports file
            let crate_init_syms = format!("    {}::{}_init_syms();\n", crate_name, crate_name);
            write!(out_file, "{}", crate_init_syms)?
        }
        Ok(())
    });
    write_lisp_fns(&out_path, &out_file, &modules)?;

    write!(out_file, "}}\n")?;

    Ok(())
}

fn get_crate_name() -> Result<String, BuildError> {
    let mut name = String::new();
    with_root_crate(|root, _| {
        name = root.name.clone().replace("-", "_");
        Ok(())
    })?;

    Ok(name)
}

/// TODO lisp export for nest modules, set the path same relative to out-dir
/// Export lisp functions defined in rust by using the macro `export_lisp_fns`
/// Add *_init_syms function of each module to the c_exports OUT_FILE
fn write_lisp_fns(
    crate_path: &PathBuf,
    mut out_file: &File,
    modules: &Vec<ModuleData>,
) -> Result<(), BuildError> {
    for mod_data in modules {
        let exports_path: PathBuf = crate_path.join([&mod_data.info.name, "_exports.rs"].concat());

        if let Some(dir) = exports_path.parent() {
            create_dir_all(dir)?;
        }

        // Start with a clean slate
        if exports_path.exists() {
            fs::remove_file(&exports_path)?;
        }

        // Add lisp_fns
        if !mod_data.lisp_fns.is_empty() {
            let mut file = File::create(&exports_path)?;
            write!(
                file,
                "export_lisp_fns! {{\n    {}\n}}\n",
                mod_data
                    .lisp_fns
                    .iter()
                    .map(|lisp_fn| match lisp_fn {
                        (Some(cfg), func) => format!("{} {}", cfg, func),
                        (_, func) => format!("{}", func),
                    })
                    .collect::<Vec<String>>()
                    .join(",\n    ")
            )?;

            write!(
                out_file,
                "    {}::rust_init_syms();\n",
                mod_data.info.name.replace("/", "::")
            )?;
        }

        // Add protected_statics
        if !mod_data.protected_statics.is_empty() {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(exports_path)?;
            write!(
                file,
                "protect_statics_from_GC! {{ {} }}\n",
                mod_data.protected_statics.join(", ")
            )?;

            write!(
                out_file,
                "    {}::rust_static_syms();\n",
                mod_data.info.name
            )?;
        }
    }

    Ok(())
}
