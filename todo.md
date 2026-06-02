# TODO: Feature parity with tinyghettobox

## UI / Visual

1. **Play button on tile items** — add a dedicated ▶ button overlaid on folder tiles that plays the folder from its resume point (distinct from tap-to-navigate); required by CLAUDE.md play interaction design
2. **Empty state view** — show a component when a folder has no children instead of a blank content area
3. **Navbar cover art** — display the browsed folder's image in `NavbarView` alongside the title
4. **Ripple animation** — add touch-feedback ripple at the press point (separate from the aurora tile sweep)
5. **Lazy tile loading** — render first 12 tiles and append 6 more on scroll-end instead of rendering all at once

## Business Logic

6. **Skip-played-tracks resume** — when playing a folder via the ▶ button, find the first unplayed entry (or oldest `played_at`) as the resume point; CLAUDE.md specifies this as the ▶ button intent
7. **`max_volume` cap** — load a volume ceiling from `SystemConfig` and limit the volume slider upper bound to it
8. **Startup volume delay** — defer applying volume changes by 2 s if the player has not yet started

## State / Architecture

9. **`ToggleShowLogs` explicit bool** — replace the blind toggle with `ToggleShowLogs(bool)` so callers can set a specific state
10. **Log `target` field** — capture the Rust module path in `LogEntry` / `UILogEntry` and display it in the log overlay
11. **Log capacity** — raise the ring buffer cap from 300 to 1000 messages
12. **`rustls` crypto provider setup** — call `aws_lc_rs::default_provider().install_default()` at startup (required for librespot)
13. **`tokio-console` integration** — add `console_subscriber::ConsoleLayer` behind a debug-build feature flag
