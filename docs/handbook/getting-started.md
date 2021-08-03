# Contributor's guide

## Getting started

Huge code bases can be intimidating and it always takes some time to
get familiar with a project. Don't be afraid to ask questions and to
say your opinion. We want to encourage you to share your thoughts with
us, so if you have an idea for a new feature or how we can improve the
existing ones, feel free to open an issue.

You should start by reading our docs. This also allows you to start
contributing by improving our docs. Since documentation is very
important to get new contibutors on board, this helps us to grow the
community.

This document is only supposed as an introduction but we will add more
doc files that contain detailed information on important parts of the
project. Advanced features like webrender or javascript support are
documented separately.

## Overview

Most of the additional functionality of emacs-ng are in
`rust_src/`. This is the root of the main crate, however the actual
code is in `rust_src/crates/`, except the crates that are
only used for the build.

Some crates will only built if you activate the related features when
calling `./configure`. You can find the features that are used by
emacs-ng in `Cargo.toml.in`. Some of them on by default. Take a look
at `configure.ac` to see how they are activated during the build
process.

We only apply changes to the emacs code if it's necessary for features
that are defined in rust. This way there are less merge conflicts when
we perform upstream merges.

Bug fixes that also affect emacs can be tracked by an issue, but
should but fixed upstream.

## Build

We use bindgen to generate rust bindings for the functions defined in
C. Those bindings are in the crate emacs and are used by other crates
through importing them from the emacs crate.

The crate ng-bindgen holds the code that is responsible for generating
the bindings.

There's an ongoing effort to explain how emacs-ng is built on top of
emacs in handbook/build.md.

## CI

Since we use github actions, the ci related files are located at
`.github/workflow`. The main job(test.yml) is for building emacs-ng
on Linux and osx (including formatting with rustfmt). There's also
docs.yml that is responsible for generating and deploying our docs.
The remaining files are used to test if the nix build works and to
create the release build that runs once a week.

## Tests

Currently we only run some JavaScript related tests. Because there are several
failing native lisp tests, we haven't included them in our build.
However there's an open issue that tracks the status. At this point
there are no tests that are written in rust.

If you want to know how you can run tests, look at handbook/tests.md.
The file also contains some test examples.

## Lisp

The emacs crate also contains most of the code that allows us to make
use of elisp types. One of the most important types is `LispObject`
which is the equivalent to the C type
`Lisp_Object`(handbook/types.md).

In order to define lisp functions in rust you have to take a look at
the macro `lisp_fn`. Compared to the C version `DEFUN` it provides a
lot more flexibility(handbook/lisp-fn.md).

## Adding features

The rust ecosystem provides tons of possibilities to improve emacs.
We intend to use make use of
[libgit](https://github.com/libgit2/libgit2) through
[git2-rs](https://github.com/rust-lang/git2-rs) to improve magit. It
would also be cool if we would have a terminal emulator based on rust.

If you have any idea for a new feature you want to work on, just add a
configure option and create a new crate. Do not hesitate to ask for help.
