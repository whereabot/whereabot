# Whereabot brand

The master is `whereabot-app-icon.svg`. Everything else is generated from it. Never edit a derived file.

| File | Use |
| --- | --- |
| `whereabot-app-icon.svg` | Master. "Night Paper Map": a white robot location pin over a folded paper map at night. Square, 1024 px, full bleed, opaque (iOS needs it). |
| `whereabot-mark.svg` | Derived. The robot pin cropped from the app icon, as a 128 px square tile. Headers, favicons, inline use. The full scene is unreadable at 16 px; the crop is not. |
| `whereabot-wordmark.svg` | Derived. Mark plus the name, text outlined from a bundled font. |

## Regenerate

```sh
(cd scripts && npm ci) && (cd app && npm ci)
node scripts/generate-brand-assets.mjs          # rewrite derived files
node scripts/generate-brand-assets.mjs --check  # CI: fail if a committed file is stale
```

The script (`scripts/generate-brand-assets.mjs`) is the source of truth for which files are derived.
Commit the output, so builds need no generator.

## Colours

The icon is a rendered scene with gradients and glows, so it has no flat palette. Its night sky runs from `#05081A`
to `#2549C0`; the pin is white. Pages and the social image use the design tokens in `design-system/color-tokens.css`
(background: `--color-bg`).

## Usage rules

- Clear space: at least one eighth of the mark width on every side (8 px at 64 px).
- Minimum size: mark 16 px (favicon only), 32 px elsewhere. Wordmark 120 px wide.
- Use the files as they are. The mark is a dark tile; it works on light and dark backgrounds.
- Inline SVG: keep `<title>Whereabot</title>` or set `aria-label`; use `aria-hidden` only next to the name.

## Do not

- Add outlines, extra shadows or effects.
- Recolour the pin, rotate, stretch or re-crop the mark.
- Put the mark on a busy photo.
- Use the cropped mark as an app icon; use the app icon.
