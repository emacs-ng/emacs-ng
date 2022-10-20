use std::io::BufRead;

use std::io::{BufReader, Result};
use std::process::{Child, Command, Stdio};
use std::thread;

use regex::Regex;

use ng_async::ng_async::{to_owned_userdata, EmacsPipe, PipeDataOption, UserData};

use emacs::lisp::LispObject;
use emacs::list::{LispCons, LispConsCircularChecks, LispConsEndChecks};
use emacs::multibyte::LispStringRef;
use lisp_macros::lisp_fn;

use emacs::bindings::{find_newline_no_quit, Fintern};

use emacs::globals::{Qnil, Qt};

#[derive(Clone)]
pub struct GitBlameChunkInfo {
    pub orig_rev: String,
    pub orig_line: isize,
    pub final_line: isize,
    pub num_lines: isize,

    pub orig_file: String,

    pub orig_pos: isize,
    pub final_pos: isize,

    pub author: String,
    pub author_mail: String,

    pub author_time: String,
    pub author_tz: String,
    pub committer: String,
    pub committer_mail: String,
    pub committer_time: String,
    pub committer_tz: String,
    pub summary: String,
}

impl Default for GitBlameChunkInfo {
    fn default() -> Self {
        Self {
            orig_rev: String::from(""),
            orig_line: 0_isize,
            final_line: 0_isize,
            num_lines: 0_isize,

            orig_file: String::from(""),

            orig_pos: 0_isize,
            final_pos: 0_isize,

            author: String::from(""),
            author_mail: String::from(""),
            author_time: String::from(""),
            author_tz: String::from(""),
            committer: String::from(""),
            committer_mail: String::from(""),
            committer_time: String::from(""),
            committer_tz: String::from(""),
            summary: String::from(""),
        }
    }
}

pub fn git_async_create_process(
    program: String,
    args: Vec<String>,
    pipe: EmacsPipe,
    dir: String,
) -> Result<()> {
    let process: Child = Command::new(program)
        .current_dir(dir)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut out = process.stdout;
    let mut out_pipe = pipe;
    let sender = out_pipe.get_sender();

    thread::spawn(move || {
        let stdout_reader = BufReader::new(out.as_mut().unwrap());
        let mut chunk_vec: Vec<String> = Vec::with_capacity(13_usize);
        let re = Regex::new(r"^filename").unwrap();

        let mut previous_chunk = GitBlameChunkInfo::default();

        let parse_line = |s: &String| -> String {
            s.split(' ')
                .skip(1)
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        };

        for l in stdout_reader.lines() {
            let s = l.as_ref().unwrap();

            chunk_vec.push(s.to_string());

            if re.is_match(s) {
                let chunk: Vec<String> = chunk_vec[0].split(' ').map(|s| s.to_string()).collect();

                let mut current_chunk: GitBlameChunkInfo = if chunk_vec.len() > 3 {
                    GitBlameChunkInfo {
                        orig_rev: chunk[0].to_owned(),
                        orig_line: chunk[1].parse().unwrap(),
                        final_line: chunk[2].parse().unwrap(),
                        num_lines: chunk[3].parse().unwrap(),

                        author: parse_line(&chunk_vec[1]),
                        author_mail: parse_line(&chunk_vec[2]),
                        author_time: parse_line(&chunk_vec[3]),
                        author_tz: parse_line(&chunk_vec[4]),
                        committer: parse_line(&chunk_vec[5]),
                        committer_mail: parse_line(&chunk_vec[6]),
                        committer_time: parse_line(&chunk_vec[7]),
                        committer_tz: parse_line(&chunk_vec[8]),
                        summary: parse_line(&chunk_vec[9]),
                        ..previous_chunk
                    }
                } else {
                    GitBlameChunkInfo {
                        orig_rev: chunk[0].to_owned(),
                        orig_line: chunk[1].parse().unwrap(),
                        final_line: chunk[2].parse().unwrap(),
                        num_lines: chunk[3].parse().unwrap(),
                        ..previous_chunk
                    }
                };
                chunk_vec.clear();

                let bytepos: *mut libc::ptrdiff_t = std::ptr::null_mut::<libc::ptrdiff_t>();

                let orig_pos =
                    unsafe { find_newline_no_quit(1, 1, current_chunk.final_line - 1, bytepos) };

                let final_pos = unsafe {
                    find_newline_no_quit(
                        orig_pos,
                        *bytepos,
                        current_chunk.num_lines,
                        std::ptr::null_mut(),
                    )
                };

                current_chunk = GitBlameChunkInfo {
                    orig_pos,
                    final_pos,
                    ..current_chunk
                };

                previous_chunk = current_chunk.clone();

                if let Err(_) = out_pipe.message_lisp(&sender, UserData::new(current_chunk)) {
                    break;
                }
            }
        }

        if let Err(e) = out_pipe.message_lisp(&sender, UserData::new(GitBlameChunkInfo::default()))
        {
            error!("{:?}", e);
        }
    });

    Ok(())
}

