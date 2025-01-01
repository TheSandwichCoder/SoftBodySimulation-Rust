#[cfg(not(target_arch = "wasm32"))]
use bevy::sprite::{Wireframe2dConfig, Wireframe2dPlugin};
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy::window::PrimaryWindow;
// use rand::Rng;

use crate:: settings:: *;

pub struct SBPlugin;

impl Plugin for SBPlugin{
    fn build(&self, app: &mut App){
        app.add_plugins((
            #[cfg(not(target_arch = "wasm32"))]
            Wireframe2dPlugin,
        ))
        .add_systems(Startup, spawn_sb_parent)
        .add_systems(Update, (spawn_sb, update_sb, update_sb_draw));
    }
}

#[derive(Component)]
pub struct SBParent;

#[derive(Component)]
pub struct SB{
    pub nodes: Vec<SBNode>,
    pub connections: Vec<SBConnection>,
    pub base_skeleton: Vec<Vec2>,
    pub skeleton: Vec<Vec2>,

    pub node_num: u8,
    pub bounding_box: BoundingBox,
    pub center: Vec2,
    pub angle: f32,
}

impl SB{
    fn get_center(&self) -> Vec2{
        let mut average_pos = Vec2::ZERO;
        
        for node in &self.nodes{
            average_pos += node.pos;
        }

        return average_pos / (self.node_num as f32);
    }
}

#[derive(Clone)]
pub struct SBNode{
    pub pos: Vec2,
    pub vel: Vec2,
}

impl SBNode{
    fn new(pos: Vec2) -> Self{
        Self{pos:pos, vel: Vec2::ZERO}
    }
}

#[derive(Clone)]
pub struct SBConnection{
    pub i1: usize,
    pub i2: usize,
    pub is_edge: bool,
    pub resting_length: f32,
}

impl SBConnection{
    fn new(i1: usize, i2: usize, is_edge: bool, resting_length: f32) -> Self{
        Self{i1, i2, is_edge, resting_length}
    }
}

#[derive(Component, Default, Reflect, Clone)]
pub struct BoundingBox{
    pub min_pos: Vec2,
    pub max_pos: Vec2, 
}

impl BoundingBox{
    fn zero() -> Self{
        Self{min_pos: Vec2::ZERO, max_pos: Vec2::ZERO}
    }
}

#[derive(Component, Default, Reflect, Clone)]
struct NodeIndex{
    i1: usize
}

#[derive(Component, Default, Reflect, Clone)]
struct ConnectionIndex{
    i1: usize,
    i2: usize
}

fn spawn_sb_parent(mut commands: Commands){
    commands.spawn((SpatialBundle::default(), SBParent, Name::new("Soft Body Parent")));
}

