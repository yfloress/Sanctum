# Sanctum Web

Offline companion for [Sanctum](https://github.com/yfloress/Sanctum). Generates and edits `sanctum_export.json` locally.

## Development

```bash
nix develop             # Environment
pnpm install            # Dependencies
pnpm approve-builds     # Enable native builds (esbuild/sharp)
pnpm dev                # Local server (open http://localhost:4321/Sanctum/)
pnpm build              # Build static site
pnpm preview            # Preview static build
```

## Deployment (GitHub Pages)

Deployment is automatic: every push to `web` triggers
[`.github/workflows/deploy-pages.yml`](.github/workflows/deploy-pages.yml),
which builds the site and deploys it as a Pages artifact. It is served at
<https://yfloress.github.io/Sanctum/>. To publish without pushing, run the
workflow from the Actions tab.

The site URL and its `/Sanctum/` base path come from `astro.config.mjs`; the
base has to match the repository name or every asset path breaks.

Repository setting required once: **Settings → Pages → Source: GitHub
Actions**. The older `pages` branch is no longer used.

## Stack
Astro 6 (beta), React 19, Tailwind CSS 4, Motion, Vite PWA.
