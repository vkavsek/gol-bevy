#![allow(clippy::type_complexity)]

use std::time::Duration;

use bevy::{
    ecs::system::SystemState,
    input::common_conditions::input_just_pressed,
    math::{ivec2, uvec2, vec2},
    prelude::*,
    utils::HashMap,
};

use crate::{prelude::*, state::GameState};

pub struct LifePlugin;

impl Plugin for LifePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Board::default())
            .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(
                UPDATE_INTERVAL_MS,
            )))
            .add_systems(
                OnEnter(GameState::Load),
                (load_meshes_and_materials, load_cell_board).chain(),
            )
            .add_systems(
                FixedUpdate,
                ((update_cell_future_life, update_cell_current_life).chain())
                    .run_if(in_state(GameState::Running)),
            )
            .add_systems(
                Update,
                (
                    handle_setup_kbd.run_if(in_state(GameState::Setup)),
                    handle_cell_color_main.run_if(in_state(GameState::Running)),
                    toggle_setup_and_running.run_if(
                        input_just_pressed(KeyCode::Enter)
                            .and(in_state(GameState::Running).or(in_state(GameState::Setup))),
                    ),
                ),
            );
    }
}

// ——> SYSTEMS

/// initialize meshes and materials in a resource
fn load_meshes_and_materials(
    world: &mut World,
    params: &mut SystemState<(
        ResMut<Assets<Mesh>>,
        ResMut<Assets<ColorMaterial>>,
        Res<Board>,
    )>,
) {
    // create material & mesh handles, and store them in the world
    let (mut meshes, mut materials, board) = params.get_mut(world);
    let cell_mesh = meshes.add(Rectangle::from_size(board.cell_size));
    let border_vert_mesh = meshes.add(Rectangle::new(
        BORDER_WIDTH_PX,
        board.pixel_size().y + 2.0 * BORDER_WIDTH_PX,
    ));
    let border_horiz_mesh = meshes.add(Rectangle::new(
        board.pixel_size().x + 2.0 * BORDER_WIDTH_PX,
        BORDER_WIDTH_PX,
    ));
    let border_mat = materials.add(ColorMaterial::from_color(BORDER_COLOR));
    let cell_alive_mat = materials.add(ColorMaterial::from_color(CELL_ALIVE_COLOR));
    let cell_dead_mat = materials.add(ColorMaterial::from_color(BG_COLOR));
    let cell_clicked_mat = materials.add(ColorMaterial::from_color(CELL_CLICKED_COLOR));
    let cell_hovered_alive_mat = materials.add(ColorMaterial::from_color(CELL_HOVERED_ALIVE_COLOR));
    let cell_hovered_dead_mat = materials.add(ColorMaterial::from_color(CELL_HOVERED_DEAD_COLOR));

    let meshes = HashMap::from([
        ("cell", cell_mesh),
        ("border_vert", border_vert_mesh),
        ("border_horiz", border_horiz_mesh),
    ]);
    let materials = HashMap::from([
        ("border", border_mat),
        ("cell_alive", cell_alive_mat),
        ("cell_dead", cell_dead_mat),
        ("cell_clicked", cell_clicked_mat),
        ("cell_hovered_alive", cell_hovered_alive_mat),
        ("cell_hovered_dead", cell_hovered_dead_mat),
    ]);
    // create an easily accessible resource for efficient reuse of materials and meshes
    world.insert_resource(MeshAndMats { meshes, materials });
}