fn spawn_sb(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    // parent: Query<Entity, With<SBParent>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
){
    if !input.just_pressed(KeyCode::Space){
        return;
    }

    // let parent = parent.single();

    let shape = Mesh2dHandle(meshes.add(Circle::new(NODE_RADIUS)));
    
    let color = Color::rgb(1.0, 1.0, 1.0);

    let node_vec = vec![
        SBNode::new(Vec2::new(-50.0, 50.0)),
        SBNode::new(Vec2::new(50.0, 50.0)),
        SBNode::new(Vec2::new(-50.0, -50.0)),
        SBNode::new(Vec2::new(50.0, -50.0)),
    ];

    let connection_vec = vec![
        SBConnection::new(0, 1, true, DEFAULT_RESTING_LENGTH),
        SBConnection::new(1, 3, true, DEFAULT_RESTING_LENGTH),
        SBConnection::new(3, 2, true, DEFAULT_RESTING_LENGTH),
        SBConnection::new(2, 0, true, DEFAULT_RESTING_LENGTH),
        SBConnection::new(0, 3, false, DEFAULT_RESTING_LENGTH),
    ];

    let base_skeleton = vec![];
    let skeleton = vec![];

    let node_num : u8 = 4;

    let soft_body = SB{
        nodes: node_vec.clone(),
        connections: connection_vec.clone(),
        base_skeleton: base_skeleton,
        skeleton: skeleton,
        node_num: node_num,
        bounding_box: BoundingBox::zero(),
        center: Vec2::new(0.0, 0.0),
        angle: 0.0,
    };

    // Spawns the soft body 
    // commands.entity(parent).with_children(|commands|{

    commands.spawn((SpatialBundle::default(), soft_body, Name::new("Soft Body"))).with_children(|parent|{


        // I guess Im a noob for not using enumerate
        let mut counter: usize = 0;

        for node in &node_vec {
            parent.spawn((
                MaterialMesh2dBundle{
                    mesh: shape.clone(),
                    material: materials.add(color),
                    transform: Transform{
                        translation: node.pos.extend(0.0),
                        ..default()
                    },
                    ..default()
                },
                NodeIndex{i1: counter},
            ));

            counter += 1;
        }

        for connection in &connection_vec{
            // Define the start and end points
            let start = node_vec[connection.i1].pos;
            let end = node_vec[connection.i2].pos;

            // Calculate the midpoint, direction, and length
            let mid_point = (start + end) / 2.0;
            let direction = end - start;
            let length = direction.length();
            let angle = direction.y.atan2(direction.x);

            // Spawn a line
            parent.spawn((
                SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(mid_point.x, mid_point.y, 0.0),
                        rotation: Quat::from_rotation_z(angle),
                        scale: Vec3::new(length, 2.0, 1.0), // Length and thickness
                        ..Default::default()
                    },
                    sprite: Sprite {
                        color: Color::rgb(0.8, 0.2, 0.2),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ConnectionIndex{i1:connection.i1, i2:connection.i2}
            ));
        }
    });

    info!("Spawned new Soft Body");
}

fn update_sb(
    mut sbObjectQuery: Query<&mut SB>,
    time: Res<Time>,
){
    for mut sbObject in &mut sbObjectQuery{
        for mut node in &mut sbObject.nodes{
            node.vel -= GRAVITY * time.delta_seconds() * ITERATION_DELTA;
            node.pos += node.vel * time.delta_seconds() * ITERATION_DELTA;
        }

        sbObject.center = sbObject.get_center();
        // println!("{:?}", sbObject.center)
    }
}


fn update_sb_draw(
    soft_body_query: Query<(&SB, &Children)>,
    mut param_set: ParamSet<(
        Query<(&mut Transform, &NodeIndex)>,
        Query<(&mut Transform, &ConnectionIndex)>,
    )>,
    // mut node_query: Query<(&mut Transform, &NodeIndex)>,
    // mut line_query: Query<(&mut Transform, &ConnectionIndex)>,
) {
    // let mut query_1 = param_set.p0();
    // let mut query_2 = param_set.p1();

    for (soft_body, children) in &soft_body_query {
        for child in children {
            // println!("{:?}", child);

            if let Ok((mut transform, point_marker)) = param_set.p0().get_mut(*child) {
                // Update the position of the node
                let node = &soft_body.nodes[point_marker.i1];
                transform.translation = node.pos.extend(0.0);
            }

            // param_set.p1().get_mut()

            else if let Ok((mut transform, line_marker)) = param_set.p1().get_mut(*child) {
                // Update the position and length of the line
                let start = soft_body.nodes[line_marker.i1].pos;
                let end = soft_body.nodes[line_marker.i2].pos;

                let mid_point = (start + end) / 2.0;
                let direction = end - start;
                let length = direction.length();
                let angle = direction.y.atan2(direction.x);

                transform.translation = mid_point.extend(0.0);
                transform.rotation = Quat::from_rotation_z(angle);
                transform.scale = Vec3::new(length, 2.0, 1.0)
                // sprite.custom_size = Some(Vec2::new(length, sprite.custom_size.unwrap().y));
            }
        }
    }
}