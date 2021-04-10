sudo add-apt-repository -y ppa:ubuntu-toolchain-r/ppa
sudo apt update
sudo apt install -y autoconf make checkinstall texinfo libxpm-dev libjpeg-dev \
     libgtk-3-dev libgif-dev libtiff-dev libpng-dev libgnutls28-dev libncurses5-dev \
     libsystemd-dev libjansson-dev libharfbuzz-dev libgccjit-9-dev gcc-9 g++-9 libxt-dev \
     libclang-10-dev

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --profile minimal --default-toolchain nightly

sudo update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-9 9
sudo update-alternatives --install /usr/bin/g++ g++ /usr/bin/g++-9 9

./autogen.sh

./configure CFLAGS="-Wl,-rpath,shared -Wl,--disable-new-dtags" \
            --with-json --with-modules --with-harfbuzz --with-compress-install \
            --with-threads --with-included-regex --with-zlib --with-cairo --with-libsystemd \
            --with-native-compilation \
            --without-rsvg --without-sound --without-imagemagick --without-makeinfo \
            --without-gpm --without-dbus --without-pop --without-toolkit-scroll-bars \
            --without-mailutils --without-gsettings \
            --with-all
sudo make NATIVE_FULL_AOT=1 PATH=$PATH:$HOME/.cargo/bin -j$(($(nproc) * 2))
sudo checkinstall -y -D --pkgname=emacs-ng --pkgversion="$1" \
     --requires="libjpeg-dev,libxpm-dev,libgtk-3-dev,libgif-dev,libtiff-dev,libpng-dev,libjansson-dev,libharfbuzz-dev,libgtk-3-dev,libncurses5-dev,libgccjit-9-dev" \
     --pkggroup=emacs --gzman=yes --install=no\
     make install-strip
