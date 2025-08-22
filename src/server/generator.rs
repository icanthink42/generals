use rand::Rng;
use generals::shared::terrain::Terrain;
use crate::map::{Map, Cell};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainConfig {
    pub mountain_density: f32,  // 0.0 to 1.0, percentage of map to be mountains
    pub swamp_density: f32,     // 0.0 to 1.0
    pub desert_density: f32,    // 0.0 to 1.0
    pub city_density: f32,      // 0.0 to 1.0
    pub clustering_factor: f32,  // 0.0 to 1.0, how much terrain should cluster together
    pub map_width: usize,       // Width of the game map
    pub map_height: usize,      // Height of the game map
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            mountain_density: 0.12,   // 12% mountains
            swamp_density: 0.08,      // 8% swamps
            desert_density: 0.15,     // 15% deserts
            city_density: 0.04,       // 4% cities
            clustering_factor: 0.7,    // High clustering
            map_width: 20,           // Default map width
            map_height: 20,          // Default map height
        }
    }
}

pub fn generate_map_tiles(width: usize, height: usize, config: &TerrainConfig) -> Vec<Cell> {
    let mut rng = rand::thread_rng();
    let total_cells = width * height;
    let mut cells = vec![Cell::default(); total_cells];

    // Helper function to get neighboring cells
    let get_neighbors = |idx: usize| -> Vec<usize> {
        let x = idx % width;
        let y = idx / width;
        let mut neighbors = Vec::new();

        // Check all 8 surrounding cells
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 { continue; }

                let new_x = x as i32 + dx;
                let new_y = y as i32 + dy;

                if new_x >= 0 && new_x < width as i32 && new_y >= 0 && new_y < height as i32 {
                    neighbors.push((new_y as usize * width) + new_x as usize);
                }
            }
        }
        neighbors
    };

    // Function to place terrain with clustering
    let mut place_terrain = |terrain: Terrain| {
        let density = match terrain {
            Terrain::Mountain => config.mountain_density,
            Terrain::Desert => config.desert_density,
            Terrain::Swamp => config.swamp_density,
            Terrain::City => config.city_density,
            _ => return,
        };

        let target_count = (total_cells as f32 * density) as usize;
        let mut placed = 0;

        while placed < target_count {
            let mut pos = rng.gen_range(0..total_cells);

            // If we're clustering and this isn't the first placement
            if placed > 0 && rng.gen_range(0.0..1.0) < config.clustering_factor {
                // Find all cells that already have this terrain
                let existing: Vec<_> = cells.iter()
                    .enumerate()
                    .filter(|(_, cell)| cell.terrain == terrain)
                    .map(|(idx, _)| idx)
                    .collect();

                if !existing.is_empty() {
                    // Pick a random existing terrain cell
                    let base = existing[rng.gen_range(0..existing.len())];
                    // Get its neighbors
                    let neighbors = get_neighbors(base);
                    if !neighbors.is_empty() {
                        // Pick a random neighbor
                        pos = neighbors[rng.gen_range(0..neighbors.len())];
                    }
                }
            }

            // Only place if the cell is empty (default terrain)
            if cells[pos].terrain == Terrain::Default {
                cells[pos].terrain = terrain;
                // Add troops to cities
                if terrain == Terrain::City {
                    cells[pos].troops = rng.gen_range(30..=50);
                }
                placed += 1;
            }
        }
    };

    // Place terrain in order of priority
    place_terrain(Terrain::Mountain);
    place_terrain(Terrain::Desert);
    place_terrain(Terrain::Swamp);
    place_terrain(Terrain::City);

    cells
}

pub fn generate_map(width: usize, height: usize, config: TerrainConfig) -> Map {
    let map = Map::new(width, height);
    let tiles = generate_map_tiles(width, height, &config);
    {
        let mut cells = map.cells.write();
        cells.clone_from_slice(&tiles);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::terrain::Terrain;

    #[test]
    fn test_terrain_generation() {
        let config = TerrainConfig::default();
        let map = generate_map(20, 20, config);
        let cells = map.cells.read();

        // Count each terrain type
        let mut counts = std::collections::HashMap::new();
        for cell in cells.iter() {
            *counts.entry(cell.terrain).or_insert(0) += 1;
        }

        let total_cells = (20 * 20) as f32;

        // Check if terrain densities are roughly correct (within 2% margin)
        let mountain_count = *counts.get(&Terrain::Mountain).unwrap_or(&0) as f32 / total_cells;
        assert!((mountain_count - config.mountain_density).abs() < 0.02);

        let desert_count = *counts.get(&Terrain::Desert).unwrap_or(&0) as f32 / total_cells;
        assert!((desert_count - config.desert_density).abs() < 0.02);

        let swamp_count = *counts.get(&Terrain::Swamp).unwrap_or(&0) as f32 / total_cells;
        assert!((swamp_count - config.swamp_density).abs() < 0.02);

        let city_count = *counts.get(&Terrain::City).unwrap_or(&0) as f32 / total_cells;
        assert!((city_count - config.city_density).abs() < 0.02);
    }
}