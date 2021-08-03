These are instructions on how to build emacs-ng.

## Build requirements

You will need [Rust installed](https://www.rust-lang.org/en-US/install.html).
The file `rust-toolchain` indicates the version that gets installed.  This
happens automatically, so don't override the toolchain manually. IMPORTANT:
Whenever the toolchain updates, you have to reinstall rustfmt manually.

### Linux

You will need a C compiler and toolchain. On Linux, you can do something like:

    apt install build-essential automake clang libclang-dev

Additional requirements:

    apt install texinfo libjpeg-dev libtiff-dev \
    libgif-dev libxpm-dev libgtk-3-dev gnutls-dev \
    libncurses5-dev libxml2-dev libxt-dev

For native-comp you will also need `zlib1g-dev libgccjit-9-dev`.

### MacOS

On MacOS, you will need Xcode.

    brew install gnutls texinfo autoconf

To use the installed version of `makeinfo` instead of the built-in
(`/usr/bin/makeinfo`) one, you'll need to make sure `/usr/local/opt/texinfo/bin`
is before `/usr/bin` in `PATH`.

Mojave install libxml2 headers with: `open
/Library/Developer/CommandLineTools/Packages/macOS_SDK_headers_for_macOS_10.14.pkg`

If you want to use native-comp, you will need to compile with `./configure
--with-native-compilation`. nativecomp will also require:

    brew install zlib libgccjit

It seems to be more difficult to build native-comp on macOS than on Linux.
There are several tutorials that provide instructions on how to successfully
compile it (no guarantee that they work):

- https://gist.github.com/mikroskeem/0a5c909c1880408adf732ceba6d3f9ab
- https://gist.github.com/AllenDang/f019593e65572a8e0aefc96058a2d23e

## Compile and install

```
$ ./autogen.sh
$ ./configure --enable-rust-debug
$ make -j$(nproc) # proc for number of processors (cores)
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