/// spawn game of life board
fn load_cell_board(
    world: &mut World,
    params: &mut SystemState<(Res<MeshAndMats>, Res<Board>, ResMut<NextState<GameState>>)>,
) {
    let (meshes_and_mats, board, _) = params.get_mut(world);
    // copy the board so that we can use it later
    let board = *board;

    let (alive_mat, dead_mat, clicked_mat, hovered_alive_mat, hovered_dead_mat) = (
        meshes_and_mats
            .materials
            .get("cell_alive")
            .unwrap()
            .to_owned(),
        meshes_and_mats
            .materials
            .get("cell_dead")
            .unwrap()
            .to_owned(),
        meshes_and_mats
            .materials
            .get("cell_clicked")
            .unwrap()
            .to_owned(),
        meshes_and_mats
            .materials
            .get("cell_hovered_alive")
            .unwrap()
            .to_owned(),
        meshes_and_mats
            .materials
            .get("cell_hovered_dead")
            .unwrap()
            .to_owned(),
    );

    let coords_iter = (0..board.size).flat_map(|y| (0..board.size).map(move |x| uvec2(x, y)));
    let cells_to_spawn = coords_iter
        .clone()
        .map(|cell_coord| {
            (
                Cell,
                Mesh2d(meshes_and_mats.meshes.get("cell").unwrap().to_owned()),
                MeshMaterial2d(dead_mat.clone()),
                // CurrentAlive(fastrand::bool()),
                Transform::from_translation(board.cell_coord_to_translation(cell_coord))
                    .with_scale(board.cell_scale.xyx()),
            )
        })
        .collect::<Vec<_>>();
    // spawn cells
    let entities: Vec<_> = world.spawn_batch(cells_to_spawn).collect();

    // add observers to support cell picking in the setup stage.
    //
    // hovering observer
    world.add_observer(cells_set_mats_on::<Pointer<Over>>(
        hovered_alive_mat.clone(),
        hovered_dead_mat.clone(),
    ));
    // end of hover observer
    world.add_observer(cells_set_mats_on::<Pointer<Out>>(alive_mat, dead_mat));
    // clicked observer
    world.add_observer(cells_set_life_on::<Pointer<Down>>(clicked_mat.clone()));
    // drag-over observer
    world.add_observer(cells_set_life_on::<Pointer<DragOver>>(clicked_mat));
    // end of click observer
    world.add_observer(cells_set_mats_on::<Pointer<Up>>(
        hovered_alive_mat,
        hovered_dead_mat,
    ));

    let neighbours = (0..entities.len())
        .map(|i| {
            let neighbour_entity_indices = board.neighbour_indices(board.idx_to_cell_coord(i));

            // temporarily initialize with the default value
            let mut neigh_entities = [entities[0]; 8];
            for (i, neigh_idx) in neighbour_entity_indices.into_iter().enumerate() {
                neigh_entities[i] = entities[neigh_idx];
            }
            neigh_entities
        })
        .map(Neighbours)
        .collect::<Vec<_>>();

    let pairs = entities.into_iter().zip(neighbours);
    // add neighbours to the cells
    world.insert_batch(pairs);

    // create borders
    let (meshes_and_mats, _, _) = params.get_mut(world);
    // meshes
    let border_vert = meshes_and_mats
        .meshes
        .get("border_vert")
        .unwrap()
        .to_owned();
    let border_horiz = meshes_and_mats
        .meshes
        .get("border_horiz")
        .unwrap()
        .to_owned();
    // create vertical and horizontal meshes and transforms
    let border_mesh_and_transforms = (0..4).map(|i| {
        // vertical
        if i % 2 == 0 {
            (
                Mesh2d(border_vert.clone()),
                // left
                if i / 2 == 0 {
                    let pos = board.center
                        - (board.pixel_size().with_y(0.0) * 0.5
                            + Vec2::new(BORDER_WIDTH_PX, 0.0) * 0.5);
                    Transform::from_translation(pos.extend(0.0))
                // or right
                } else {
                    let pos = board.center
                        + (board.pixel_size().with_y(0.0) * 0.5
                            + Vec2::new(BORDER_WIDTH_PX, 0.0) * 0.5);
                    Transform::from_translation(pos.extend(0.0))
                },
            )
        // or horizontal
        } else {
            (
                Mesh2d(border_horiz.clone()),
                // up
                if i / 2 == 0 {
                    let pos = board.center
                        + (board.pixel_size().with_x(0.0) * 0.5
                            + Vec2::new(0.0, BORDER_WIDTH_PX) * 0.5);
                    Transform::from_translation(pos.extend(0.0))
                // or down
                } else {
                    let pos = board.center
                        - (board.pixel_size().with_x(0.0) * 0.5
                            + Vec2::new(0.0, BORDER_WIDTH_PX) * 0.5);
                    Transform::from_translation(pos.extend(0.0))
                },
            )
        }
    });
    let border_mat = meshes_and_mats.materials.get("border").unwrap().to_owned();
    // connect all the components in a bundle
    let borders = border_mesh_and_transforms
        .map(|(mesh, transform)| (Border, MeshMaterial2d(border_mat.clone()), mesh, transform));
    world.spawn_batch(borders);

    let (_, _, mut game_state) = params.get_mut(world);
    game_state.set(GameState::Setup);
}

/// Returns an observer that changes the life status of a cell when clicked on, while also
/// highlighting that cell by changing its material.
fn cells_set_life_on<E>(
    highlight_mat: Handle<ColorMaterial>,
) -> impl Fn(
    Trigger<E>,
    Query<(&mut MeshMaterial2d<ColorMaterial>, &mut CurrentAlive), With<Cell>>,
    Res<State<GameState>>,
) {
    move |trigger, mut query, state| {
        if matches!(state.get(), GameState::Setup) {
            if let Ok((mut material, mut alive)) = query.get_mut(trigger.entity()) {
                material.0 = highlight_mat.clone();
                alive.0 = !alive.0;
            }
        }
    }
}

