# MDRV fork changes

This fork (github.com/mdrv/gpui-ce) carries the patches the
[mdrv-ds](../../mdrv-ds) suite needs on top of
[gpui-ce/gpui-ce](https://github.com/gpui-ce/gpui-ce). It exists because
these changes are hard to keep as out-of-tree patches and (so far) have
not been merged upstream.

## Patches

### `Window::set_keyboard_interactivity` (wayland layer-shell)

Runtime keyboard-interactivity switching for layer-shell windows: overlays
need to take the keyboard when shown and hand it back to the compositor
when hidden — without re-creating the window.

- `crates/gpui/src/platform.rs` — `PlatformWindow::set_keyboard_interactivity`
  trait method (default no-op, linux/wayland only).
- `crates/gpui/src/window.rs` — public `Window::set_keyboard_interactivity`.
- `crates/gpui_linux/src/linux/wayland/window.rs` — real implementation:
  sets the layer-surface keyboard mode and commits immediately.

### `Window::set_margin` (wayland layer-shell)

Runtime margin updates for layer-shell windows, CSS order (top, right,
bottom, left). On a surface with no anchors the margin _is_ its position,
so this is how apps implement free placement and dragging of floating
panels (e.g. sticky notes) that must stay on `Layer::Top`.

- `crates/gpui/src/platform.rs` — `PlatformWindow::set_margin` trait
  method (default no-op, linux/wayland only).
- `crates/gpui/src/window.rs` — public `Window::set_margin`.
- `crates/gpui_linux/src/linux/wayland/window.rs` — real implementation:
  stages the layer-surface margins; they land with the next presented
  frame. **No explicit commit on purpose**: a commit here would also apply
  any pending `Window::resize` size against the _old_ buffer, which the
  compositor then scales for a frame (rounded borders smear). Stage and
  `cx.notify()` — the present commit carries margins, size and buffer
  atomically.

### Synchronous wayland window resize (tag `mdrv-gpui-0.0.260925.4`)

Per-frame programmatic `Window::resize` on layer surfaces (edge-drag
resizing a floating panel):

- `crates/gpui_linux/src/linux/wayland/window.rs` — the trait `resize`
  stages the layer-surface size (`set_geometry`) and then applies the
  client-side resize (wgpu surface + drawable, via `set_size_and_scale`)
  **synchronously**; only the gpui-core resize callback is fired from a
  spawned task. Both halves are load-bearing:
  - _Deferred drawable resize_ (upstream behavior) raced the frame: the
    staged size could commit with a buffer still drawn at the old size,
    and the compositor scaled it for a frame (rounded borders smeared
    into straight lines).
  - _The callback_ must stay deferred: it re-enters the App via
    `AsyncApp::update`, which deadlocks when fired mid-update (tag .3
    fired it synchronously and froze the whole UI thread while the
    daemon's other threads stayed alive).

### `vendor/arrayref` (pinned 0.3.9)

`arrayref` is a transitive dependency (via `tiny-skia`). The pin is vendored
in-fork so every mdrv-ds app can `[patch.crates-io]` point here instead of
keeping per-tree copies, and builds stay deterministic/offline-friendly.

### `image` exact pin (=0.25.10)

The workspace reqs `image = "=0.25.10"` on purpose. Git-dep consumers
generate their own `Cargo.lock`; with a caret req they drifted to versions
missing/renaming `into_raw_bgra` and the fork stopped compiling
(mdrc's upperadd hit 0.25.9, impin hit the newer drift). An exact req
makes every consumer lock resolve the tested version.

### macOS backend fixes (tag `mdrv-gpui-0.0.260925.7`)

One commit (`macos-port`) making the AppKit backend usable for
chrome-less overlay PopUps. The last three are latent crashes that only
fire on macOS builds (their arms are cfg'd out elsewhere):

- `Window::set_position` — `PlatformWindow` trait method (default
  no-op; Wayland placement stays margin-based) plus the public wrapper
  in `crates/gpui/src/window.rs`. The `MacWindow` impl converts
  GPUI-global top-left-origin logical px to Cocoa
  (`setFrameTopLeftPoint`, y-flip via primary-screen `maxY`),
  executor-spawned like `resize`.
- `MacDisplay::bounds` — real `CGDisplayBounds` origins (multi-display
  global topology, secondaries may be negative) instead of the stubbed
  `(0,0)`; anything mapping displays to global positions needs this.
- Borderless `WindowKind::PopUp` — `titlebar: None` re-styles the
  NSPanel to `Borderless | NonactivatingPanel` (canonical
  floating-palette recipe); previously traffic lights rendered through
  the hidden-titlebar styling.
- `NSTrackingArea` init msg_send declared as returning `ObjcId`, not
  `()` — objc2's debug encoding check panicked (`expected '@', found
  'v'`) on the first PopUp open.
- `set_window_cursor_style` debug assert widened to `Paint | Prepaint`:
  views' `Render::render` runs in `DrawPhase::Prepaint` on this fork
  (`draw_roots` sets `Paint` only after layout+prepaint), so the
  strict-`Paint` assert fired on every debug-build drag, on every
  platform.

### GPUIApplication ivars class-check (tag `mdrv-gpui-0.0.260929.1`)

`[GPUIApplication sharedApplication]` returns any existing shared
NSApplication *regardless of its actual class*. `MacPlatform::run`
unconditionally wrote `ivars().platform` through the returned object, so an
app that had instantiated plain `NSApplication` before `application().run()`
(e.g. `NSApplication::sharedApplication` + `setActivationPolicy(.Accessory)`
for accessory mode) got those writes past the end of the smaller allocation:
silent heap corruption surfacing much later as malloc-zone aborts / SEGVs in
innocent allocators (impin daemon, ~50% of rapid restarts). Found with guard
malloc (`DYLD_INSERT_LIBRARIES=/usr/lib/libgmalloc.dylib`), which catches the
*writer* instead of a victim. `MacPlatform::run` now asserts the shared app
is actually a `GPUIApplication` (loud message pointing at
`Application::with_activation_policy`, the supported way to set the policy).
App-side rule: **never call AppKit entry points that instantiate
NSApplication before `application().run()`**.

### Sprite half-texel UV inset (tag `mdrv-gpui-0.0.260929.2`)

`atlas_texture_coordinates` (crates/gpui_render/src/shaders/common.rs) mapped
the sprite quad's unit square linearly onto the atlas tile's full pixel span.
With the linear sampler, the edge pixels' bilinear footprint then crosses the
tile boundary and blends in *neighboring atlas texels*: at integer scales
pixel centers align with texel centers so nothing shows, but at a fractional
scale (an image at a non-integer zoom, a fractional-DPI icon) the sprite edge
grows a stray 1px line colored by whatever is adjacent in the atlas (often a
white glyph). Found in impin: a white hairline across a pinned image's bottom
edge at certain zooms.

Fix: inset the UV mapping by half a texel — unit 0/1 map to the centers of
the first/last texels, so sampling can never leave the tile. Integer scales
stay pixel-exact (texel centers land on pixel centers either way). Applies to
all sprite kinds (monochrome, polychrome, underlay) on both the Metal
(gpui_apple) and wgpu (gpui_wgpu) renderers, which share the shader module.

## Branch policy

`main` carries the MDRV patches (consumers path-depend on the working
tree — a patch branch would silently break them on checkout). Sync with
upstream via:

    git fetch upstream && git merge upstream/main

Keep patches minimal and re-submit upstream when feasible; drop them from
this file when they land.

## Dependency convention (since the 2026-08-31 upstream sync)

Upstream renamed the packages: `crates/gpui` is package **`gpui-ce`**
(lib name still `gpui`, so `use gpui::…` is unchanged) and
`crates/gpui_platform` is package **`gpui_ce_platform`**. Consumers
therefore need `package=` keys:

    [dependencies]
    gpui = { path = "/g/gpui-ce/crates/gpui", package = "mdrv-gpui-ce" }
    gpui_platform = { path = "/g/gpui-ce/crates/gpui_platform", package = "mdrv-gpui-platform", features = ["wayland", "x11"] }

## Registry / portability (2026-09-25)

Eleven support leaves are published to crates.io as
`mdrv-gpui-{derive-refineable,macros,media,path,refineable,util,collections,sum-tree,scheduler,shared-string,zed-util}`
@ `0.0.260925` (squat-secured). The core family is **not**
registry-publishable — same as upstream, whose `gpui_ce_render`/`gpui_ce_platform`
are 404 on the index despite `publish = true`: `wgsl-rs` is a git dep
(taints render/apple/wgpu/windows), platform depends on those, and
gpui-ce dev-depends on platform (41 examples). Portability policy is
**git tags** instead:

    [dependencies]
    gpui = { package = "mdrv-gpui-ce", git = "https://github.com/mdrv/gpui-ce", tag = "mdrv-gpui-0.0.260925" }
    gpui_platform = { package = "mdrv-gpui-platform", git = "https://github.com/mdrv/gpui-ce", tag = "mdrv-gpui-0.0.260925", features = ["wayland"] }

    [patch.crates-io]
    arrayref = { git = "https://github.com/mdrv/gpui-ce" }

Tag per release as `mdrv-gpui-0.0.<version>`; `vendor/arrayref` is a
workspace member since `55d7e50751` so the git-form patch resolves.

Full sync procedure: `/x/m/v270/gpui-ce/50-upstream-sync.md`.

## Consumers

mdrv-ds-clock, mdrv-ds-launcher, mdrv-ds-legend, mdrv-ds-overlay,
mdrv-ds-shell — all path-dep `/g/gpui-ce/crates/*` (only
launcher/clock/overlay use `gpui_platform` directly). The former
mdrv-ds-{audio,settings,notify,battery} satellite crates were merged
into mdrv-ds-overlay on 2026-08-31; their CLI binaries survive as
`src/bin/*` in that repo (same names, same socket protocol).
