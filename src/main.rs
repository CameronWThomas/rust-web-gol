use bevy::prelude::*;

const GRID_SIZE: usize = 64;
const CELL_SIZE: f32 = 10.0;

#[derive(Component)]
struct Cell {
    alive: bool,
}

#[derive(Resource)]
struct Grid {
    cells: Vec<Vec<bool>>,
}

fn main() {
    App::new()
        .insert_resource(Grid {
            cells: vec![vec![false; GRID_SIZE]; GRID_SIZE],
        })
        .add_plugins(DefaultPlugins)
        .add_startup_systems(setup)
        .add_system(update_cells)
        .add_system(render_cells)
        .run();
}

fn setup(mut commands: Commands, mut grid: ResMut<Grid>) {
    commands.spawn(Camera2dBundle::default());

    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            let alive = rand::random::<bool>();
            grid.cells[y][x] = alive;

            commands.spawn((
                Cell { alive },
                SpriteBundle {
                    sprite: Sprite {
                        color: if alive { Color::WHITE } else { Color::BLACK },
                        custom_size: Some(Vec2::splat(CELL_SIZE)),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        x as f32 * CELL_SIZE,
                        y as f32 * CELL_SIZE,
                        0.0,
                    )),
                    ..Default::default()
                },
            ));
        }
    }
}

fn update_cells(mut grid: ResMut<Grid>, mut query: Query<(&mut Cell, &mut Sprite)>) {
    let mut new_cells = grid.cells.clone();

    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            let alive_neighbors = count_alive_neighbors(&grid.cells, x, y);
            let is_alive = grid.cells[y][x];

            new_cells[y][x] = match (is_alive, alive_neighbors) {
                (true, 2) | (true, 3) => true,
                (true, _) => false,
                (false, 3) => true,
                _ => false,
            };
        }
    }

    grid.cells = new_cells;

    for (mut cell, mut sprite) in query.iter_mut() {
        let x = (sprite.transform.translation.x / CELL_SIZE) as usize;
        let y = (sprite.transform.translation.y / CELL_SIZE) as usize;

        cell.alive = grid.cells[y][x];
        sprite.sprite.color = if cell.alive { Color::WHITE } else { Color::BLACK };
    }
}

fn count_alive_neighbors(cells: &Vec<Vec<bool>>, x: usize, y: usize) -> usize {
    let mut count = 0;

    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = x as isize + dx;
            let ny = y as isize + dy;

            if nx >= 0
                && ny >= 0
                && nx < GRID_SIZE as isize
                && ny < GRID_SIZE as isize
                && cells[ny as usize][nx as usize]
            {
                count += 1;
            }
        }
    }

    count
}

fn render_cells(grid: Res<Grid>, mut query: Query<(&mut Cell, &mut Sprite)>) {
    for (mut cell, mut sprite) in query.iter_mut() {
        let x = (sprite.transform.translation.x / CELL_SIZE) as usize;
        let y = (sprite.transform.translation.y / CELL_SIZE) as usize;

        cell.alive = grid.cells[y][x];
        sprite.sprite.color = if cell.alive { Color::WHITE } else { Color::BLACK };
    }
}