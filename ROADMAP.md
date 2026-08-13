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
- [ ] Nexora's requirements drive scope before general framework features
- [ ] Each primitive has automated tests and explicit acceptance criteria
- [ ] Only `TODO` and filename comments are used in source code

## Phase 0 — Engineering baseline

- [ ] Define the minimum supported Rust version
- [ ] Pin dependency versions intentionally
- [ ] Establish `cargo fmt` and `cargo clippy` checks
- [ ] Establish unit, integration, render, and example test locations
- [ ] Add CI for Linux, Windows, and macOS
- [ ] Define supported GPU backends and fallback behavior
- [ ] Define error-reporting and logging policy
- [ ] Record architectural decisions in short decision documents

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
- [ ] Confirm identical sizing after resize and DPI changes

### Rounded rectangle primitive

- [x] Draw a filled rounded rectangle with an analytic SDF
- [x] Calculate distance in physical pixel space
- [x] Use derivative-based analytic antialiasing
- [x] Pass position, size, radius, and color from Rust
- [x] Clamp radius to half of the shortest side
- [ ] Support independent corner radii
- [ ] Support border width and border color
- [ ] Support clipping without cutting off antialias coverage
- [ ] Define behavior for zero size, subpixel size, and zero radius
- [ ] Define behavior for fractional position and fractional scale factors
- [ ] Batch multiple rectangles in one render pass

### Rounded rectangle tests

- [x] Unit-test radius clamping and geometry calculations
- [ ] Render golden images for zero, small, medium, maximum, and oversized radii
- [ ] Test square, wide, tall, tiny, and fractional-coordinate rectangles
- [ ] Test 1.0, 1.25, 1.5, 2.0, and 3.0 scale factors
- [ ] Verify horizontal and vertical symmetry pixel by pixel
- [ ] Verify all four corners have identical alpha coverage
- [x] Compare rendered boundaries against a CPU-generated reference mask
- [x] Define an allowed per-channel pixel tolerance
- [ ] Store renderer name and backend with golden-test results
- [ ] Check output on Vulkan, Metal, and DX12 before freezing the primitive

### Gate

- [x] Rounded rectangle parameters come from Rust rather than shader constants
- [ ] Automated pixel tests pass at every required size and scale factor
- [ ] Manual checks pass on Linux, Windows, and macOS
- [ ] Step 1 is declared complete only after these checks pass

## Phase 2 — Text and font system

### Font infrastructure

- [ ] Integrate `cosmic-text`
- [ ] Load bundled and system fonts
- [ ] Implement deterministic fallback chains
- [ ] Cache shaped runs and glyphs
- [ ] Build and update a GPU glyph atlas
- [ ] Handle atlas eviction without visible corruption
- [ ] Support font size, weight, style, line height, and letter spacing

### Unicode and bidi

- [ ] Shape Persian and Arabic contextual forms correctly
- [ ] Resolve Unicode bidi paragraphs
- [ ] Render mixed Persian, Arabic, English, numbers, and punctuation
- [ ] Support explicit LTR, RTL, and automatic base direction
- [ ] Preserve combining marks and grapheme clusters
- [ ] Handle neutral characters and paired punctuation correctly
- [ ] Apply mirrored glyphs where Unicode requires them
- [ ] Define digit policy without assuming Persian or Latin digits globally

### Text interaction

- [ ] Map pointer positions to grapheme-aware text positions
- [ ] Map logical text positions to visual positions
- [ ] Implement bidi-aware selection ranges
- [ ] Implement caret movement in logical and visual directions
- [ ] Support copy, cut, paste, and platform input methods
- [ ] Support composition/pre-edit text

### Tests and gate

- [ ] Maintain corpus tests for Persian, Arabic, English, and mixed-direction text
- [ ] Add golden images using bundled test fonts
- [ ] Compare shaping output with known-good reference applications
- [ ] Confirm identical line metrics across supported platforms when bundled fonts are used
- [ ] Phase passes only when shaped Persian text and mixed-direction text are correct

## Phase 3 — Core geometry and layout

### Geometry

- [ ] Implement point, size, rectangle, edges, constraints, and transforms
- [ ] Separate logical and physical coordinate types
- [ ] Define rounding rules for layout-to-render conversion
- [ ] Implement overflow and clipping primitives

### Direction-aware layout

- [ ] Implement cascading `Direction::Ltr` and `Direction::Rtl`
- [ ] Use inline/block and start/end concepts internally
- [ ] Implement row and column layout
- [ ] Implement gap, padding, margin, min/max size, and alignment
- [ ] Reverse row placement in RTL without reversing semantic child order
- [ ] Resolve start/end alignment from inherited direction
- [ ] Mirror logical padding and margin correctly
- [ ] Keep explicitly physical properties available for exceptional cases
- [ ] Define nested LTR-inside-RTL and RTL-inside-LTR behavior

### Tests and gate

- [ ] Unit-test every logical-to-physical mapping
- [ ] Mirror complete layout fixtures and compare their geometry
- [ ] Test nesting, overflow, fractional sizes, and constrained layouts
- [ ] Phase passes only when an LTR fixture and its RTL mirror are structurally equivalent

## Phase 4 — Runtime, widget tree, and events

- [ ] Define retained widget tree and stable widget identity
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

- [ ] Only components required by Nexora are promoted to stable
- [ ] Stable components satisfy the common per-widget checklist
- [ ] The gallery documents supported variants without claiming full DaisyUI API compatibility

## Phase 9 — Nexora business components

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

- [ ] Build one real Nexora screen end to end in both RTL and LTR
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
- [ ] Publish only after Nexora validates the architecture in production-like use

## Current next actions

- [x] Replace shader constants with a Rust-side rounded-rectangle instance
- [x] Create a CPU reference rasterizer or mask generator for pixel comparisons
- [x] Add offscreen GPU capture and readback infrastructure
- [ ] Add golden-image capture and comparison infrastructure
- [ ] Test radius and antialiasing across sizes and scale factors
- [ ] Confirm the primitive on Vulkan, Metal, and DX12
- [ ] Mark Phase 1 complete before integrating text
