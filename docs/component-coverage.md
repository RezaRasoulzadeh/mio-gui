# Component coverage

Mio-GUI uses DaisyUI only as a reference for component variety and design vocabulary. The APIs, rendering, layout, interaction, and accessibility implementations documented here are native Rust contracts. HTML, CSS, JavaScript, WebView behavior, and DaisyUI API compatibility are not supported or implied.

## Promotion requirements

A Phase 8 component is promoted only when it has a public backend-neutral model, retained layout and paint integration when applicable, explicit LTR and RTL behavior, accessibility semantics, focused tests, a theme/direction/scale promotion matrix, and a runnable example or representative application use case. Static presentation and layout components have no keyboard behavior; interactive components must support keyboard-only operation.

## Supported vocabulary

| Family | Components | Supported variation | Example coverage |
| --- | --- | --- | --- |
| Actions | Button, Dropdown, Modal, Swap, Theme switcher | semantic variants and states; open/closed overlays | `buttons`, `dropdown`, `modal`, `swap_theme` |
| Data display | Accordion, Avatar, Badge, Card, Carousel, Chat bubble, Countdown, Diff, Kbd, Stat, Table, Timeline | content states, selection where applicable, light/dark and LTR/RTL | `accordion_carousel`, `avatar_card`, `badge_kbd`, `chat_diff`, `stat_countdown`, `table_timeline` |
| Navigation | Breadcrumbs, Dock, Link, Menu, Navbar, Pagination, Steps, Tabs | direction-aware selection and keyboard movement | `link_breadcrumbs`, `dock_navbar`, `menu`, `pagination_steps_tabs` |
| Feedback | Alert, Loading, Progress, Radial progress, Skeleton, Toast, Tooltip | determinate/activity states, visibility, placement | `alert_progress`, `loading_skeleton`, `radial_progress_toast`, `tooltip` |
| Data input | Calendar, Checkbox, Date input, Fieldset, File input, Filter, Radio, Range, Rating, Select, Text input/area, Toggle | enabled/disabled, selected/value/error, keyboard and semantic actions | `calendar_date_input`, `checkbox`, `fieldset_rating`, `filter_file_input`, `representative_form` |
| Layout | Divider, Drawer, Footer, Hero, Indicator, List, Mask, Stack | logical alignment, overlay/flow, circle and rounded masks | `drawer`, `footer_hero`, `image_icon`, `spacer_divider` |

The promotion matrices in `src/widgets/widget.rs` cover light and dark themes, LTR and RTL directions, normal and 150% text scales, and relevant component states. Renderer-level GPU tests separately cover retained rectangle, glyph, image, clipping, and mirror behavior.
