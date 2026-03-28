//! Spatial indexing for world entities.
//!
//! Provides a simple grid-based spatial index for efficient lookups
//! of entities by 3D position. Used by the picking system to find
//! candidate entities near a raycast hit point.

use bevy::log::{debug, trace};
use bevy::prelude::*;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Grid cell
// ---------------------------------------------------------------------------

/// Integer grid cell coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl GridCell {
    /// Converts a world-space position to a grid cell.
    pub fn from_position(pos: Vec3, cell_size: f32) -> Self {
        Self {
            x: (pos.x / cell_size).floor() as i32,
            y: (pos.y / cell_size).floor() as i32,
            z: (pos.z / cell_size).floor() as i32,
        }
    }
}

// ---------------------------------------------------------------------------
// Spatial index resource
// ---------------------------------------------------------------------------

/// Grid-based spatial index mapping grid cells to entity lists.
///
/// Rebuilt each frame from entities with `Transform` and `Selectable`
/// components. The cell size determines spatial resolution.
#[derive(Resource)]
pub struct SpatialIndex {
    /// The grid cell size (world units).
    pub cell_size: f32,
    /// Maps grid cells to the entities they contain.
    cells: HashMap<GridCell, Vec<Entity>>,
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self {
            cell_size: 20.0,
            cells: HashMap::new(),
        }
    }
}

impl SpatialIndex {
    /// Creates a new spatial index with the given cell size.
    pub fn new(cell_size: f32) -> Self {
        debug!("SpatialIndex::new — cell_size={cell_size}");
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    /// Clears all entries from the index.
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Inserts an entity at the given world position.
    pub fn insert(&mut self, entity: Entity, position: Vec3) {
        let cell = GridCell::from_position(position, self.cell_size);
        trace!(
            "SpatialIndex::insert — entity={:?} pos={:?} cell={:?}",
            entity,
            position,
            cell
        );
        self.cells.entry(cell).or_default().push(entity);
    }

    /// Returns all entities in the cell containing `position`.
    pub fn query_at(&self, position: Vec3) -> &[Entity] {
        let cell = GridCell::from_position(position, self.cell_size);
        trace!(
            "SpatialIndex::query_at — pos={:?} cell={:?}",
            position,
            cell
        );
        self.cells.get(&cell).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Returns all entities in the cell containing `position` and
    /// its 26 neighbouring cells (3x3x3 cube).
    pub fn query_nearby(&self, position: Vec3) -> Vec<Entity> {
        let center = GridCell::from_position(position, self.cell_size);
        trace!(
            "SpatialIndex::query_nearby — pos={:?} center={:?}",
            position,
            center
        );
        let mut result = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let cell = GridCell {
                        x: center.x + dx,
                        y: center.y + dy,
                        z: center.z + dz,
                    };
                    if let Some(entities) = self.cells.get(&cell) {
                        result.extend(entities);
                    }
                }
            }
        }
        result
    }

    /// Returns the number of occupied cells.
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Returns the total number of entity entries.
    pub fn entity_count(&self) -> usize {
        self.cells.values().map(|v| v.len()).sum()
    }

    /// Removes an entity from the cell containing `position`.
    ///
    /// Returns `true` if the entity was found and removed.
    pub fn remove(&mut self, entity: Entity, position: Vec3) -> bool {
        let cell = GridCell::from_position(position, self.cell_size);
        trace!(
            "SpatialIndex::remove — entity={:?} pos={:?} cell={:?}",
            entity,
            position,
            cell
        );
        if let Some(entities) = self.cells.get_mut(&cell) {
            if let Some(idx) = entities.iter().position(|e| *e == entity) {
                entities.swap_remove(idx);
                if entities.is_empty() {
                    self.cells.remove(&cell);
                }
                return true;
            }
        }
        false
    }

    /// Moves an entity from `old_position` to `new_position`.
    ///
    /// Equivalent to `remove(entity, old) + insert(entity, new)`.
    pub fn update(&mut self, entity: Entity, old_position: Vec3, new_position: Vec3) {
        trace!(
            "SpatialIndex::update — entity={:?} old={:?} new={:?}",
            entity,
            old_position,
            new_position
        );
        self.remove(entity, old_position);
        self.insert(entity, new_position);
    }

    /// Returns all entities within `radius` of `center`.
    ///
    /// Checks all grid cells that could overlap the sphere, then
    /// distance-filters against actual positions (not available here,
    /// so returns all entities in overlapping cells — caller should
    /// refine with actual Transform checks).
    pub fn query_sphere(&self, center: Vec3, radius: f32) -> Vec<Entity> {
        let cells_radius = (radius / self.cell_size).ceil() as i32;
        let center_cell = GridCell::from_position(center, self.cell_size);
        trace!(
            "SpatialIndex::query_sphere — center={:?} radius={} cells_radius={}",
            center,
            radius,
            cells_radius
        );
        let mut result = Vec::new();
        for dx in -cells_radius..=cells_radius {
            for dy in -cells_radius..=cells_radius {
                for dz in -cells_radius..=cells_radius {
                    let cell = GridCell {
                        x: center_cell.x + dx,
                        y: center_cell.y + dy,
                        z: center_cell.z + dz,
                    };
                    if let Some(entities) = self.cells.get(&cell) {
                        result.extend(entities);
                    }
                }
            }
        }
        result
    }

    /// Simple grid-walk raycast: returns entities in cells along `ray_direction`
    /// from `ray_origin`, up to `max_distance`.
    ///
    /// Samples the ray at `cell_size / 2` intervals and collects unique
    /// entities from each sampled cell.
    pub fn raycast(&self, ray_origin: Vec3, ray_direction: Vec3, max_distance: f32) -> Vec<Entity> {
        let dir = ray_direction.normalize_or_zero();
        if dir == Vec3::ZERO {
            return Vec::new();
        }
        let step = self.cell_size * 0.5;
        let steps = (max_distance / step).ceil() as usize;
        trace!(
            "SpatialIndex::raycast — origin={:?} dir={:?} max_dist={} steps={}",
            ray_origin,
            dir,
            max_distance,
            steps
        );
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        for i in 0..=steps {
            let pos = ray_origin + dir * (i as f32 * step);
            let cell = GridCell::from_position(pos, self.cell_size);
            if visited.insert(cell) {
                if let Some(entities) = self.cells.get(&cell) {
                    result.extend(entities);
                }
            }
        }
        result
    }

    /// Returns the total number of entity entries (alias).
    pub fn len(&self) -> usize {
        self.entity_count()
    }

    /// Returns `true` if the index contains no entities.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}
