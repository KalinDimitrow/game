#!/usr/bin/env bash
# Pull latest commits for external skill submodules, then relink. This is the
# only script that touches the network or moves a submodule's pinned commit —
# review `git diff --submodule` on externals/skills/ before committing the result.
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

git -C "$root_dir" submodule update --remote --merge -- externals/skills

"$root_dir/scripts/sync-skills.sh"
