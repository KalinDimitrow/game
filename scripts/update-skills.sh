#!/usr/bin/env bash
# Pull latest commits for vendored skill submodules, then relink. This is the
# only script that touches the network or moves a submodule's pinned commit —
# review `git diff --submodule` on vendor/skills/ before committing the result.
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

git -C "$root_dir" submodule update --remote --merge -- vendor/skills

"$root_dir/scripts/sync-skills.sh"
