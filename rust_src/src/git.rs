use std::path::Path;

use git2::{Blame, BlameHunk, Commit, Oid, Repository};

use lisp_macros::lisp_fn;

use lisp::multibyte::LispStringRef;

use lisp::remacs_sys::{
    call0, call1, call2, call3, make_string, make_string_from_utf8, EmacsInt, Ffuncall,
};

use lisp::{
    lisp::{ExternalPtr, LispObject},
    symbol::intern,
};

use libc;
use std;

pub fn git_repo(path: LispStringRef) -> Repository {
    match Repository::init(Path::new(path.to_utf8().as_str())) {
        Ok(repo) => repo,
        Err(e) => {
            error!("Error initializing repository {:?}", e);
        }
    }
}

pub fn git_blame_file_rs<'a>(repo: &'a Repository, path: LispStringRef) -> Blame<'a> {
    match repo.blame_file(&Path::new(path.to_utf8().as_str()), None) {
        Err(e) => {
            println!("{:?}", e);
            error!("Error getting blame {:?}", e);
        }
        Ok(b) => b,
    }
}

pub fn git_commit<'a>(repo: &'a Repository, oid: Oid) -> Commit<'a> {
    match repo.find_commit(oid) {
        Ok(commit) => commit,
        Err(e) => {
            error!("Error finding commit {:?}", e);
        }
    }
}

pub fn git_summary(c: &Commit) -> LispObject {
    unsafe {
        match c.summary() {
            None => error!("Error getting author name {:?}", "summary"),

            Some(s) => make_string_from_utf8(
                s.as_ptr() as *const libc::c_char,
                s.chars().count() as isize,
            ),
        }
    }
}

pub fn git_author(c: &Commit) -> LispObject {
    let a = c.author();
    unsafe {
        match a.name() {
            None => error!("Error getting author name {:?}", "summary"),
            Some(s) => make_string_from_utf8(
                s.as_ptr() as *const libc::c_char,
                s.chars().count() as isize,
            ),
        }
    }
}

pub fn git_author_mail(c: &Commit) -> LispObject {
    let a = c.author();
    unsafe {
        match a.email() {
            None => error!("Error getting author name {:?}", "foo"),
            Some(s) => make_string_from_utf8(
                s.as_ptr() as *const libc::c_char,
                s.chars().count() as isize,
            ),
        }
    }
}

pub fn git_author_time(c: &Commit) -> LispObject {
    let a = c.author();
    unsafe {
        let fuu = a.when().seconds();
        let seconds = fuu.to_string();
        let seconds = seconds.as_str();

        make_string_from_utf8(
            seconds.as_ptr() as *const libc::c_char,
            seconds.chars().count() as isize,
        )
    }
}

pub fn git_author_tz(c: &Commit) -> LispObject {
    let a = c.author();
    unsafe {
        let fuu = a.when().offset_minutes();
        let seconds = fuu.to_string();
        let seconds = seconds.as_str();

        make_string_from_utf8(
            seconds.as_ptr() as *const libc::c_char,
            seconds.chars().count() as isize,
        )
    }
}

pub fn git_committer(c: &Commit) -> LispObject {
    let a = c.committer();
    unsafe {
        match a.name() {
            None => error!("Error getting author name {:?}", "foo"),
            Some(s) => make_string_from_utf8(
                s.as_ptr() as *const libc::c_char,
                s.chars().count() as isize,
            ),
        }
    }
}

pub fn git_committer_mail(c: &Commit) -> LispObject {
    let a = c.committer();
    unsafe {
        match a.email() {
            None => error!("Error getting author name {:?}", "foo"),
            Some(s) => make_string_from_utf8(
                s.as_ptr() as *const libc::c_char,
                s.chars().count() as isize,
            ),
        }
    }
}

pub fn git_committer_time(c: &Commit) -> LispObject {
    let a = c.committer();
    unsafe {
        let fuu = a.when().seconds();
        let seconds = fuu.to_string();
        let seconds = seconds.as_str();

        make_string_from_utf8(
            seconds.as_ptr() as *const libc::c_char,
            seconds.chars().count() as isize,
        )
    }
}

pub fn git_committer_tz(c: &Commit) -> LispObject {
    let a = c.committer();
    unsafe {
        let fuu = a.when().offset_minutes();
        let seconds = fuu.to_string();
        let seconds = seconds.as_str();

        make_string_from_utf8(
            seconds.as_ptr() as *const libc::c_char,
            seconds.chars().count() as isize,
        )
    }
}

include!(concat!(env!("OUT_DIR"), "/git_exports.rs"));
