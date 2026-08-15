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
- [x] Define core, render, runtime, and optional framework layers
- [x] Enforce the dependency direction `core <- render <- runtime <- framework`
- [x] Keep backend and application-lifecycle types out of core contracts
- [ ] Extract stable layers into workspace crates without changing their contracts

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
- [x] Define state ownership and update/message flow
- [x] Freeze layout geometry, clips, paint order, and hit-testing into one frame snapshot
- [x] Implement layout, paint, hit-test, and event phases
- [x] Implement invalidation and partial redraw rules
- [x] Implement pointer capture
- [x] Implement hover, press, release, drag, scroll, and touch events
- [x] Route events through nested and clipped widgets
- [x] Establish deterministic stacking and hit-test order
- [x] Prevent layout and paint mutation from producing inconsistent frames

### Gate

- [x] Nested interactive rectangles receive correct events after resize and DPI changes
- [x] Event routing and hit-testing have automated tests
- [x] Phase passes only when nested event routing remains stable across resize and DPI changes

## Phase 5 — Focus, keyboard, and accessibility

- [x] Implement focusable, disabled, hidden, and inert states
- [x] Implement tab order independently from visual mirroring
- [x] Implement direction-aware arrow navigation
- [x] Draw a themeable focus indicator
- [x] Restore focus safely after tree changes
- [x] Define semantic roles, names, values, states, and actions
- [x] Connect to platform accessibility APIs through the runtime adapter
- [x] Define platform-neutral keyboard events and route them through focused widget ancestry
- [x] Translate native `winit` keyboard input at the runtime boundary
- [x] Apply Tab and direction-aware arrow navigation from keyboard events
- [x] Translate keyboard activation and adjustment keys into validated semantic actions
### Gate

- [x] A representative form model has equivalent keyboard and semantic behavior in both directions
- [x] Focus, keyboard, semantics, and platform adapter contracts have automated tests

## Phase 6 — Theme and style system

- [x] Define centralized reduced-motion, contrast, and text-scale policies
- [x] Connect supported platform preference sources in the runtime layer
- [x] Apply platform preferences to resolved themes, typography, motion, and widget style inputs
- [x] Define semantic color tokens rather than component-specific raw colors
- [x] Define spacing, radius, typography, border, elevation, and motion tokens
- [x] Support light and dark themes
- [x] Support runtime theme switching
- [x] Define component size and visual variants
- [x] Resolve state styles for hover, active, focus, disabled, selected, and error
- [x] Ensure themes meet contrast targets
- [x] Keep styling independent from layout direction except where semantically required

### Gate

- [x] One component gallery renders consistently in LTR, RTL, light, and dark modes
  - [x] Automated offscreen GPU pixels mirror between LTR and RTL in both color schemes
  - [x] Manual gallery appearance is accepted on a supported desktop

## Phase 7 — Foundational widgets

- [x] Text
  - [x] Define public API, logical alignment, inherited direction, wrapping, and line limits
  - [x] Preserve grapheme-safe source ranges through measurement and paint descriptions
  - [x] Define static-text accessibility semantics
  - [x] Connect the widget through the retained layout and render path
  - [x] Add widget-level visual goldens and a minimal example
- [x] Icon and image
  - [x] Define validated backend-neutral pixel image resources and image paint descriptions
  - [x] Define image sizing, fitting, logical alignment, explicit RTL mirroring, and semantics
  - [x] Connect image and icon variants through retained rendering
  - [x] Add image/icon visual goldens and minimal examples
- [x] Spacer and divider
- [x] Container and surface
- [x] Row, column, stack, and scroll view
- [x] Button and icon button
  - [x] Route icon-button focus and activation through native keyboard, pointer, and accessibility paths
- [x] Checkbox, radio, switch, and slider
  - [x] Add retained radio-group exclusivity and wrapping arrow-key movement
  - [x] Give radio groups one roving Tab stop without blocking pointer or accessibility focus
  - [x] Dispatch control actions on key down without duplicating them on key release
  - [x] Complete slider arrow, edge, page, repeat, and RTL keyboard behavior
  - [x] Expose validated slider value, range, and step metadata to platform accessibility
