---
sidebar_position: 9
---

# Reducing App Size

A bevy_cef app ships three kinds of files: your app's own binary, the bevy_cef render process, and the CEF runtime (the Chromium Embedded Framework on macOS, `libcef` and its resources on Windows and Linux). This page covers the Rust binaries.

## Release profile for your app

Cargo keeps symbol tables in release binaries by default. Stripping them is the cheapest size win and does not change how the app runs:

```toml
[profile.release]
strip = "symbols"
```

Keep an unstripped copy of each release build, or split debug info, if you need to symbolicate crash reports.

Link-time optimization with a single codegen unit shrinks the binary further at the cost of longer release builds. On bevy_cef's render process it saved a further 19%:

```toml
[profile.release]
strip = "symbols"
lto = "fat"
codegen-units = 1
```

bevy_cef also builds its render process with `opt-level = "z"`, which trades speed for size. That suits the render process, which runs little Rust code, but usually not a Bevy app.

## Render process

Since bevy_cef 0.15.0, `bevy_cef_render_process`, installed with `cargo install bevy_cef_render_process`, is built with bevy_cef's size-oriented profile and does not depend on Bevy, so there is nothing to configure. A macOS bundle carries four copies of it, one per helper app.

On Windows (MSVC), debug info already lives in a separate `.pdb` file, so `strip` changes little there.
