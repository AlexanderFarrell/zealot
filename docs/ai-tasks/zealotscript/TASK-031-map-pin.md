# TASK-031: ZealotScript — Map Pin (`:::map`)

## Context
Location notes are useful in a personal wiki for travel logs, meeting venues, field notes, and
project sites. A `:::map` block embeds a map centered on given coordinates or an address,
without requiring user accounts or API keys (use a tile provider that allows anonymous access,
e.g. OpenStreetMap via Leaflet or a static map image via the OpenStreetMap tile URL).

## Goal
Parse `:::map` blocks and render an interactive map (or a static tile image fallback) at the
specified location.

## Requirements

### Syntax
```
:::map 51.5074,-0.1278
:::map 51.5074,-0.1278 zoom=14
:::map London Bridge, London
```

- First argument: `LAT,LON` (decimal degrees) or a free-form address string
- Optional: `zoom=N` (default 13)
- Closing `:::` is not needed — the block is self-contained on one line (like `:::progress`)

### Schema
- Block atom node: `map_pin` with attrs:
  - `query: string` — the raw argument (lat/lon or address)
  - `zoom: number` — default 13

### Viewer behaviour
- Render as `<div class="zealot-map" data-query="…" data-zoom="N">`
- After mount, initialise **Leaflet** (`npm install leaflet`) centered on the location
- If `query` is a `LAT,LON` string, parse directly; if it is an address, use the
  Nominatim geocoding API (`https://nominatim.openstreetmap.org/search?q=…&format=json`) to
  resolve coordinates before placing the marker (one request, cache result in memory)
- Add a single pin marker at the resolved coordinates
- Map height: fixed at `280px`; width: `100%`
- Show a "Map data © OpenStreetMap contributors" attribution (required by OSM tile licence)

### Editor behaviour
- Render as a non-editable info chip: `📍 51.5074, -0.1278 (zoom 13)` — no live map in editor

### Serializer
- If the original query was `LAT,LON`: `:::map LAT,LON zoom=N`
- If it was an address string: `:::map ADDRESS zoom=N`

## Implementation Notes
- Leaflet requires a CSS import: `import "leaflet/dist/leaflet.css"` — ensure this is included
  in the `packages/content` bundle or imported in the view component
- Leaflet markers need their icon image URLs configured (`L.Icon.Default.mergeOptions(…)`) when
  bundled via Vite/webpack — add the necessary asset config
- Consider a dynamic import of Leaflet to keep the main bundle small: `const L = await import("leaflet")`
- Nominatim: set a `User-Agent` request header identifying the app (required by Nominatim policy)
- Geocoding is only triggered for non-coordinate queries; skip for `LAT,LON` inputs

## Dependencies
- TASK-017 (parser infrastructure)
- TASK-018 (serializer)

## Files Likely Involved
- `packages/ui/src/zealotscript/schema.ts` — `map_pin` node spec
- `packages/ui/src/zealotscript/parse/parse_map_pin.ts` — new file
- `packages/ui/src/zealotscript/parser.ts` — register in `multiblockTypes`
- `packages/ui/src/zealotscript/serializer.ts` — serialize `map_pin`
- `packages/ui/src/zealotscript/zealotscript_view.ts` — Leaflet init after mount
- `packages/content/src/css/` — `.zealot-map`
