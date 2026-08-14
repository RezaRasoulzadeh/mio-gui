# Mio-GUI Development Checklist

Mio-GUI is a native, cross-platform Rust GUI framework with RTL and LTR as equal first-class layout directions. DaisyUI is the reference for component variety and design vocabulary, not an implementation dependency.

Every phase is gated. Do not start the next phase until the current phase's acceptance checks pass on all currently supported platforms.

## Project principles

- [ ] Rust-native rendering with no browser or WebView dependency
- [ ] Windows, Linux, and macOS as first-class targets
- [ ] RTL and LTR are equal, cascading layout directions
- [ ] Direction can be inherited, overridden, and nested within any subtree
- [ ] Rendering primitives remain direction-neutral
- [ ] Components use logical semantics and resolve them through local direction
- [ ] Mixed-direction text is shaped by a Unicode-compliant text engine
- [ ] Logical properties are used internally: start/end, not left/right
- [ ] Keyboard navigation, focus order, icons, and animations respect direction
- [ ] Components are accessible, themeable, deterministic, and testable
- [ ] Concrete application use cases drive scope before speculative framework features
- [ ] Each primitive has automated tests and explicit acceptance criteria
- [ ] Only `TODO` and filename comments are used in source code

## Phase 0 — Engineering baseline

- [x] Define the minimum supported Rust version
- [x] Pin dependency versions intentionally
- [x] Establish `cargo fmt` and `cargo clippy` checks
- [x] Establish unit, integration, render, and example test locations
- [x] Add CI for Linux, Windows, and macOS
- [x] Define supported GPU backends and fallback behavior
- [x] Define error-reporting and logging policy
- [x] Record architectural decisions in short decision documents

### Gate

- [ ] A clean checkout passes all automated checks on every target platform

## Phase 1 — Window and renderer foundation

### Window lifecycle

- [x] Create a window with `winit`
- [x] Initialize adapter, device, queue, surface, and render pipeline
- [x] Handle redraw requests
- [x] Handle non-zero resize events
- [x] Handle minimize, restore, suspend, and resume
- [x] Recover from lost, outdated, and suboptimal surfaces
- [x] Report unrecoverable GPU errors rather than silently ignoring them
- [ ] Validate behavior on mixed-DPI and multi-monitor setups

### Coordinate and color systems

- [x] Define logical pixels, physical pixels, scale factors, and conversions
- [x] Choose and document pixel-center and rectangle-boundary conventions
- [x] Define rectangle containment and edge-inclusion rules
- [x] Use linear color internally with explicit sRGB conversion boundaries
- [x] Define alpha and compositing conventions
- [x] Confirm identical sizing after resize and DPI changes

### Rounded rectangle primitive

- [x] Draw a filled rounded rectangle with an analytic SDF
- [x] Calculate distance in physical pixel space
- [x] Use derivative-based analytic antialiasing
- [x] Pass position, size, radius, and color from Rust
- [x] Clamp radius to half of the shortest side
- [x] Support independent corner radii
- [x] Support border width and border color
- [x] Support clipping without cutting off antialias coverage
- [x] Define behavior for zero size, subpixel size, and zero radius
- [x] Define behavior for fractional position and fractional scale factors
- [x] Batch multiple rectangles in one render pass

### Rounded rectangle tests

- [x] Unit-test radius clamping and geometry calculations
- [x] Render golden images for zero, small, medium, maximum, and oversized radii
- [x] Test square, wide, tall, tiny, and fractional-coordinate rectangles
- [x] Test 1.0, 1.25, 1.5, 2.0, and 3.0 scale factors
- [x] Verify horizontal and vertical symmetry pixel by pixel
- [x] Verify all four corners have identical alpha coverage
- [x] Compare rendered boundaries against a CPU-generated reference mask
- [x] Define an allowed per-channel pixel tolerance
- [x] Store renderer name and backend with golden-test results
- [ ] Check output on Vulkan, Metal, and DX12 before freezing the primitive

### Gate

- [x] Rounded rectangle parameters come from Rust rather than shader constants
- [x] Automated pixel tests pass at every required size and scale factor
- [ ] Manual checks pass on Linux, Windows, and macOS
- [ ] Step 1 is declared complete only after these checks pass

## Phase 2 — Text and font system

### Font infrastructure

- [x] Integrate `cosmic-text`
- [x] Load bundled and system fonts
- [x] Resolve missing requested families deterministically to bundled Vazirmatn
- [x] Expose source-range-safe missing-glyph diagnostics for fallback coverage
- [x] Bundle deterministic cross-script fallback families beyond Vazirmatn coverage
- [x] Cache shaped runs with bounded storage and font-change invalidation
- [x] Cache rasterized glyphs for atlas reuse
- [x] Build and update a GPU glyph atlas
- [x] Handle atlas eviction without visible corruption
- [x] Support font size, weight, style, line height, and letter spacing

