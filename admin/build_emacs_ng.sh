#!/bin/bash

if [[ -z $@ ]]; then
    echo "Usage: ./build_emacs_ng.sh <emacs_version> [<build_flags>]"
fi

render_libs="librsvg2-dev libxpm-dev libjpeg-dev libtiff-dev libpng-dev libgif-dev libgtk-3-dev libharfbuzz-dev"
render_deps=",librsvg2-2,libxpm4,libjpeg9,libtiff5,libgif7,libpng16-16,libgtk-3-0,libharfbuzz0b"

if [[ ${@:2} == *--with-webrender* ]]; then
    render_libs="${render_libs} libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev"
    render_deps="${render_deps},libxcb1,libxcb-render0,libxcb-shape0,libxcb-xfixes0"
fi

sudo add-apt-repository -y ppa:ubuntu-toolchain-r/ppa
sudo apt update
sudo apt install -y dpkg-dev autoconf make texinfo $render_libs libgnutls28-dev \
     libncurses5-dev libsystemd-dev libjansson-dev libgccjit-9-dev gcc-9 g++-9 libxt-dev \
     libclang-10-dev curl

sudo update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-9 9
sudo update-alternatives --install /usr/bin/g++ g++ /usr/bin/g++-9 9

./autogen.sh

arch=$(dpkg-architecture -q DEB_BUILD_ARCH)
pkg_name=emacs-ng_$1_$arch
deb_dir=$(pwd)/$pkg_name
mkdir -p $deb_dir/usr/local/

echo arch=$arch
echo deb_dir=$deb_dir
echo pkg_name=$pkg_name

./configure CFLAGS="-Wl,-rpath,shared,--disable-new-dtags -g -O2" \
            --prefix=/usr/local/ \
            --with-json --with-modules --with-harfbuzz --with-compress-install \
            --with-threads --with-included-regex --with-zlib --with-cairo --with-libsystemd \
            --with-rsvg --with-native-compilation ${@:2}\
            --without-sound --without-imagemagick --without-makeinfo --without-gpm --without-dbus \
            --without-pop --without-toolkit-scroll-bars --without-mailutils --without-gsettings \
            --with-all

make -j$(($(nproc) * 2)) NATIVE_FULL_AOT=1
# NOTE: use `checkinstall` will make `make install` hangs.
# See https://github.com/emacs-ng/emacs-ng/issues/364
make install-strip DESTDIR=$deb_dir

# create control file
mkdir -p $deb_dir/DEBIAN

cat > $deb_dir/DEBIAN/control << EOF
Package: emacs-ng
Version: $1
Architecture: $arch
Maintainer: https://emacs-ng.github.io/emacs-ng/
Description: A new approach to Emacs - Including TypeScript, Threading, Async I/O, and WebRender
Depends: libjansson4,libncurses5,libgccjit0${render_deps}
EOF

dpkg-deb --build -z9 --root-owner-group $deb_dir
