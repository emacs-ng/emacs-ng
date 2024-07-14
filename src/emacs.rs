// #[cfg(debug_assertions)]
#![feature(lazy_cell)]

use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use emacs_sys::bindings::main1;
use emacs_sys::bindings::terminate_due_to_signal;
use emacs_sys::bindings::will_dump_p;
use std::sync::LazyLock;

struct StandardArg {
    name: String,
    longname: Option<String>,
    priority: i32,
    nargs: i32,
}

static STANDARDARGS: LazyLock<Vec<StandardArg>> = LazyLock::new(|| {
    let mut args = Vec::new();
    let mut insert = |name: &str, longname: Option<&str>, priority: i32, nargs: i32| {
        let name = name.to_string();
        let longname = longname.map(|n| n.to_string());
        args.push(StandardArg {
            name,
            longname,
            priority,
            nargs,
        });
    };
    insert("-version", Some("--version"), 150, 0);
    #[cfg(have_pdumper)]
    insert("-fingerprint", Some("--fingerprint"), 140, 0);
    insert("-chdir", Some("--chdir"), 130, 1);
    insert("-t", Some("--terminal"), 120, 1);
    insert("-nw", Some("--no-window-system"), 110, 0);
    insert("-nw", Some("--no-windows"), 110, 0);
    insert("-batch", Some("--batch"), 100, 0);
    insert("-script", Some("--script"), 100, 1);
    insert("-daemon", Some("--daemon"), 99, 0);
    insert("-bg-daemon", Some("--bg-daemon"), 99, 0);
    insert("-fg-daemon", Some("--fg-daemon"), 99, 0);
    insert("-help", Some("--help"), 90, 0);
    insert("-nl", Some("--no-loadup"), 70, 0);
    insert("-nsl", Some("--no-site-lisp"), 65, 0);
    insert("-no-build-details", Some("--no-build-details"), 63, 0);
    #[cfg(have_modules)]
    insert("-module-assertions", Some("--module-assertions"), 62, 0);
    /* -d must come last before the options handled in startup.el.  */
    insert("-d", Some("--display"), 60, 1);
    insert("-display", None, 60, 1);
    /* Now for the options handled in `command-line' (startup.el).  */
    /* (Note that to imply -nsl, -Q is partially handled here.)  */
    insert("-Q", Some("--quick"), 55, 0);
    insert("-quick", None, 55, 0);
    insert("-x", None, 55, 0);
    insert("-q", Some("--no-init-file"), 50, 0);
    insert("-no-init-file", None, 50, 0);
    insert("-init-directory", Some("--init-directory"), 30, 1);
    insert("-no-x-resources", Some("--no-x-resources"), 40, 0);
    insert("-no-site-file", Some("--no-site-file"), 40, 0);
    insert("-no-comp-spawn", Some("--no-comp-spawn"), 60, 0);
    insert("-u", Some("--user"), 30, 1);
    insert("-user", None, 30, 1);
    insert("-debug-init", Some("--debug-init"), 20, 0);
    insert("-iconic", Some("--iconic"), 15, 0);
    insert("-D", Some("--basic-display"), 12, 0);
    insert("-basic-display", None, 12, 0);
    insert("-nbc", Some("--no-blinking-cursor"), 12, 0);
    /* Now for the options handled in `command-line-1' (startup.el).  */
    insert("-nbi", Some("--no-bitmap-icon"), 10, 0);
    insert("-bg", Some("--background-color"), 10, 1);
    insert("-background", None, 10, 1);
    insert("-fg", Some("--foreground-color"), 10, 1);
    insert("-foreground", None, 10, 1);
    insert("-bd", Some("--border-color"), 10, 1);
    insert("-bw", Some("--border-width"), 10, 1);
    insert("-ib", Some("--internal-border"), 10, 1);
    insert("-ms", Some("--mouse-color"), 10, 1);
    insert("-cr", Some("--cursor-color"), 10, 1);
    insert("-fn", Some("--font"), 10, 1);
    insert("-font", None, 10, 1);
    insert("-fs", Some("--fullscreen"), 10, 0);
    insert("-fw", Some("--fullwidth"), 10, 0);
    insert("-fh", Some("--fullheight"), 10, 0);
    insert("-mm", Some("--maximized"), 10, 0);
    insert("-g", Some("--geometry"), 10, 1);
    insert("-geometry", None, 10, 1);
    insert("-T", Some("--title"), 10, 1);
    insert("-title", None, 10, 1);
    insert("-name", Some("--name"), 10, 1);
    insert("-xrm", Some("--xrm"), 10, 1);
    insert("-parent-id", Some("--parent-id"), 10, 1);
    insert("-r", Some("--reverse-video"), 5, 0);
    insert("-rv", None, 5, 0);
    insert("-reverse", None, 5, 0);
    insert("-hb", Some("--horizontal-scroll-bars"), 5, 0);
    insert("-vb", Some("--vertical-scroll-bars"), 5, 0);
    insert("-color", Some("--color"), 5, 0);
    insert("-no-splash", Some("--no-splash"), 3, 0);
    insert("-no-desktop", Some("--no-desktop"), 3, 0);
    /* The following three must be just above the file-name args, to get
    them out of our way, but without mixing them with file names.  */
    insert("-temacs", Some("--temacs"), 1, 1);
    #[cfg(have_pdumper)]
    insert("-dump-file", Some("--dump-file"), 1, 1);
    #[cfg(seccomp_usable)]
    insert("-seccomp", Some("--seccomp"), 1, 1);
    #[cfg(have_ns)]
    {
        insert("-NSAutoLaunch", None, 5, 1);
        insert("-NXAutoLaunch", None, 5, 1);
        insert("-_NSMachLaunch", None, 85, 1);
        insert("-MachLaunch", None, 85, 1);
        insert("-macosx", None, 85, 0);
        insert("-NSHost", None, 85, 1);
    }
    /* These have the same priority as ordinary file name args,
    so they are not reordered with respect to those.  */
    insert("-L", Some("--directory"), 0, 1);
    insert("-directory", None, 0, 1);
    insert("-l", Some("--load"), 0, 1);
    insert("-load", None, 0, 1);
    /* This has no longname, because using --scriptload confuses sort_args,
    because then the --script long option seems to match twice; ie
    you can't have a long option which is a prefix of another long
    option.  In any case, this is entirely an internal option.  */
    insert("-scriptload", None, 0, 1);
    insert("-f", Some("--funcall"), 0, 1);
    insert("-funcall", None, 0, 1);
    insert("-eval", Some("--eval"), 0, 1);
    insert("-execute", Some("--execute"), 0, 1);
    insert("-find-file", Some("--find-file"), 0, 1);
    insert("-visit", Some("--visit"), 0, 1);
    insert("-file", Some("--file"), 0, 1);
    insert("-insert", Some("--insert"), 0, 1);
    #[cfg(have_ns)]
    {
        insert("-NXOpen", None, 0, 1);
        insert("-NXOpenTemp", None, 0, 1);
        insert("-NSOpen", None, 0, 1);
        insert("-NSOpenTemp", None, 0, 1);
        insert("-GSFilePath", None, 0, 1);
    }
    /* This should be processed after ordinary file name args and the like.  */
    insert("-kill", Some("--kill"), -10, 0);
    args
});

