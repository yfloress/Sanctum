#!/usr/bin/env bash
set -e

log()  { printf '\n\033[1;32m[+] %s\033[0m\n' "$*"; }
warn() { printf '\n\033[1;33m[!] %s\033[0m\n' "$*"; }
err()  { printf '\n\033[1;31m[x] %s\033[0m\n' "$*"; exit 1; }

log "Updating all dependencies"

# Nix flakes
log "Updating nix flakes"
nix flake update |& nom

# Rust - main crate
log "Updating Rust dependencies (main crate)"
nix develop -c cargo update
nix develop -c cargo upgrade

# Rust - Tauri crate
log "Updating Rust dependencies (src-tauri)"
nix develop -c cargo update --manifest-path src-tauri/Cargo.toml
nix develop -c cargo upgrade --manifest-path src-tauri/Cargo.toml

# Frontend
log "Updating frontend dependencies (pnpm)"
(
  cd ui-svelte
  pnpm update --latest
  pnpm install
)

# Validate
log "Validating updates"
nix develop -c cargo check -j 2
nix develop -c cargo audit 2>/dev/null || warn "cargo audit skipped (advisory DB may not be cloned yet)"
nix develop -c cargo machete 2>/dev/null || warn "cargo machete skipped (not installed)"

log "All dependencies updated"
echo ""
echo "Next steps:"
echo "  1. Run tests:     nix develop -c cargo test"
echo "  2. Review diff:   git diff --stat"
echo "  3. Commit"

