#!/bin/bash

if [[ -z $@ ]]; then
    echo "Usage: ./build_emacs_ng.sh <emacs_version> [<build_flags>]"
fi

render_libs="librsvg2-dev libxpm-dev libjpeg-dev libtiff-dev libpng-dev libgif-dev libgtk-3-dev libharfbuzz-dev"
render_deps=",librsvg2-2,libxpm4,libjpeg9,libtiff5,libgif7,libpng16-16,libgtk-3-0,libharfbuzz0b"

if [[ ${@:2} == *--with-webrender* ]]; then
    echo "in"
    render_libs=
    render_deps=
fi

sudo add-apt-repository -y ppa:ubuntu-toolchain-r/ppa
sudo apt update
sudo apt install -y autoconf make checkinstall texinfo $render_libs libgnutls28-dev \
     libncurses5-dev libsystemd-dev libjansson-dev libgccjit-9-dev gcc-9 g++-9 libxt-dev \
     libclang-10-dev

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --profile minimal --default-toolchain $(cat rust-toolchain)

sudo update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-9 9
sudo update-alternatives --install /usr/bin/g++ g++ /usr/bin/g++-9 9

./autogen.sh

./configure CFLAGS="-Wl,-rpath,shared -Wl,--disable-new-dtags" \
            --with-json --with-modules --with-harfbuzz --with-compress-install \
            --with-threads --with-included-regex --with-zlib --with-cairo --with-libsystemd \
            --with-rsvg --with-native-compilation ${@:2}\
            --without-sound --without-imagemagick --without-makeinfo --without-gpm --without-dbus \
            --without-pop --without-toolkit-scroll-bars --without-mailutils --without-gsettings \
            --with-all

sudo make NATIVE_FULL_AOT=1 PATH=$PATH:$HOME/.cargo/bin -j$(($(nproc) * 2))
sudo checkinstall -y -D --pkgname=emacs-ng --pkgversion="$1" \
     --requires="libjansson4,libncurses5,libgccjit0${render_deps}" \
     --pkggroup=emacs --gzman=yes --install=no \
     make install-strip
