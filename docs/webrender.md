## WebRender

WebRender rendering is a opt-in feature.

### Usage
#### WebRender with PGTK

Emacs WebRender now supports Emacs PGTK build. To use WebRender with PGTK:

```
$ ./configure --with-webrender --with-pgtk
```

#### WebRender with winit(TAO)
[Winit](https://github.com/rust-windowing/winit) is a cross-platform
window creation and event loop management
library. [TAO](https://github.com/tauri-apps/tao) is is a fork of
winit which replaces Linux's port to Gtk, adding support for
webkit2gtk, and a lot of Desktop Environment features like a menu bar,
system tray, global shortcuts etc.

We are only experimenting with them to build a Emacs window system from scratch. It works to some
extent. But more details need to be handled before using it in production environment. You've been warned!

##### Using winit
```
$ ./configure --with-webrender --with-winit
```
##### Using TAO
```
$ ./configure --with-webrender --with-winit=tao
```
### OpenGL context creation
We have implemented three means (Surfman/Glutin/Gtk3) to create a OpenGL context creation for WebRender. If the default one does not work for you. You can try with another one.
```
--with-wr-gl=[surfman|glutin|gtk3]
```

- Surfman is used by default with winit/TAO, Glutin can be used if Surfman does not work  on your device.
- Gtk3 is used by default with PGTK, and can be used with TAO on UNIX.

### Troubleshooting
#### Couldn't find any available vsync extension
If you get "Couldn't find any available vsync extension" runtime panic, enabling 3D acceleration will fix it.

#### Random crashes with winit(TAO)

Try building with `--enable-winit-pselect`

```
$ ./configure --with-webrender --with-winit=tao --enable-winit-pselect
```

#### Black screen/flickering with winit(TAO) and glutin on Linux

TAO uses gtk under the hood on Linux, so you should build with
`--with-wr-gl=gtk3` to use gtk's gl context creation and avoid the
conflict.
