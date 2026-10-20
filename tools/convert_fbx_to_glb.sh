#!/usr/bin/env bash
# Convention: any directory under raw_assets/ that directly holds .fbx
# files (a base mesh file plus separate animation-only files, all flat in
# the same dir, .zip downloads extracted in place if needed) gets each
# .fbx converted to its own standalone .glb at the same relative path
# under data/assets/ - no merging. Bevy matches animation clips to scene
# entities purely by bone name (AnimationTargetId, a hash of the bone
# name path), not by which file a clip came from, so the game loads the
# base mesh and each animation as separate glTF assets and combines them
# at runtime instead of needing them pre-merged into one file ahead of
# time - see tools/convert_single_fbx.py for why.
#
# Requires: blender on PATH (override with BLENDER_BIN).
set -euo pipefail

TOOLS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GAME_DIR="$(dirname "$TOOLS_DIR")"
RAW_DIR="$GAME_DIR/raw_assets"
ASSETS_DIR="$GAME_DIR/data/assets"
BLENDER_BIN="${BLENDER_BIN:-blender}"

# Extract any raw .zip downloads in place, next to where their .fbx goes.
find "$RAW_DIR" -type f -iname "*.zip" -print0 | while IFS= read -r -d '' zip; do
    dir="$(dirname "$zip")"
    echo "Extracting: ${zip#"$RAW_DIR"/}"
    python3 -m zipfile -e "$zip" "$dir"
done

find "$RAW_DIR" -type f -iname "*.fbx" -print0 | while IFS= read -r -d '' fbx; do
    rel="${fbx#"$RAW_DIR"/}"
    rel_dir="$(dirname "$rel")"
    name="$(basename "${fbx%.fbx}" | tr ' ' '_')"
    out="$ASSETS_DIR/$rel_dir/$name.glb"
    mkdir -p "$(dirname "$out")"
    echo "Converting: $rel -> ${out#"$GAME_DIR"/}"
    "$BLENDER_BIN" --background --python "$TOOLS_DIR/convert_single_fbx.py" -- "$fbx" "$out"
done

echo "Done."
