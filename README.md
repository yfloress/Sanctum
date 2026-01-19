# Sanctum Web

Offline companion for [Sanctum](https://codeberg.org/Kyronix/Sanctum). Generates and edits `sanctum_export.json` locally.

## Development

```bash
nix develop             # Environment
pnpm install            # Dependencies
pnpm approve-builds     # Enable native builds (esbuild/sharp)
pnpm dev                # Local server
pnpm build              # Build static site
```

## Deployment (Codeberg Pages)

1. Build in `dev-pages` branch: `pnpm build`
2. Push `dist/` to `pages` branch:
```bash
git add dist -f
git commit -m "Deploy update"
git subtree push --prefix dist origin pages
git reset HEAD~1        # Clean up local commit
```

## Stack
Astro 6 (Beta), React 19, Tailwind CSS, Vite PWA.