- [x] Text input, text area, and search input
  - [x] Add grapheme-safe visual-line Up/Down movement and Shift extension for text areas
  - [x] Preserve independent selection anchors for reversible keyboard and pointer selection
  - [x] Add visual-line Home/End and primary-modified document-edge movement for text areas
  - [x] Allow caret and selection navigation in read-only fields while rejecting mutation
  - [x] Apply assistive-technology text and numeric value changes through backend-neutral actions
  - [x] Expose distinct single-line, multiline, and search-field roles to platform accessibility
  - [x] Expose editable control placeholders through core semantics and AccessKit
  - [x] Expose editable text runs, character boundaries, and selection through platform accessibility
- [x] Select, dropdown, menu, and context menu
  - [x] Close select and dropdown popups when keyboard focus leaves their trigger
  - [x] Expose select controls as combo boxes through core semantics and AccessKit
  - [x] Publish retained menu items as individually actionable accessibility nodes
  - [x] Publish open select, dropdown, and context-menu items as actionable accessibility nodes
  - [x] Track active popup items through native accessibility active descendants
  - [x] Freeze popup item bounds for native accessibility exploration and hit testing
- [x] Tooltip, popover, modal, and drawer

For every widget:

- [x] Define anatomy and public API
- [x] Define states and keyboard interaction
- [x] Verify complete keyboard-only operation
- [x] Define LTR and RTL behavior
- [x] Define accessibility semantics
- [x] Add unit and interaction tests
- [x] Add visual golden tests for states, themes, directions, and scale factors
- [x] Add a minimal example

### Gate

- [x] A representative retained form has equivalent geometry, semantics, and keyboard focus order in both directions
  - [x] Cover single-line, search, multiline, select, radio, checkbox, switch, slider, and button controls
- [x] The renderer consumes complete retained widget frames through a render-layer API
- [x] The representative form runs in a native window with keyboard dispatch and AccessKit updates
- [x] A representative rendered form is keyboard-usable in both directions
- [ ] Manually verify the representative form with platform screen readers in both directions

## Phase 8 — DaisyUI-inspired component coverage

### Actions

- [x] Button
- [x] Dropdown
- [x] Modal
- [x] Swap
- [x] Theme controller

### Data display

- [x] Accordion and collapse
- [x] Avatar
- [x] Badge
- [x] Card
- [x] Carousel
- [x] Chat bubble
- [x] Countdown
- [x] Diff
- [x] Kbd
- [x] Stat
- [x] Table
- [x] Timeline

### Navigation

- [x] Breadcrumbs
- [x] Dock or bottom navigation
- [x] Link
- [x] Menu
- [x] Navbar
- [x] Pagination or join group
- [x] Steps
- [x] Tabs

### Feedback

- [x] Alert
- [x] Loading indicator
- [x] Progress
- [x] Radial progress
- [x] Skeleton
- [x] Toast
- [x] Tooltip

### Data input

- [x] Calendar and date input
- [x] Checkbox
- [x] Fieldset, label, and validation message
- [x] File input
- [x] Filter
- [x] Radio
- [x] Range
- [x] Rating
- [x] Select
- [x] Text input and text area
- [x] Toggle

### Layout

- [x] Divider
- [x] Drawer
- [x] Footer
- [x] Hero
- [x] Indicator
- [x] List
- [x] Mask
- [x] Stack

### Gate

- [x] Only components validated by concrete application use cases are promoted to stable
- [x] Stable components satisfy the common per-widget checklist
- [x] The gallery documents supported variants without claiming full DaisyUI API compatibility

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
- [ ] Complete native-DPI text sampling and built-in icon-mask quality tuning
- [ ] Polish popup overlay sizing, trigger-only focus geometry, and viewport overflow presentation
- [ ] Stress-test large widget trees, tables, and text documents
- [ ] Test device loss and renderer recovery
- [ ] Test long-running memory behavior
- [ ] Add reproducible benchmarks and performance regression thresholds

## Phase 11 — Public API and distribution

- [ ] Extract core, render, runtime, and framework into independently consumable crates
- [ ] Keep backend-specific types out of the public widget API
- [ ] Make the high-level framework layer optional
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
