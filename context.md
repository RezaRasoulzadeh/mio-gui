We are continuing development of Mio-GUI in:

/home/reza/Documents/GitHub/mio-gui

Project goal

Build a native cross-platform Rust GUI framework for Linux, Windows, and macOS, with RTL and LTR as equal first-class directions.

DaisyUI is the reference for component variety and design vocabulary only. Do not introduce browser, HTML, CSS, WebView, or JavaScript dependencies.

Important user preferences

- Work autonomously through bounded roadmap slices.
- I will verify logic and visuals when necessary.
- If human verification is required, write the request in bold.
- Do not stop for ordinary syntax or implementation questions.
- Preserve amber as the default primary theme color.
- Never mention BizTrace or Nexora in this framework.
- Do not overwrite unrelated user changes.
- Use `apply_patch` for source edits.
- Source comments may only be filename comments or `TODO` comments.
- Run formatting, compilation, tests, Clippy, and `git diff --check` after each completed slice.
- Continue from the roadmap rather than restarting or redesigning completed work.

Architecture

Mio-GUI remains a framework, but its foundational GUI system must also be usable as a library.

The intended dependency direction is:

core <- render <- runtime <- framework

- Core owns backend-neutral geometry, layout, widgets, drawing descriptions, state, events, focus, semantics, styling, and text models.
- Render translates core drawing descriptions to GPU operations.
- Runtime owns windows, platform events, clipboard, IME, accessibility adapters, and the event loop.
- Framework owns optional application lifecycle and conventions.
- Core contracts must not expose `wgpu`, `winit`, or framework lifecycle types.

See:

docs/decisions/0004-layered-library-framework.md

Toolchain and important pinned dependencies

- Rust minimum version: 1.87
- winit = 0.30.13
- wgpu = 30.0.0
- cosmic-text = 0.15.0
- accesskit = 0.24.1
- accesskit_winit = 0.33.2

Current roadmap state

Read `ROADMAP.md` before editing.

Completed:

- Phase 0 engineering infrastructure, except cross-platform clean-check gate.
- Phase 1 renderer automation, with remaining manual backend, mixed-DPI, and platform checks.
- Phase 2 text/font/bidi/editing/clipboard/IME system.
- Phase 3 direction-aware geometry and layout.
- Phase 4 retained runtime, frozen frames, events, hit testing, pointer capture, invalidation.
- Phase 5 focus, keyboard, semantics, AccessKit adapter.
- Phase 6 themes and styling, including its automated and manual gallery gate.
- Phase 7 foundational `Text` widget.

Current active work

Phase 7: “Icon and image”.

Already completed for image:

- `PixelFormat::{Rgba8, Alpha8}`
- Validated immutable `PixelImage`
- `PixelImageError`
- Backend-neutral `ImageDraw`
- `Image` widget
- `ImageFit::{Contain, Cover, Fill, None}`
- Logical inline/block alignment
- Inherited direction
- Explicit opt-in `mirror_in_rtl`
- Alternative-text semantics
- Decorative images are hidden from accessibility
- Image layout and draw descriptions
- Tests for fitting, alignment, mirroring, semantics, clipping, data validation, and immutable shared bytes

Relevant files:

- src/drawing.rs
- src/widgets/image.rs
- src/widgets/text.rs
- src/widgets/widget.rs
- src/widgets/mod.rs
- src/lib.rs
- ROADMAP.md

Exact next roadmap work

Continue the “Icon and image” item:

1. Add the icon model, preferably using validated alpha-mask image data and semantic theme tinting.
2. Extend the retained `Widget` enum and `WidgetLayout` with Image and Icon variants.
3. Make `WidgetFrame` emit retained `ImageDraw` commands in frozen paint order.
4. Add renderer support for `ImageDraw` without moving GPU types into core.
5. Respect image clipping and horizontal mirroring.
6. Add image/icon visual goldens.
7. Add minimal image and icon examples.
8. Only mark “Icon and image” complete after all of the above passes.

Current retained widget architecture

- `Widget` is an extensible enum currently containing `Text`.
- `WidgetFrame::build` freezes geometry, semantics, and paint descriptions using stable `WidgetId`s.
- `WidgetFrame` currently exposes:
  - `geometry`
  - `semantics`
  - `rectangles`
  - `text`
- Add an `images: Vec<ImageDraw>` output rather than coupling widgets directly to the renderer.
- Backend-neutral `RectDraw`, `TextDraw`, and `ImageDraw` live in `src/drawing.rs`.

Completed Text widget features

- Public `Text` API
- Logical alignment and inherited/overridden direction
- Grapheme-safe wrapping
- Explicit newlines and maximum-line truncation
- Source ranges preserved through layout and paint
- Static accessibility semantics
- Retained widget integration
- Renderer-ready text draws
- Bundled-font visual digests for LTR, RTL, mixed bidi, and wrapping
- Minimal example: `examples/text_widget.rs`

Theme/style state

- Linear colors with explicit sRGB boundaries
- Semantic light/dark palettes using amber primary
- Light, dark, and system modes
- Runtime platform theme changes
- Normal, increased, and reduced contrast palettes
- WCAG contrast calculations and automated assertions
- Text scaling and reduced-motion preferences
- Spacing, radii, borders, elevation, typography, and motion tokens
- Component sizes: Small, Medium, Large
- Variants: Solid, Outline, Soft, Ghost
- Deterministic hover, active, focus, disabled, selected, and error states
- Logical start/end adornments
- LTR/RTL component gallery passed human verification

Important renderer history

Resize behavior was previously problematic across Linux and macOS.

Current behavior coalesces resize events and configures the surface immediately before rendering the latest pending size. Do not return to calling `surface.configure` synchronously for every `WindowEvent::Resized`.

The initial macOS frame and current resize path were accepted after several iterations. Preserve this behavior unless a tested change is necessary.

Rounded rectangles use an analytic physical-pixel SDF with derivative antialiasing. Their smoothness and symmetry are fundamental and extensively tested.

Current validation baseline

The last complete run passed:

- 236 library unit tests
- Integration tests
- All examples
- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo test --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `git diff --check`

Use this full validation command after the next slice:

cargo fmt --all &&
cargo fmt --all -- --check &&
cargo check --all-targets &&
cargo test --all-targets &&
cargo clippy --all-targets -- -D warnings &&
git diff --check

Current worktree

The worktree was clean when this handoff was generated. Still run `git status --short` before editing and preserve any changes that appear.

How to continue

- Begin with a concise commentary update.
- Read the relevant current files before editing.
- Implement the next bounded Icon/Image slice.
- Add focused tests first.
- Run the complete validation suite.
- Update `ROADMAP.md` only for work that is genuinely complete.
- Do not ask for human verification unless visual or platform behavior cannot be validated automatically.