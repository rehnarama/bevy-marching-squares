use core::panic;

use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    math::{vec2, vec3, IRect},
    prelude::*,
    reflect::erased_serde::__private::serde::__private::ser::FlatMapSerializeMap,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
    window::PrimaryWindow,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (march, update_field, zoom_system, move_system))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let mut field = Field::new();
    field.set(IVec2::new(0, 0), 1.);
    field.set(IVec2::new(1, 0), 1.);
    field.set(IVec2::new(1, 1), 1.);
    field.set(IVec2::new(0, 1), 1.);

    commands
        .spawn(MaterialMesh2dBundle {
            mesh: meshes.add(field.march(&0.5)).into(),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            transform: Transform::from_translation(Vec3::new(-150., 0., 0.))
                .with_scale(vec3(5., 5., 5.)),
            ..default()
        })
        .insert(field);
}

fn update_field(
    mut field: Query<(&mut Field, &Transform)>,
    mut motion_evr: EventReader<CursorMoved>,
    buttons: Res<Input<MouseButton>>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = camera.single();

    if buttons.pressed(MouseButton::Left) {
        for ev in motion_evr.iter() {
            if let Some(pos) = camera
                .viewport_to_world(camera_transform, ev.position)
                .map(|ray| ray.origin.truncate())
            {
                for (mut field, transform) in field.iter_mut() {
                    let local_pos: Vec3 =
                        Transform::from_matrix(transform.compute_matrix().inverse())
                            .transform_point(Vec3::new(pos.x, pos.y, 0.));

                    let i_pos = IVec2::new(local_pos.x.round() as i32, local_pos.y.round() as i32);

                    field.set(i_pos, 1.);
                }
            }
        }
    }
}

fn zoom_system(
    mut scroll_evr: EventReader<MouseWheel>,
    keys: Res<Input<KeyCode>>,
    mut camera_q: Query<(&mut OrthographicProjection, &mut Transform), With<Camera2d>>,
    window_q: Query<&Window>
) {
    let window = window_q.single();

    let (mut projection, mut transform) = camera_q.single_mut();

    if keys.pressed(KeyCode::ControlLeft) {
        for e in scroll_evr.iter() {
            projection.scale = projection.scale - projection.scale * e.y * 0.1;
        }
    }
}

fn move_system(
    mut scroll_evr: EventReader<MouseWheel>,
    keys: Res<Input<KeyCode>>,
    mut camera_q: Query<&mut Transform, With<Camera2d>>,
) {
    let mut transform = camera_q.single_mut();
    if !keys.pressed(KeyCode::ControlLeft) {
        for e in scroll_evr.iter() {
            transform.translation = transform.translation + vec3(0., e.y * 50., 0.);
        }
    }
}

fn march(
    mut commands: Commands,
    mut query: Query<(&Field, Entity)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (field, entity) in query.iter() {
        let new_handle: Mesh2dHandle = meshes.add(field.march(&0.5)).into();

        commands.entity(entity).insert(new_handle);
    }
}

#[derive(Component)]
struct Field {
    data: HashMap<IVec2, f32>,
    dimensions: IRect,
}

const TOP: Vec2 = Vec2 { x: 0., y: -0.5 };
const RIGHT: Vec2 = Vec2 { x: 0.5, y: 0. };
const BOTTOM: Vec2 = Vec2 { x: 0., y: 0.5 };
const LEFT: Vec2 = Vec2 { x: -0.5, y: 0. };
const TOP_LEFT: Vec2 = Vec2 { x: -0.5, y: -0.5 };
const TOP_RIGHT: Vec2 = Vec2 { x: 0.5, y: -0.5 };
const BOTTOM_LEFT: Vec2 = Vec2 { x: -0.5, y: 0.5 };
const BOTTOM_RIGHT: Vec2 = Vec2 { x: 0.5, y: 0.5 };

impl Field {
    fn new() -> Field {
        Field {
            data: HashMap::new(),
            dimensions: IRect::new(0, 0, 0, 0),
        }
    }

