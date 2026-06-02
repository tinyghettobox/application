# Children's Music Player

Children-friendly music player for an embedded system with an 800×480 touch display. Targets a device that plays Spotify, HTTP radio streams, and local audio files.

## Play interaction design

Two distinct play intents triggered from different UI elements:

**Tap on tile or list row:**
- Folders → navigate into folder
- Leaf entries (File/Spotify/Stream) → flatten parent folder to ordered leaves, play tapped entry, queue everything after it

**▶ button overlaid on tile image (tile view only, folders only):**
- Flatten folder recursively to all leaf entries sorted by `sort_key`
- Find resume point: first entry with `played_at = null`, or oldest `played_at`
- Play it, queue the rest

## Queue auto-advance

On `PlayerEvent::TrackEnded`: pop front of queue → play next. If queue empty: clear playing state.

## View mode selection

Content area variant is determined by the **first child's variant** of the active library entry. Folders/Streams → tile grid. Files/Spotify → numbered list.

## Display constraints

800×480px, touch only. Font sizes ≥ 16px, touch targets ≥ 44px.
