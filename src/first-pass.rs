//! Shows how to render simple primitive shapes with a single color.
//!
//! You can toggle wireframes with the space bar except on wasm. Wasm does not support
//! `POLYGON_MODE_LINE` on the gpu.

use bevy::prelude::*;
use rand::Rng;
use std::io::{self, Write};

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
    )
    .add_systems(Startup, cell_stage_setup)
    .add_systems(Update, cell_stage_update);
    #[cfg(not(target_arch = "wasm32"))]
    app.run();
}


#[derive(Component)]
struct Cell {
    alive: bool,
    x: usize,
    y: usize,
    neighbors: Vec<Entity>,
}

const SQUARE_SIZE: f32 = 10.0;
const X_CELLS_WIDE: usize = 200;
const Y_CELLS_WIDE: usize = 200;
const ALIVE_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
const DEAD_COLOR: Color = Color::srgb(0.0, 1.0, 0.0);

fn cell_stage_setup(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Cell)>) {
    // draws a grid of cells
    // each cell is a square
    // each cell is a boolean (on or off    
    

    commands.spawn(Camera2d);

    for i in 0..X_CELLS_WIDE{
        for j in 0..Y_CELLS_WIDE  {
            
            // draw a cell
            let x = i as f32 * SQUARE_SIZE;
            let y = j as f32 * SQUARE_SIZE;
            let alive = rand::rng().gen_bool(0.5);
            let color = if alive { ALIVE_COLOR } else { DEAD_COLOR };
            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                    ..default()
                },
                Transform::from_xyz(x, y, 0.0),
                Visibility::default(),
                Cell {
                    alive,
                    x: i,
                    y: j,
                    neighbors: Vec::new(),
                },
            ));
        }
    }

    for (_, mut cell) in query.iter_mut() {
        let mut neighbors = Vec::new();
        // Iterate over the relative positions of all potential neighbors
        for offset_x in -1..=1 {
            for offset_y in -1..=1 {
                // Skip the cell itself
                if offset_x == 0 && offset_y == 0 {
                    continue;
                }

                // Calculate the neighbor's coordinates
                let neighbor_x = cell.x as isize + offset_x;
                let neighbor_y = cell.y as isize + offset_y;

                // Check if the neighbor is within the grid bounds
                if neighbor_x >= 0
                    && neighbor_x < X_CELLS_WIDE as isize
                    && neighbor_y >= 0
                    && neighbor_y < Y_CELLS_WIDE as isize
                {
                    // Add the neighbor's entity to the list
                    neighbors.push(Entity::from_raw((neighbor_x as usize * Y_CELLS_WIDE + neighbor_y as usize) as u32));
                }
            }
        }
        cell.neighbors = neighbors;
    }
    

}
fn cell_stage_update(
    mut commands: Commands,
    mut query_cells: Query<(&mut Cell, &mut Sprite)>,
) {
    // Collect updates to apply after the iteration
    let mut updates = Vec::new();

    // First pass: gather the new states for all cells
    for (cell, _) in query_cells.iter() {
        let mut alive_neighbors = 0;

        for neighbor in &cell.neighbors {
            if let Ok((neighbor_cell, _)) = query_cells.get(*neighbor) {
                if neighbor_cell.alive {
                    alive_neighbors += 1;
                }
            }
        }

        let new_state = if !cell.alive && alive_neighbors == 3 {
            true
        } else if cell.alive && (alive_neighbors < 2 || alive_neighbors > 3) {
            false
        } else {
            cell.alive
        };

        if new_state == false
        {
            io::stdout().write_all(format!("Cell ({}, {}) - Alive: {}, Living Neighbors: {}, New State: {}\n", cell.x, cell.y, cell.alive, alive_neighbors ,new_state).as_bytes()).unwrap();
        }
        
        updates.push((cell.x, cell.y, new_state));
    }

    // Second pass: apply the updates
    for (x, y, new_state) in updates {
        if let Ok((mut cell, mut sprite)) = query_cells.get_mut(Entity::from_raw((x * Y_CELLS_WIDE + y) as u32)) {
            cell.alive = new_state;
            sprite.color = if new_state { ALIVE_COLOR } else { DEAD_COLOR };
        }
    }
}