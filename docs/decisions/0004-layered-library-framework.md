# 0004 — Layered Library and Framework

Status: accepted

Mio-GUI remains a GUI framework, but its fundamental GUI functionality is designed as an independently usable library.

The intended dependency direction is `core <- render <- runtime <- framework`. Core owns backend-neutral geometry, layout, widgets, state, events, focus, semantics, styling contracts, and text models. Render translates core drawing descriptions to a graphics backend. Runtime owns windows, platform input, clipboard, IME, accessibility adapters, and event-loop integration. Framework provides the optional application lifecycle, routing, navigation, and conventions.

Dependencies may point only toward lower layers. Core must not expose or depend on `winit`, `wgpu`, or framework lifecycle types. Platform adapters consume core snapshots rather than placing platform objects in the widget API.

The project will enforce these boundaries within the current crate while contracts are evolving. Physical workspace crates and feature flags are introduced after the boundaries stabilize, avoiding premature packaging churn without allowing architectural coupling.
