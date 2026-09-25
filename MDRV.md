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

Full sync procedure: `/x/m/v270/gpui/gpui-ce-sync.md`.

## Consumers

mdrv-ds-clock, mdrv-ds-launcher, mdrv-ds-legend, mdrv-ds-overlay,
mdrv-ds-shell — all path-dep `/g/gpui-ce/crates/*` (only
launcher/clock/overlay use `gpui_platform` directly). The former
mdrv-ds-{audio,settings,notify,battery} satellite crates were merged
into mdrv-ds-overlay on 2026-08-31; their CLI binaries survive as
`src/bin/*` in that repo (same names, same socket protocol).
