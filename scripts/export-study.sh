#!/usr/bin/env bash
# scripts/export-study.sh — Genera study-export/ para NotebookLM
# Copia archivos relevantes del repo en un solo directorio plano,
# extrayendo solo la lógica TypeScript de los .svelte.

set -euo pipefail

OUTDIR="${1:-study-export}"

if [ -d "$OUTDIR" ]; then
  echo "Error: $OUTDIR ya existe. Bórralo o especifica otro nombre."
  exit 1
fi

mkdir -p "$OUTDIR"
echo "→ Exportando a $OUTDIR/"

# ─── Rust (src/ + src-tauri/src/ + tests/) ──────────────────────────
find src/ src-tauri/src/ tests/ -name '*.rs' | while read -r f; do
  flat=$(echo "$f" | sed 's|/|_|g')
  cp "$f" "$OUTDIR/$flat"
done
echo "  ✓ Rust: $(find src/ src-tauri/src/ tests/ -name '*.rs' | wc -l) archivos"

# ─── Svelte (solo <script>, como .md) ───────────────────────────────
find ui-svelte/src/ -name '*.svelte' | while read -r f; do
  name=$(basename "$f" .svelte)
  flat=$(echo "$f" | sed 's|^\./||; s|/|_|g; s|\.svelte$|_svelte.md|')
  script=$(sed -n '/<script/,/<\/script>/p' "$f" | sed '1d;$d')
  {
    echo "# $name"
    echo ""
    echo '```typescript'
    echo "$script"
    echo '```'
  } > "$OUTDIR/$flat"
done
echo "  ✓ Svelte: $(find ui-svelte/src/ -name '*.svelte' | wc -l) archivos (solo lógica)"

# ─── TypeScript (ui-svelte/src/) ────────────────────────────────────
find ui-svelte/src/ -name '*.ts' | while read -r f; do
  flat=$(echo "$f" | sed 's|/|_|g')
  cp "$f" "$OUTDIR/$flat"
done
echo "  ✓ TypeScript: $(find ui-svelte/src/ -name '*.ts' | wc -l) archivos"

# ─── Locales ────────────────────────────────────────────────────────
cp locales/*.ftl "$OUTDIR/"
echo "  ✓ Locales: 2 archivos"

# ─── Configs raíz ───────────────────────────────────────────────────
for f in Cargo.toml AGENTS.md flake.nix README.md README_ES.md CLAUDE.md deny.toml .envrc install.sh; do
  [ -f "$f" ] && cp "$f" "$OUTDIR/${f//\//_}"
done
echo "  ✓ Configs raíz"

# ─── Configs Tauri ──────────────────────────────────────────────────
cp src-tauri/Cargo.toml "$OUTDIR/src-tauri_Cargo.toml" 2>/dev/null || true
cp src-tauri/build.rs "$OUTDIR/src-tauri_build.rs" 2>/dev/null || true
cp src-tauri/tauri.conf.json "$OUTDIR/src-tauri_tauri.conf.json" 2>/dev/null || true
[ -f src-tauri/capabilities/default.json ] && cp src-tauri/capabilities/default.json "$OUTDIR/src-tauri_capabilities_default.json"
echo "  ✓ Configs Tauri"

# ─── Configs Frontend ───────────────────────────────────────────────
for f in package.json vite.config.ts svelte.config.js tsconfig.json .npmrc index.html; do
  [ -f "ui-svelte/$f" ] && cp "ui-svelte/$f" "$OUTDIR/ui-svelte_${f//\//_}"
done
echo "  ✓ Configs Frontend"

# ─── Docs ───────────────────────────────────────────────────────────
find docs/ -name '*.md' | while read -r f; do
  flat=$(echo "$f" | sed 's|/|_|g')
  cp "$f" "$OUTDIR/$flat"
done
echo "  ✓ Docs: $(find docs/ -name '*.md' | wc -l) archivos"

# ─── Tree del repositorio ───────────────────────────────────────────
if command -v tree &>/dev/null; then
  tree -a --charset=utf-8 \
    -I '.git|target|node_modules|.direnv|.astro|dist|.claude|packaging|assets|src-tauri/gen|*.svg|*.lock|*.png|*.ico|*.icns|*.pem|*.ttf|*.woff2|*.xlsx|*.css|*.wasm|*.dex|*.class|*.kt' \
    /app > "$OUTDIR/TREE.txt" 2>/dev/null
else
  find . -not -path './.git/*' -not -path './target/*' -not -path './node_modules/*' -not -path './.direnv/*' -not -path './study-export*' -type f | sort > "$OUTDIR/TREE.txt"
fi
echo "  ✓ TREE.txt"

# ─── Forzar extensión .txt en todo lo que no sea .md ────────────────
# NotebookLM no entiende .rs, .ts, .toml, .nix, .json, etc.
for f in "$OUTDIR"/*; do
  [ -f "$f" ] || continue
  case "$f" in
    *.md|*.txt) ;;
    *) mv "$f" "$f.txt" ;;
  esac
done
echo "  ✓ Extensiones: $(find "$OUTDIR" -name '*.txt' | wc -l) .txt, $(find "$OUTDIR" -name '*.md' | wc -l) .md"

# ─── Resumen ────────────────────────────────────────────────────────
total=$(find "$OUTDIR" -type f | wc -l)
echo ""
echo "═══ Estudio listo ═══"
echo "  Directorio: $OUTDIR/"
echo "  Archivos:   $total"
echo "  Tamaño:     $(du -sh "$OUTDIR" | cut -f1)"
echo "═══════════════════════"