### Unicode and bidi

- [x] Shape Persian contextual forms correctly
- [x] Shape Arabic contextual forms correctly
- [x] Resolve automatic Unicode bidi paragraphs
- [x] Render mixed Persian, English, numbers, and punctuation
- [x] Render mixed Arabic, English, numbers, and punctuation
- [x] Support explicit LTR, RTL, and automatic base direction
- [x] Preserve combining marks in shaped clusters
- [x] Preserve extended grapheme clusters during interaction
- [x] Handle neutral characters and paired punctuation correctly
- [x] Apply mirrored glyphs where Unicode requires them
- [x] Preserve source digits by default without assuming a global digit set
- [x] Add explicit locale-driven digit formatting outside text shaping

### Text interaction

- [x] Map pointer positions to shaped-cluster-aware text positions
- [x] Map pointer positions to extended grapheme-aware text positions
- [x] Map shaped-cluster text positions to visual caret positions
- [x] Implement bidi-aware selection-range geometry
- [x] Implement shaped-cluster caret movement in logical and visual directions
- [x] Implement grapheme-safe copy, cut, and paste editing operations
- [x] Connect copy, cut, and paste to platform clipboards
- [x] Connect platform input methods through `winit` IME events
- [x] Implement platform-independent composition/pre-edit state
- [x] Connect composition/pre-edit state to platform IME events

### Tests and gate

- [x] Maintain corpus tests for Persian, Arabic, English, and mixed-direction text
- [x] Add deterministic raster-mask goldens using bundled test fonts
- [x] Compare shaping output with known-good reference applications
  - [x] Add a deterministic headless shaping report and comparison checklist
  - [x] Record independent reference results for Persian, Arabic, and mixed-direction fixtures
- [x] Lock bundled-font line metrics for cross-platform CI comparison
- [x] Phase passes only when shaped Persian text and mixed-direction text are correct

## Phase 3 — Core geometry and layout

### Geometry

- [x] Implement point, size, rectangle, edges, constraints, and transforms
- [x] Separate logical and physical coordinate types
- [x] Define rounding rules for layout-to-render conversion
- [x] Implement overflow and clipping primitives

### Direction-aware layout

- [x] Implement cascading `Direction::Ltr` and `Direction::Rtl`
- [x] Use inline/block and start/end concepts internally
- [x] Implement row and column layout
- [x] Implement gap, padding, margin, min/max size, and alignment
- [x] Reverse row placement in RTL without reversing semantic child order
- [x] Resolve start/end alignment from inherited direction
- [x] Mirror logical padding and margin correctly
- [x] Keep explicitly physical properties available for exceptional cases
- [x] Define nested LTR-inside-RTL and RTL-inside-LTR behavior

### Tests and gate

- [x] Unit-test every logical-to-physical mapping
- [x] Mirror complete layout fixtures and compare their geometry
- [x] Test nesting, overflow, fractional sizes, and constrained layouts
- [x] Phase passes only when an LTR fixture and its RTL mirror are structurally equivalent

## Phase 4 — Runtime, widget tree, and events

- [x] Define retained widget tree and stable widget identity
- [ ] Define state ownership and update/message flow
- [ ] Implement layout, paint, hit-test, and event phases
- [ ] Implement invalidation and partial redraw rules
- [ ] Implement pointer capture
- [ ] Implement hover, press, release, drag, scroll, and touch events
- [ ] Route events through nested and clipped widgets
- [ ] Establish deterministic stacking and hit-test order
- [ ] Prevent layout and paint mutation from producing inconsistent frames

### Gate

- [ ] Nested interactive rectangles receive correct events after resize and DPI changes
- [ ] Event routing and hit-testing have automated tests

## Phase 5 — Focus, keyboard, and accessibility

- [ ] Implement focusable, disabled, hidden, and inert states
- [ ] Implement tab order independently from visual mirroring
- [ ] Implement direction-aware arrow navigation
- [ ] Draw a themeable focus indicator
- [ ] Restore focus safely after tree changes
- [ ] Define semantic roles, names, values, states, and actions
- [ ] Connect to platform accessibility APIs
- [ ] Support keyboard-only operation for every interactive component
- [ ] Respect reduced motion, contrast, and platform text settings where available

### Gate

- [ ] A representative form is usable with keyboard and screen reader in both directions

## Phase 6 — Theme and style system

- [ ] Define semantic color tokens rather than component-specific raw colors
- [ ] Define spacing, radius, typography, border, elevation, and motion tokens
- [ ] Support light and dark themes
- [ ] Support runtime theme switching
- [ ] Define component size and visual variants
- [ ] Resolve state styles for hover, active, focus, disabled, selected, and error
- [ ] Ensure themes meet contrast targets
- [ ] Keep styling independent from layout direction except where semantically required

