"""
Convert exactly one .fbx to .glb - no merging. Each output keeps its own
single animation (if any), renamed to match the source file so downstream
code can find it predictably. Bevy matches animation clips to scene
entities purely by bone name (AnimationTargetId, a hash of the bone name
path), not by which file a clip came from - so the game can load the base
character mesh from one .glb and each animation from its own separate
.glb, and apply them all to the same skeleton at runtime. That sidesteps
every bug hit trying to pre-merge multiple clips into one file (Blender's
NLA/multi-track export, assimp's inverse-bind-matrix corruption,
COLLADA2GLTF's animation export, all of it) - single-file conversion is
the one thing every tool tried this far has done correctly.

Mixamo FBX files typically import as TWO actions: the real animation
(named like "Armature|mixamo.com|Layer0") and a generic placeholder
("Armature|Take 001|BaseLayer"). Blender's FBX importer doesn't reliably
leave the real one as the active action, so this explicitly picks
whichever action has "mixamo" in its name when present. It also has to
deal with Blender 4.4+'s "Action Slots" system (Action -> Slots -> Layers
-> Strips -> Channelbags -> F-Curves) - assigning `.action` alone doesn't
guarantee the correct slot gets bound, which can leave the armature
effectively unanimated even though `.action` looks right.

Run headless, no display needed, with args after `--`:
    blender --background --python convert_single_fbx.py -- input.fbx output.glb
"""

import bpy
import os
import sys

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
input_path, output_path = argv[0], argv[1]

bpy.ops.wm.read_homefile(use_empty=True, use_factory_startup=True)
bpy.ops.import_scene.fbx(filepath=input_path)

clip_name = os.path.splitext(os.path.basename(input_path))[0].strip().replace(" ", "_")

for armature in (o for o in bpy.data.objects if o.type == "ARMATURE"):
    if armature.animation_data is None:
        continue

    mixamo_actions = [a for a in bpy.data.actions if "mixamo" in a.name.lower()]
    if mixamo_actions:
        action = mixamo_actions[0]
    elif armature.animation_data.action is not None:
        action = armature.animation_data.action
    else:
        continue

    armature.animation_data.action = action
    if armature.animation_data.action_suitable_slots:
        armature.animation_data.action_slot = armature.animation_data.action_suitable_slots[0]

    action.name = clip_name
    print(f"Using action {action.name!r} for armature {armature.name!r}")

bpy.ops.object.select_all(action="SELECT")
export_kwargs = {
    "filepath": output_path,
    "export_format": "GLB",
    "use_selection": True,
}
op_props = bpy.ops.export_scene.gltf.get_rna_type().properties.keys()
if "export_animation_mode" in op_props:
    export_kwargs["export_animation_mode"] = "ACTIVE_ACTIONS"

os.makedirs(os.path.dirname(output_path), exist_ok=True)
print(f"Exporting {input_path!r} -> {output_path!r}")
bpy.ops.export_scene.gltf(**export_kwargs)
print("Done.")
