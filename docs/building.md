These are instructions to build emacs-ng on your workstation. First [check the requirements][compile-req].

[compile-req]: https://emacs-ng.github.io/emacs-ng/getting-started/index.html#requirements

```
$ ./autogen.sh
$ ./configure --enable-rust-debug
$ make -j 8 # or your number of cores
```

For a release build, don't pass `--enable-rust-debug`.

The Makefile obeys cargo's RUSTFLAGS variable and additional options can be
passed to cargo with CARGO_FLAGS.

For example:

``` bash
$ make CARGO_FLAGS="-vv" RUSTFLAGS="-Zunstable-options --cfg MARKER_DEBUG"
```

If you want to install it, just use

```bash
make install
```

You may need to run `sudo make install` depending on your system configuration.

Now emacs should be available at `./src/emacs`. We can launch the application
via `./src/emacs`. We can navigate to the lisp scratchpad by pressing C-x b and
hitting enter.

### Using Docker

If you want to build packages without installing all the tooling, you can use a Docker container. Our CI builds [releases for Ubuntu 20](https://github.com/emacs-ng/emacs-ng/releases), you can find instruction on how to build with Docker [at this link](https://github.com/emacs-ng/emacs-ng/tree/master/docker).