### Gate

- [ ] One component gallery renders consistently in LTR, RTL, light, and dark modes

## Phase 7 — Foundational widgets

- [ ] Text
- [ ] Icon and image
- [ ] Spacer and divider
- [ ] Container and surface
- [ ] Row, column, stack, and scroll view
- [ ] Button and icon button
- [ ] Checkbox, radio, switch, and slider
- [ ] Text input, text area, and search input
- [ ] Select, dropdown, menu, and context menu
- [ ] Tooltip, popover, modal, and drawer

For every widget:

- [ ] Define anatomy and public API
- [ ] Define states and keyboard interaction
- [ ] Define LTR and RTL behavior
- [ ] Define accessibility semantics
- [ ] Add unit and interaction tests
- [ ] Add visual golden tests for states, themes, directions, and scale factors
- [ ] Add a minimal example

## Phase 8 — DaisyUI-inspired component coverage

### Actions

- [ ] Button
- [ ] Dropdown
- [ ] Modal
- [ ] Swap
- [ ] Theme controller

### Data display

- [ ] Accordion and collapse
- [ ] Avatar
- [ ] Badge
- [ ] Card
- [ ] Carousel
- [ ] Chat bubble
- [ ] Countdown
- [ ] Diff
- [ ] Kbd
- [ ] Stat
- [ ] Table
- [ ] Timeline

### Navigation

- [ ] Breadcrumbs
- [ ] Dock or bottom navigation
- [ ] Link
- [ ] Menu
- [ ] Navbar
- [ ] Pagination or join group
- [ ] Steps
- [ ] Tabs

### Feedback

- [ ] Alert
- [ ] Loading indicator
- [ ] Progress
- [ ] Radial progress
- [ ] Skeleton
- [ ] Toast
- [ ] Tooltip

### Data input

- [ ] Calendar and date input
- [ ] Checkbox
- [ ] Fieldset, label, and validation message
- [ ] File input
- [ ] Filter
- [ ] Radio
- [ ] Range
- [ ] Rating
- [ ] Select
- [ ] Text input and text area
- [ ] Toggle

### Layout

- [ ] Divider
- [ ] Drawer
- [ ] Footer
- [ ] Hero
- [ ] Indicator
- [ ] List
- [ ] Mask
- [ ] Stack

### Gate

- [ ] Only components validated by concrete application use cases are promoted to stable
- [ ] Stable components satisfy the common per-widget checklist
- [ ] The gallery documents supported variants without claiming full DaisyUI API compatibility

## Phase 9 — Data-intensive application components

- [ ] Validated form and form section
- [ ] Searchable and editable data table
- [ ] Sort, filter, pagination, and row selection
- [ ] Master-detail layout
- [ ] Application sidebar and responsive navigation
- [ ] Command palette
- [ ] Date, time, currency, and quantity inputs
- [ ] Persian calendar integration if required by product decisions
- [ ] Empty, loading, error, and permission-denied states
- [ ] Role-aware action presentation
- [ ] Large-list and large-table virtualization

### Gate

- [ ] Build one representative data-intensive screen end to end in both RTL and LTR
- [ ] Meet agreed frame-time, memory, accessibility, and interaction targets

## Phase 10 — Performance and reliability

- [ ] Set frame-time and memory budgets
- [ ] Measure layout, shaping, batching, upload, and GPU timings
- [ ] Cache only with explicit invalidation rules
- [ ] Batch shapes, glyphs, images, and clips efficiently
- [ ] Avoid per-frame allocations in steady state
- [ ] Stress-test large widget trees, tables, and text documents
- [ ] Test device loss and renderer recovery
- [ ] Test long-running memory behavior
- [ ] Add reproducible benchmarks and performance regression thresholds

## Phase 11 — Public API and distribution

- [ ] Separate core, renderer, widgets, theme, and platform concerns into stable boundaries
- [ ] Keep backend-specific types out of the public widget API
- [ ] Add API documentation and runnable examples
- [ ] Define feature flags and default features
- [ ] Define semantic-versioning and deprecation policy
- [ ] Audit licenses, fonts, and bundled assets
- [ ] Package representative applications for Windows, Linux, and macOS
- [ ] Publish only after representative applications validate the architecture under production-like use

## Current next actions

- [x] Replace shader constants with a Rust-side rounded-rectangle instance
- [x] Create a CPU reference rasterizer or mask generator for pixel comparisons
- [x] Add offscreen GPU capture and readback infrastructure
- [x] Add golden-image capture and comparison infrastructure
- [ ] Test radius and antialiasing across sizes and scale factors
- [ ] Confirm the primitive on Vulkan, Metal, and DX12
- [ ] Mark Phase 1 complete before integrating text
