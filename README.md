# Sanctum Web

Offline companion for [Sanctum](https://codeberg.org/Kyronix/Sanctum). Generates and edits `sanctum_export.json` locally.

## Development

```bash
nix develop             # Environment
pnpm install            # Dependencies
pnpm approve-builds     # Enable native builds (esbuild/sharp)
pnpm dev                # Local server (open http://localhost:4321/Sanctum/)
pnpm build              # Build static site
pnpm preview            # Preview static build
```

## Deployment (Codeberg Pages)

```bash
pnpm build
git checkout pages
cp -r dist/* .
git add .
git commit -m "Deploy update"
git push
git checkout dev-pages
```

## Stack
Astro 6 (beta), React 19, Tailwind CSS 4, Motion, Vite PWA.
