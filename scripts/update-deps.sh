#!/usr/bin/env bash
set -e

echo "Updating all dependencies..."
echo ""

# Nix flakes
echo "[1/5] Updating nix flakes..."
nix flake update |& nom

echo ""

# Rust - main crate
echo "[2/5] Updating Rust dependencies (main crate)..."
nix develop -c cargo update
nix develop -c cargo upgrade

echo ""

# Rust - Tauri crate
echo "[3/5] Updating Rust dependencies (src-tauri)..."
nix develop -c cargo update --manifest-path src-tauri/Cargo.toml
nix develop -c cargo upgrade --manifest-path src-tauri/Cargo.toml

echo ""

# Frontend
echo "[4/5] Updating frontend dependencies (pnpm)..."
cd ui-svelte
pnpm update --latest
pnpm install
cd ..

echo ""
echo "[5/5] All dependencies updated!"
echo ""
echo "Next steps:"
echo "  1. Test the build: nix develop -c cargo check"
echo "  2. Test frontend: cd ui-svelte && pnpm check && pnpm build"
echo "  3. Review changes and commit"
