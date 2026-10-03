# mdrv-gpui

> [!CAUTION]
> **Personal fork, not for general use.**
> This is my private working copy of GPUI, maintained by and for me (mdrv).
> Development is heavily LLM-assisted. Most commits are written with AI
> agents and verified only against my own apps. No support, no stability
> guarantee, no release schedule. If you're not me, you'll have a far better
> time with [Zed's GPUI](https://gpui.rs) or
> [GPUI-CE](https://github.com/gpui-ce/gpui-ce) instead.

## What's in the fork

See [MDRV.md](MDRV.md) for the full log. Highlights:

- Crate `mdrv-gpui` (lib name stays `gpui`); platform crates prefixed
  `mdrv-gpui-*`. Not published to crates.io; consumers pin a git tag.
- A render-scale knob in `gpui_wgpu` (`set_render_scale`, 0.25 to 1.0) with
  scene-space fragment coordinates, for mobile fill-rate headroom.
- Android keyboard and BACK-button handling (consumed via
  [mdrv-gpui-mobile](https://github.com/mdrv/mdrv-gpui-mobile)), layer-shell
  focus work for the mdrv-ds overlay, emoji font allowlist.
- Versioning is YYM. `0.271.0` is January 2027, `0.272.0` is February 2027,
  and the patch number bumps within a month. October to December is the
  exception: the whole quarter shares one number, so `0.270.0` is Oct-Dec
  2026 and `0.280.0` is Oct-Dec 2027.

## Using it (if you insist)

```toml
[dependencies]
gpui = { package = "mdrv-gpui", git = "https://github.com/mdrv/mdrv-gpui", tag = "mdrv-gpui-0.270.0", default-features = false }
```

The apps that pin it run it daily on Linux, Android and wasm.

## License

Apache-2.0, same as upstream ([LICENSE.md](LICENSE.md)). The warning at the
top is a social notice, not a license change.