// Include the main c_exports file that holds the main rust_init_syms.
// This function calls the other crates init_syms functions which contain
// the generated bindings.
#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/c_exports.rs"));

#[allow(unused_doc_comments)]
#[no_mangle]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use libc::c_char;
    use libc::c_int;
    use std::ffi::CString;

    // https://stackoverflow.com/questions/34379641/how-do-i-convert-rust-args-into-the-argc-and-argv-c-equivalents/34379937#34379937
    // create a vector of zero terminated strings
    let args = std::env::args()
        .map(|arg| CString::new(arg).unwrap())
        .collect::<Vec<CString>>();
    // convert the strings to raw pointers
    let mut c_args = args
        .iter()
        .map(|arg| arg.clone().into_raw())
        .collect::<Vec<*mut c_char>>();
    // pass the pointer of the vector's internal buffer to a C function
    let argc = c_args.len() as c_int;
    let argv = c_args.as_mut_ptr();

    fn handle_exit_code(_code: c_int) -> Result<(), Box<dyn std::error::Error>> {
        // TODO
        Ok(())
    }

    unsafe {
        if will_dump_p() {
            let exit_code = main1(argc, argv);
            return handle_exit_code(exit_code);
        }
    }

    // install global collector configured based on EMACSNG_LOG env var.
    // #[cfg(debug_assertions)]
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_env("EMACSNG_LOG"))
        .init();

    log::trace!("Emacs NG");

    let exit_code = unsafe { main1(argc, argv) };

    // emacs abort
    unsafe { terminate_due_to_signal(libc::SIGABRT, 40) };
    return handle_exit_code(exit_code);
}
