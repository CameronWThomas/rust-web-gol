use bevy::prelude::*;

const CELL_SIZE: f32 = 1.0;
const DESIRED_WIDTH: usize = 256;
const DESIRED_HEIGHT: usize =256;
const MANUAL_WINDOW_SIZE: bool = true;
#[derive(Component)]
struct Cell {
    alive: bool,
}

#[derive(Resource)]
struct Grid {
    cells: Vec<Vec<bool>>,
    width: i16,
    height: i16,
}

fn main() {
    App::new()
        .insert_resource(Grid {
            cells: vec![],
            width: 0,
            height: 0,
        })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (update_cells, render_cells))
        .run();
}

fn setup(mut commands: Commands, mut grid: ResMut<Grid>, mut window: Single<&mut Window>) {
    if MANUAL_WINDOW_SIZE {
        window.resolution.set(
            DESIRED_WIDTH as f32,
            DESIRED_HEIGHT as f32,
        );
    }

    let grid_width = (window.width() / CELL_SIZE) as usize;
    let grid_height = (window.height() / CELL_SIZE) as usize;

    grid.width = grid_width as i16;
    grid.height = grid_height as i16;
    grid.cells = vec![vec![false; grid_width]; grid_height];

    commands.spawn(Camera2d::default());

    // Offset so the grid is centered on screen (sprites are center-anchored)
    let offset_x = -(grid_width as f32 * CELL_SIZE) / 2.0 + CELL_SIZE / 2.0;
    let offset_y = -(grid_height as f32 * CELL_SIZE) / 2.0 + CELL_SIZE / 2.0;

    for y in 0..grid_height {
        for x in 0..grid_width {
            let alive = rand::random::<bool>();
            grid.cells[y][x] = alive;

            commands.spawn((
                Cell { alive },
                Sprite {
                    color: if alive { Color::WHITE } else { Color::BLACK },
                    custom_size: Some(Vec2::splat(CELL_SIZE)),
                    ..Default::default()
                },
                Transform::from_translation(Vec3::new(
                    offset_x + x as f32 * CELL_SIZE,
                    offset_y + y as f32 * CELL_SIZE,
                    0.0,
                )),
            ));
        }
    }
}
fn update_cells(mut grid: ResMut<Grid>, mut query: Query<(&mut Cell, &mut Sprite, &Transform)>) {
    let mut new_cells = grid.cells.clone();

    let width = grid.width as usize;
    let height = grid.height as usize;

    for y in 0..height {
        for x in 0..width {
            let alive_neighbors = count_alive_neighbors(&grid.cells, x, y, width, height);
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

    let offset_x = -(grid.width as f32 * CELL_SIZE) / 2.0 + CELL_SIZE / 2.0;
    let offset_y = -(grid.height as f32 * CELL_SIZE) / 2.0 + CELL_SIZE / 2.0;

    for (mut cell, mut sprite, transform) in query.iter_mut() {
        let x = ((transform.translation.x - offset_x) / CELL_SIZE) as usize;
        let y = ((transform.translation.y - offset_y) / CELL_SIZE) as usize;

        cell.alive = grid.cells[y][x];
        sprite.color = if cell.alive { Color::WHITE } else { Color::BLACK };
    }
}

fn count_alive_neighbors(cells: &Vec<Vec<bool>>, x: usize, y: usize, width: usize, height: usize) -> usize {
    let mut count = 0;

    for dy in -1..=1_isize {
        for dx in -1..=1_isize {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = x as isize + dx;
            let ny = y as isize + dy;

            if nx >= 0
                && ny >= 0
                && nx < width as isize
                && ny < height as isize
                && cells[ny as usize][nx as usize]
            {
                count += 1;
            }
        }
    }

    count
}

fn render_cells(grid: Res<Grid>, mut query: Query<(&mut Cell, &mut Sprite, &Transform)>) {
    let offset_x = -(grid.width as f32 * CELL_SIZE) / 2.0 + CELL_SIZE / 2.0;
    let offset_y = -(grid.height as f32 * CELL_SIZE) / 2.0 + CELL_SIZE / 2.0;

    for (mut cell, mut sprite, transform) in query.iter_mut() {
        let x = ((transform.translation.x - offset_x) / CELL_SIZE) as usize;
        let y = ((transform.translation.y - offset_y) / CELL_SIZE) as usize;

        cell.alive = grid.cells[y][x];
        sprite.color = if cell.alive { Color::WHITE } else { Color::BLACK };
    }
}