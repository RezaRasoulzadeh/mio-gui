# Errors and Diagnostics

## Policy

Recoverable runtime conditions should not terminate the application. Unrecoverable initialization, validation, and invariant failures must remain visible to developers and must never be silently converted into successful results.

Library-facing operations will return typed errors. Executable boundaries decide whether to report, retry, or terminate. Panics are reserved for violated internal invariants and tests, not expected platform or user conditions.

Diagnostic output is disabled during ordinary rendering. `MIO_GUI_DIAGNOSTICS=1` enables timestamped surface lifecycle telemetry on standard error. Diagnostic work must not change rendering decisions or timing-sensitive state.

GPU test setup and readback failures fail their tests. An unavailable adapter is not considered a passing render test.

## Current gaps

Renderer initialization returns typed surface, adapter, and device errors. Window and event-loop failures are reported at the executable boundary. Device-loss reporting and recreation are not implemented. A structured logging facade will be selected when persistent or categorized diagnostics are required.
