## Webrender

Webrender rendering is a opt-in feature. You can enable it by this.

```
$ ./configure --with-webrender
```

If you get "Couldn't find any available vsync extension" runtime panic, enabling 3D acceleration will fix it.
