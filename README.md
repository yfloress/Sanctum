# Sanctum Web

A static companion website for [Sanctum](https://codeberg.org/music-soul/Sanctum) — your personal finance, crypto, and habits fortress.

## Purpose

This web tool allows users to:

1. **Learn about Sanctum** — Beautiful landing page explaining the project philosophy and features
2. **Generate import files** — Create `sanctum_export.json` files without installing the desktop app
3. **Edit existing exports** — Load and append entries to existing JSON files

All data stays in your browser. Nothing is sent to any server.

## Stack

| Aspect | Technology | Why |
|--------|------------|-----|
| Core | Astro 5 | Static HTML by default, instant loading |
| UI Logic | React 19 | For the form and JSON handling (as an "Island") |
| Styling | Tailwind CSS + Shadcn/ui | Beautiful, accessible components |
| Animation | Motion + View Transitions | Lighter than GSAP, better React integration |
| Offline | Vite PWA | Edit JSONs without internet |

## Development

```bash
# Enter dev shell
nix develop

# Install dependencies
pnpm install

# Start dev server
pnpm dev

# Build for production
pnpm build

# Preview production build
pnpm preview
```

## Deployment

The `pages` branch is automatically deployed to Codeberg Pages.

Build output goes to `dist/` which is served as the static site.

## License

GPL-3.0 — See [LICENSE](LICENSE)