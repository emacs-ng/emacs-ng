[![](https://badges.gitter.im/emacs-ng/emacs-ng.svg)](https://gitter.im/emacsng/community)

# emacs-ng

## Motivation

The goal of this fork is to explore new development approaches.

## Features
emacs-ng has integrated the [Deno Runtime](https://deno.land/) to allow for the execution of JavaScript to control the editor. Further details and examples can be found in DENO_README.md. JS tests can be run by building the editor and executing `./src/emacs --batch -l test/js/bootstrap.el`.