/// Returns an observer that updates the cell's material to one of the specified materials,
/// depending on the cell's life status.
fn cells_set_mats_on<E>(
    new_mat_alive: Handle<ColorMaterial>,
    new_mat_dead: Handle<ColorMaterial>,
) -> impl Fn(
    Trigger<E>,
    Query<(&mut MeshMaterial2d<ColorMaterial>, &CurrentAlive), With<Cell>>,
    Res<State<GameState>>,
) {
    move |trigger, mut query, state| {
        if matches!(state.get(), GameState::Setup) {
            if let Ok((mut material, alive)) = query.get_mut(trigger.entity()) {
                if alive.0 {
                    material.0 = new_mat_alive.clone();
                } else {
                    material.0 = new_mat_dead.clone();
                }
            }
        }
    }
}

fn handle_setup_kbd(
    mut cell_query: Query<(&mut CurrentAlive, &mut MeshMaterial2d<ColorMaterial>), With<Cell>>,
    meshes_and_mats: Res<MeshAndMats>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        for (mut alive, mut material) in cell_query.iter_mut() {
            alive.0 = fastrand::bool();
            if alive.0 {
                material.0 = meshes_and_mats
                    .materials
                    .get("cell_alive")
                    .unwrap()
                    .to_owned();
            } else {
                material.0 = meshes_and_mats
                    .materials
                    .get("cell_dead")
                    .unwrap()
                    .to_owned();
            }
        }
    }
}

fn toggle_setup_and_running(
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    match state.get() {
        GameState::Setup => next_state.set(GameState::Running),
        GameState::Running => next_state.set(GameState::Setup),
        _ => unreachable!(),
    }
}

fn handle_cell_color_main(
    mut cell_query: Query<
        (&mut MeshMaterial2d<ColorMaterial>, &CurrentAlive),
        (
            With<Cell>,
            Or<(
                Changed<CurrentAlive>,
                Changed<MeshMaterial2d<ColorMaterial>>,
            )>,
        ),
    >,
    mesh_n_mats: Res<MeshAndMats>,
) {
    for (mut material, cell_alive) in cell_query.iter_mut() {
        if **cell_alive {
            **material = mesh_n_mats.materials.get("cell_alive").unwrap().to_owned();
        } else {
            **material = mesh_n_mats.materials.get("cell_dead").unwrap().to_owned();
        }
    }
}

fn update_cell_future_life(
    mut cell_query: Query<(&mut FutureAlive, &Neighbours), With<Cell>>,
    immutable_query: Query<&CurrentAlive, With<Cell>>,
) {
    for (mut future, neighbours) in cell_query.iter_mut() {
        let nval = immutable_query
            .many(**neighbours)
            .map(|curr| if **curr { 1u8 } else { 0 })
            .iter()
            .sum::<u8>();

        match nval {
            3 => **future = Some(true),
            2 => (),
            _ => **future = Some(false),
        }
    }
}

fn update_cell_current_life(
    mut cell_query: Query<
        (&mut FutureAlive, &mut CurrentAlive),
        (With<Cell>, Changed<FutureAlive>),
    >,
) {
    for (mut fut, mut curr) in cell_query.iter_mut() {
        if let Some(alive) = **fut {
            **curr = alive;
            **fut = None;
        }
    }
}

// ——> COMPONENTS

#[derive(Component)]
#[require(CurrentAlive, FutureAlive, Mesh2d)]
struct Cell;

#[derive(Component, Debug, Default, DerefMut, Deref)]
struct CurrentAlive(bool);

#[derive(Component, Debug, Default, DerefMut, Deref)]
struct FutureAlive(Option<bool>);

#[derive(Component, Debug, DerefMut, Deref)]
struct Neighbours([Entity; 8]);

#[derive(Component)]
#[require(Mesh2d)]
struct Border;

// ——> RESOURCES

/// hold handles for meshes and materials
#[derive(Resource, Clone)]
struct MeshAndMats {
    meshes: HashMap<&'static str, Handle<Mesh>>,
    materials: HashMap<&'static str, Handle<ColorMaterial>>,
}

#[derive(Resource, Clone, Copy)]
struct Board {
    /// the center of the board
    center: Vec2,
    /// the amount of cells on each axis
    size: u32,
    /// the size of each individual cell
    cell_size: Vec2,
    /// scale of each individual cell (should be 0.0 - 1.0)
    cell_scale: Vec2,
}

