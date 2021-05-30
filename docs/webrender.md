## Webrender

[WebRender](https://github.com/servo/webrender) is a GPU-based 2D rendering engine written in Rust from Mozilla. Firefox, the research web browser Servo, and other GUI frameworks draw with it. emacs-ng use it as a new experimental graphic backend to leverage GPU hardware.

Webrender rendering is a opt-in feature. You can enable it by this.

```
$ ./configure --with-webrender
```

If you get "Couldn't find any available vsync extension" runtime panic, enabling 3D acceleration will fixes it.