#[lisp_fn]
pub fn git_blame_handler(proc: LispObject, data: LispObject) -> LispObject {
    unsafe {
        let user_data: UserData = to_owned_userdata(data);
        let msg: GitBlameChunkInfo = user_data.unpack();

        if msg.author.is_empty() {
            call!(
                Fintern(LispObject::from("ng-git-blame-sentinel"), Qnil),
                proc
            );
        } else {
            let chunk = call!(
                Fintern(LispObject::from("ng-git-blame-chunk"), Qnil),
                Fintern(LispObject::from(":orig-rev"), Qnil),
                LispObject::from(msg.orig_rev),
                Fintern(LispObject::from(":orig-line"), Qnil),
                LispObject::from(msg.orig_line),
                Fintern(LispObject::from(":final-line"), Qnil),
                LispObject::from(msg.final_line),
                Fintern(LispObject::from(":num-lines"), Qnil),
                LispObject::from(msg.num_lines),
                Fintern(LispObject::from(":orig-file"), Qnil),
                LispObject::from(msg.orig_file)
            );

            let revinfo = list!(
                (LispObject::from("author"), LispObject::from(msg.author)),
                (
                    LispObject::from("author-mail"),
                    LispObject::from(msg.author_mail)
                ),
                (
                    LispObject::from("author-time"),
                    LispObject::from(msg.author_time)
                ),
                (
                    LispObject::from("author-tz"),
                    LispObject::from(msg.author_tz)
                ),
                (
                    LispObject::from("committer"),
                    LispObject::from(msg.committer)
                ),
                (
                    LispObject::from("committer-mail"),
                    LispObject::from(msg.committer_mail)
                ),
                (
                    LispObject::from("committer-time"),
                    LispObject::from(msg.committer_time)
                ),
                (
                    LispObject::from("committer-tz"),
                    LispObject::from(msg.committer_tz)
                ),
                (LispObject::from("summary"), LispObject::from(msg.summary))
            );

            call!(
                Fintern(LispObject::from("ng-git-blame-make-overlays"), Qnil),
                proc,
                chunk,
                revinfo,
                LispObject::from(msg.orig_pos),
                LispObject::from(msg.final_pos)
            );
        }
    }
    LispObject::from(1)
}

#[lisp_fn]
pub fn git_make_process(
    command: LispObject,
    args: LispObject,
    handler: LispObject,
    current_dir: LispStringRef,
) -> LispObject {
    let command_ref: LispStringRef = command.into();
    let command_string = command_ref.to_utf8();
    let (emacs_pipe, proc) = EmacsPipe::with_handler(
        handler,
        PipeDataOption::USER_DATA,
        PipeDataOption::USER_DATA,
    );

    let mut args_vec: Vec<String> = vec![];
    if args.is_not_nil() {
        let list_args: LispCons = args.into();

        list_args
            .iter_cars(LispConsEndChecks::on, LispConsCircularChecks::on)
            .for_each(|x| {
                if let Some(string_ref) = x.as_string() {
                    args_vec.push(string_ref.to_utf8());
                } else {
                    error!("make-git-command takes a list of string arguments");
                }
            });
    }

    if let Err(e) =
        git_async_create_process(command_string, args_vec, emacs_pipe, current_dir.to_utf8())
    {
        error!("{:?}", e);
    }

    proc
}

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/out/blame_exports.rs"));
