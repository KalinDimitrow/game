# Level Format

RON format for level files (`data/levels/*.ron`), loaded by
`systems::terrain::level_config::load_level_config` and read into the
`LevelConfig` resource. The same format is being re-validated piece by
piece in `utils::level_map` (see `[[01 Cleanup Backlog]]`).

The reference example below is `data/levels/example_level.ron`, not a copy -
it's embedded into `utils::level_map`'s tests via `include_str!`, so it
doubles as this doc's worked example and as the fixture the validator is
built and tested against. If you add a field here, add it to that file
first and let the tests tell you what still needs parsing.

```ron
(
    name: "example_level",

    grid: (
        width: 5,
        depth: 4,
        cell_size: 1.0,
    ),

    terrain: (
        tiles: [
            (id: "ground", path: "models/terrain/ground_tile.glb"),
            (id: "water", path: "models/terrain/water_tile.glb"),
        ],
        default_tile: "ground",
        overrides: [
            (x: 2, z: 1, tile: "water", height: -0.5, walkable: false),
        ],
    ),

    points_of_interest: [
        (kind: PlayerSpawn, position: (0, 0)),
        (kind: EnemySpawn("north_gate"), position: (4, 0)),
        (kind: Objective("core"), position: (0, 3)),
    ],
)
```

## Fields

- **name** - level identifier, currently just for logging.
- **grid.width** / **grid.depth** - cell counts along X/Z. Must both be
  non-zero.
- **grid.cell_size** - world-space size of one cell. Must be positive.
- **terrain.tiles** - the tile registry: `id` is a local name used by
  `default_tile` and `overrides` below, `path` is a glTF asset path
  (resolved against `ASSET_ROOT_PATH`).
- **terrain.default_tile** - a tile `id` that must exist in `terrain.tiles`.
  Used for every cell not named in `overrides`.
- **terrain.overrides** - per-cell exceptions: `(x, z)` must be inside the
  grid, `tile` must be a registered `id`. `height` offsets the tile
  vertically; `walkable` is reserved for pathing/collision, not read yet.
- **points_of_interest** - markers placed on the grid: `PlayerSpawn`,
  `EnemySpawn(id)`, or `Objective(id)`, each at a grid `position` that must
  be inside the grid. `id` lets later systems (wave composition, win/lose
  conditions) reference a specific gate/base by name.

## Validation rules

Checked by `level_config::resolve` today, and being rebuilt as small,
independently-tested functions in `utils::level_map` (one function per rule,
composed into `validate_level_map`):

1. `grid.width` and `grid.depth` are non-zero.
2. `grid.cell_size` is positive.
3. `terrain.default_tile` names a tile that exists in `terrain.tiles`.
4. Every `terrain.overrides` entry's `(x, z)` is inside the grid, and its
   `tile` names a registered tile.
5. Every `points_of_interest` entry's `position` is inside the grid.