impl Board {
    /// computes full size of the board in pixels
    #[inline]
    fn pixel_size(&self) -> Vec2 {
        vec2(
            self.size as f32 * self.cell_size.x,
            self.size as f32 * self.cell_size.y,
        )
    }

    #[inline]
    fn cell_coord_to_translation(&self, cell_coord: UVec2) -> Vec3 {
        (self.center - (self.pixel_size() * 0.5)
            + cell_coord.as_vec2() * self.cell_size
            + self.cell_size * 0.5)
            .extend(10.0)
    }

    #[inline]
    fn cell_coord_to_idx(&self, cell_coord: UVec2) -> usize {
        ((cell_coord.y % self.size) * self.size + (cell_coord.x % self.size)) as usize
    }

    #[inline]
    fn idx_to_cell_coord(&self, idx: usize) -> UVec2 {
        uvec2(idx as u32 % self.size, idx as u32 / self.size)
    }

    #[inline]
    fn neighbour_indices(&self, cell_coord: UVec2) -> [usize; 8] {
        let mut result = [0; 8];
        for (i, neigh_pos) in (-1..=1)
            .flat_map(|y| (-1..=1).map(move |x| ivec2(x, y)))
            // filter out if pos_offs is (0, 0)
            .filter(|pos_offs| !(pos_offs.x == 0 && pos_offs.y == 0))
            .enumerate()
            .map(|(i, pos_offs)| {
                let pos = cell_coord.as_ivec2() + pos_offs;
                let mut neigh_pos = pos.as_uvec2();
                if pos.x < 0 {
                    neigh_pos.x = self.size - 1;
                } else if pos.x >= self.size as i32 {
                    neigh_pos.x = 0;
                }
                if pos.y < 0 {
                    neigh_pos.y = self.size - 1;
                } else if pos.y >= self.size as i32 {
                    neigh_pos.y = 0;
                }

                (i, neigh_pos)
            })
        {
            result[i] = self.cell_coord_to_idx(neigh_pos);
        }

        result
    }
}

impl Default for Board {
    fn default() -> Self {
        Self {
            center: BOARD_POS,
            size: BOARD_SIZE,
            cell_size: CELL_SIZE_PX,
            cell_scale: CELL_SCALE,
        }
    }
}

#[cfg(test)]
mod test {
    use bevy::math::vec3;

    use super::*;

    #[test]
    fn board_works() {
        let board = Board {
            center: Vec2::ZERO,
            cell_size: Vec2::splat(8.0),
            cell_scale: Vec2::splat(0.9),
            size: 8,
        };

        let px_size = board.pixel_size();
        assert_eq!(vec2(64., 64.), px_size);

        let pos1_1 = uvec2(1, 1);
        assert_eq!(9, board.cell_coord_to_idx(pos1_1));
        assert_eq!(0, board.cell_coord_to_idx(uvec2(8, 8)));
        assert_eq!(7, board.cell_coord_to_idx(uvec2(7, 8)));
        assert_eq!(56, board.cell_coord_to_idx(uvec2(8, 7)));
        assert_eq!(uvec2(7, 7), board.idx_to_cell_coord(63));
        assert_eq!(
            vec3(-4.0, -4.0, 10.),
            board.cell_coord_to_translation(uvec2(3, 3))
        );

        let neigh1_1 = board.neighbour_indices(pos1_1);
        let expected_1_1 = [
            board.cell_coord_to_idx(uvec2(0, 0)),
            board.cell_coord_to_idx(uvec2(1, 0)),
            board.cell_coord_to_idx(uvec2(2, 0)),
            board.cell_coord_to_idx(uvec2(0, 1)),
            board.cell_coord_to_idx(uvec2(2, 1)),
            board.cell_coord_to_idx(uvec2(0, 2)),
            board.cell_coord_to_idx(uvec2(1, 2)),
            board.cell_coord_to_idx(uvec2(2, 2)),
        ];
        assert_eq!(expected_1_1, neigh1_1);

        let neigh0_1 = board.neighbour_indices(uvec2(0, 1));
        let expected_0_1 = [
            board.cell_coord_to_idx(uvec2(7, 0)),
            board.cell_coord_to_idx(uvec2(0, 0)),
            board.cell_coord_to_idx(uvec2(1, 0)),
            board.cell_coord_to_idx(uvec2(7, 1)),
            board.cell_coord_to_idx(uvec2(1, 1)),
            board.cell_coord_to_idx(uvec2(7, 2)),
            board.cell_coord_to_idx(uvec2(0, 2)),
            board.cell_coord_to_idx(uvec2(1, 2)),
        ];
        assert_eq!(expected_0_1, neigh0_1);
    }
}
