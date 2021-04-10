<img src="images/logo.png" width="120" align="right">

[![](https://badges.gitter.im/emacs-ng/emacs-ng.svg)](https://gitter.im/emacsng)
[![](https://github.com/emacs-ng/emacs-ng/workflows/CI/badge.svg)](https://github.com/emacs-ng/emacs-ng/actions?query=workflow%3ACI)
[![](https://img.shields.io/reddit/subreddit-subscribers/emacsng?label=Join%20r%2Femacsng&style=social)](https://www.reddit.com/r/emacsng/)

# emacs-ng

A new approach to Emacs - Including TypeScript, Threading, Async I/O, and WebRender.

<hr>
<p align="center">
  <a href="https://emacs-ng.github.io/emacs-ng"><strong>homepage</strong></a> •
  <a href="https://emacs-ng.github.io/emacs-ng/main-features/"><strong>features</strong></a> •
  <a href="https://emacs-ng.github.io/emacs-ng/getting-started/"><strong>getting started</strong></a> •
  <a href="https://emacs-ng.github.io/emacs-ng/using-deno/"><strong>using deno</strong></a> •
  <a href="https://emacs-ng.github.io/emacs-ng/faq"><strong>FAQ</strong></a>
</p>
<hr>

## Overview

emacs-ng is based off of the `native-comp` branch of emacs, and regularly merges
in the latest from that branch.

The last merged commit is `978afd788f` by Andrea Corallo (Thu Apr 1 2021).

## Motivation

The goal of this fork is to explore new development approaches. To accomplish
this, we aim to maintain an inclusive and innovative environment. The project is
not about replacing elisp with a more popular language like Javascript. We just
want to make emacs more approachable for people who don't like lisp as much as
we do.

Contributions are welcome from anyone and we are always happy to invite new
people to the project. We are open towards interesting ideas to make emacs
better. Our only request is that you open an issue before starting work and be
willing to take feedback from the core contributors.

## Why Emacs-ng

Emacs-ng is an additive native layer over emacs, bringing features like Deno's
Javascript and Async I/O environment, Mozilla's Webrender (experimental opt-in
feature), and other features in development. emacs-ng's approach is to utilize
multiple new development approaches and tools to bring Emacs to the next
level. emacs-ng is maintained by a team that loves Emacs and everything it
stands for - being totally introspectable, with a fully customizable and free
development environment. We want Emacs to be a editor 40+ years from now that
has the flexibility and design to keep up with progressive technology.

## Why JavaScript

One of emacs-ng's primary features is integrating the [Deno
Runtime](https://deno.land/), which allows execution of JavaScript and
Typescript within Emacs. The details of that feature are listed below, however
many users would ask themselves **WHY JAVASCRIPT?** JavaScript is an extremely
dynamic language that allows for a user to inspect and control their scripting
environment. The key to note is that bringing in Deno isn't JUST JavaScript -
it's an ecosystem of powerful tools and approaches that Emacs just doesn't have
currently.

* TypeScript offers an extremely flexible typing system, that allows to user to
  have compile time control of their scripting, with the flexibility of types
  "getting out of the way" when not needed.
* Deno uses Google's v8 JavaScript engine, which features an extremely powerful
  JIT and world-class garbage collector.
* Usage of modern Async I/O utilizing Rust's Tokio library.
* Emacs-ng has WebWorker support, meaning that multiple JavaScript engines can
  be running in parallel within the editor. The only restriction is that only
  the 'main' JS Engine can directly call lisp functions.
* Emacs-ng also has WebAssembly support - compile your C module as WebAsm and
  distribute it to the world. Don't worry about packaging shared libraries or
  changing module interfaces, everything can be handled and customized by you
  the user, at the scripting layer. No need to be dependent on native
  implementation details.

### Performance

v8's world-class JIT offers the potential for large performance gains. Async I/O
from Deno, WebWorkers, and WebAsm, gives you the tools to make Emacs a smoother
and faster experience without having to install additional tools to launch as
background processes or worry about shared library versions.


## Contributing

Contributions are welcome. We try to maintain a list of "new contributor"
friendly issues tagged with "good first issue".