    fn get_pattern(pattern: u32) -> Vec<[Vec2; 3]> {
        if pattern == Field::calculate_pattern(true, true, true, true) {
            // 0
            vec![
                [BOTTOM_LEFT, TOP_RIGHT, TOP_LEFT],
                [BOTTOM_LEFT, BOTTOM_RIGHT, TOP_RIGHT],
            ]
        } else if pattern == Field::calculate_pattern(true, true, false, true) {
            // 1
            vec![
                [TOP_LEFT, LEFT, TOP_RIGHT],
                [LEFT, BOTTOM, TOP_RIGHT],
                [BOTTOM, BOTTOM_RIGHT, TOP_RIGHT],
            ]
        } else if pattern == Field::calculate_pattern(true, true, true, false) {
            // 2
            vec![
                [TOP_LEFT, BOTTOM_LEFT, TOP_RIGHT],
                [BOTTOM_LEFT, BOTTOM, TOP_RIGHT],
                [BOTTOM, RIGHT, TOP_RIGHT],
            ]
        } else if pattern == Field::calculate_pattern(true, true, false, false) {
            // 3
            vec![[TOP_LEFT, LEFT, TOP_RIGHT], [LEFT, TOP_RIGHT, RIGHT]]
        } else if pattern == Field::calculate_pattern(true, false, true, true) {
            // 4
            vec![
                [TOP_LEFT, BOTTOM_LEFT, TOP],
                [BOTTOM_LEFT, BOTTOM_RIGHT, TOP],
                [BOTTOM_RIGHT, RIGHT, TOP],
            ]
        } else if pattern == Field::calculate_pattern(true, false, false, true) {
            // 5
            vec![[LEFT, TOP, TOP_LEFT], [BOTTOM, BOTTOM_RIGHT, RIGHT]]
        } else if pattern == Field::calculate_pattern(true, false, true, false) {
            // 6
            vec![[TOP_LEFT, BOTTOM_LEFT, TOP], [BOTTOM_LEFT, BOTTOM, TOP]]
        } else if pattern == Field::calculate_pattern(true, false, false, false) {
            // 7
            vec![[TOP_LEFT, LEFT, TOP]]
        } else if pattern == Field::calculate_pattern(false, true, true, true) {
            // 8
            vec![
                [LEFT, BOTTOM_LEFT, TOP],
                [BOTTOM_LEFT, TOP_RIGHT, TOP],
                [BOTTOM_LEFT, BOTTOM_RIGHT, TOP_RIGHT],
            ]
        } else if pattern == Field::calculate_pattern(false, true, false, true) {
            // 9
            vec![[BOTTOM, TOP_RIGHT, TOP], [BOTTOM, BOTTOM_RIGHT, TOP_RIGHT]]
        } else if pattern == Field::calculate_pattern(false, true, true, false) {
            // 10
            vec![[LEFT, BOTTOM_LEFT, BOTTOM], [TOP, TOP_RIGHT, RIGHT]]
        } else if pattern == Field::calculate_pattern(false, true, false, false) {
            // 11
            vec![[TOP, TOP_RIGHT, RIGHT]]
        } else if pattern == Field::calculate_pattern(false, false, true, true) {
            // 12
            vec![
                [BOTTOM_LEFT, RIGHT, LEFT],
                [BOTTOM_LEFT, BOTTOM_RIGHT, RIGHT],
            ]
        } else if pattern == Field::calculate_pattern(false, false, false, true) {
            // 13
            vec![[BOTTOM, BOTTOM_RIGHT, RIGHT]]
        } else if pattern == Field::calculate_pattern(false, false, true, false) {
            // 14
            vec![[BOTTOM_LEFT, BOTTOM, LEFT]]
        } else if pattern == Field::calculate_pattern(false, false, false, false) {
            vec![]
        } else {
            panic!("Not a valid pattern!");
        }
    }

    fn set(&mut self, pos: IVec2, value: f32) {
        self.dimensions = self.dimensions.union_point(pos);
        self.data.insert(pos, value);
    }

    fn get(&self, pos: &IVec2) -> &f32 {
        self.data.get(pos).unwrap_or(&0.)
    }

    const fn calculate_pattern(
        has_top_left: bool,
        has_top_right: bool,
        has_bottom_left: bool,
        has_bottom_right: bool,
    ) -> u32 {
        let first = if has_top_left { 1 } else { 0 };
        let second = if has_top_right { 1 } else { 0 };
        let third = if has_bottom_left { 1 } else { 0 };
        let fourth = if has_bottom_right { 1 } else { 0 };

        let pattern = (first << 0) + (second << 1) + (third << 2) + (fourth << 3);

        pattern
    }

    fn march(&self, isoValue: &f32) -> Mesh {
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut current_position = 0;
        let mut indices: Vec<u32> = Vec::new();

        for x in (self.dimensions.min.x - 1)..=(self.dimensions.max.x + 1) {
            for y in (self.dimensions.min.y - 1)..=(self.dimensions.max.y + 1) {
                let has_top_left = self.get(&IVec2 { x, y }) > isoValue;
                let has_top_right = self.get(&IVec2 { x: x + 1, y }) > isoValue;
                let has_bottom_left = self.get(&IVec2 { x, y: y + 1 }) > isoValue;
                let has_bottom_right = self.get(&IVec2 { x: x + 1, y: y + 1 }) > isoValue;
                let first = if has_top_left { 1 } else { 0 };
                let second = if has_top_right { 1 } else { 0 };
                let third = if has_bottom_left { 1 } else { 0 };
                let fourth = if has_bottom_right { 1 } else { 0 };

                let pattern = (first << 0) + (second << 1) + (third << 2) + (fourth << 3);

                let triangles = Field::get_pattern(pattern);
                for triangle in triangles {
                    for vertices in triangle {
                        positions.push([vertices[0] + x as f32, vertices[1] + y as f32, 0.]);
                        normals.push([0.0, 0.0, 1.0]);
                        indices.push(current_position);
                        current_position = current_position + 1;
                    }
                }
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_indices(Some(Indices::U32(indices)));

        mesh
    }
}

fn create_square() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 2.0, 0.0],
            [2.0, 2.0, 0.0],
            [1.0, 0.0, 0.0],
        ],
    );
    // Assign a UV coordinate to each vertex.
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0.0, 1.0], [0.5, 0.0], [1.0, 0.0], [0.5, 1.0]],
    );
    // Assign normals (everything points outwards)
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
    );
    // After defining all the vertices and their attributes, build each triangle using the
    // indices of the vertices that make it up in a counter-clockwise order.
    mesh.set_indices(Some(Indices::U32(vec![
        // First triangle
        0, 3, 1, // Second triangle
        1, 3, 2,
    ])));

    mesh
}
