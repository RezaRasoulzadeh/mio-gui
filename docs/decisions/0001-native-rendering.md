# 0001 — Native Rendering Stack

Status: accepted

Mio-GUI uses winit for native windows and input, wgpu for cross-platform GPU rendering, and WGSL shaders for rendering primitives.

This keeps the framework independent of browser and WebView runtimes while targeting Vulkan, Metal, and Direct3D 12 through one renderer API.

The decision implies ownership of layout, events, accessibility integration, text integration, widgets, and renderer lifecycle behavior.
