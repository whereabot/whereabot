# Whereabot brand

Masters live here. Everything else is generated from them. Never edit a derived file.

| Master | Use |
| --- | --- |
| `whereabot-mark.svg` | The mark: a location pin that holds a bell. Transparent background. Headers, favicons, inline use. |
| `whereabot-app-icon.svg` | Square app icon, 1024 px, solid background, no transparency (iOS needs it), mark inside the safe zone. |
| `whereabot-wordmark.svg` | Mark plus the name, text outlined. Generated from the mark and a bundled font. |

## Regenerate

```sh
(cd scripts && npm ci) && (cd app && npm ci)
node scripts/generate-brand-assets.mjs          # rewrite derived files
node scripts/generate-brand-assets.mjs --check  # CI: fail if a committed file is stale
```

The script (`scripts/generate-brand-assets.mjs`) is the source of truth for which files are derived.
Commit the output, so builds need no generator.

## Colours

Colours come from the design tokens in `design-system/color-tokens.css`. The mark uses three:
pin `#2F5BEA` (`--accent`), bell `#FFFFFF`, clapper `#FFB020`. The icon background is `#F6F7F9` (`--color-bg`).

## Usage rules

- Clear space: at least one eighth of the mark width on every side (8 px at 64 px).
- Minimum size: mark 16 px (favicon only), 32 px elsewhere. Wordmark 120 px wide.
- Use the files as they are. Keep the colours on light and dark backgrounds.
- Inline SVG: keep `<title>Whereabot</title>` or set `aria-label`; use `aria-hidden` only next to the name.

## Do not

- Add outlines, shadows, gradients or effects.
- Recolour the pin, rotate, stretch or crop the mark.
- Put the mark on a busy photo or a background close to `#2F5BEA`.
- Use the transparent mark as an app icon.
