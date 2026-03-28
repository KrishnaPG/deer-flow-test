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
}
