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

## Consumers

mdrv-ds-audio, mdrv-ds-battery, mdrv-ds-launcher, mdrv-ds-legend,
mdrv-ds-notify, mdrv-ds-settings (all path-dep `/g/gpui-ce/crates/*`).
