#[cfg(not(target_arch = "wasm32"))]
use bevy::sprite::{Wireframe2dConfig, Wireframe2dPlugin};
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy::window::PrimaryWindow;
use std::f32::NAN;
// use rand::Rng;

use crate:: settings:: *;

pub struct SBPlugin;

impl Plugin for SBPlugin{
    fn build(&self, app: &mut App){
        app.add_plugins((
            #[cfg(not(target_arch = "wasm32"))]
            Wireframe2dPlugin,
        ))
        .add_systems(Update, (spawn_sb, update_processes, update_sb_draw))
        .add_systems(Update, interact);
    }
}

#[derive(Component)]
pub struct SB{
    pub nodes: Vec<SBNode>,
    pub connections: Vec<SBConnection>,
    pub base_skeleton: Vec<Vec2>,
    pub base_skeleton_norm: Vec<Vec2>,
    pub skeleton: Vec<Vec2>,

    pub node_num: u8,
    pub bounding_box: BoundingBox,
    pub center: Vec2,
    pub angle: f32,
}

impl SB{
    fn new(nodes: &Vec<SBNode>, connections: &Vec<SBConnection>) -> Self{
        let node_num : u8 = nodes.len() as u8; 

        let mut center = Vec2::ZERO;

        for node in nodes{
            center += node.read_pos;
        }

        center /= node_num as f32;        

        let mut base_skeleton: Vec<Vec2> = vec![Vec2::ZERO; node_num as usize];
        let mut base_skeleton_norm: Vec<Vec2> = vec![Vec2::ZERO; node_num as usize];


        for i in 0..(node_num as usize){
            base_skeleton[i] = nodes[i].read_pos - center;
            base_skeleton_norm[i] = (nodes[i].read_pos - center).normalize();
        }

        let mut sb: SB = SB{
            nodes: nodes.clone(),
            connections: connections.clone(),
            base_skeleton: base_skeleton,
            base_skeleton_norm: base_skeleton_norm,
            skeleton: vec![Vec2::ZERO; node_num as usize],
            node_num: node_num,
            bounding_box: BoundingBox::zero(),
            center: center,
            angle: 0.0,
        };

        sb.update_skeleton();

        return sb;
    }

    fn get_center(&self) -> Vec2{
        let mut average_pos = Vec2::ZERO;
        
        for node in &self.nodes{
            average_pos += node.read_pos;
        }

        return average_pos / (self.node_num as f32);
    }

    fn get_angle(&self) -> f32{
        let mut average_angle : f32 = 0.0;

        for i1 in 0..(self.node_num as usize){
            let vec1 = (self.nodes[i1].read_pos - self.center).normalize();
            let vec2 = self.base_skeleton[i1].normalize();

            let dot = vec1.dot(vec2).clamp(-1.0, 1.0);

            let cross = vec1.perp_dot(vec2);

            let angle: f32;

            if cross < 0.0{
                angle = dot.acos();
            }
            else{
                angle = TAU - dot.acos();
            }

            if angle - average_angle < PI{
                average_angle += angle / (self.node_num as f32);
            }
            else{
                average_angle -= (TAU - angle) / (self.node_num as f32);
            }

            // println!("ave a:{}", average_angle);
        }

        if average_angle.abs() < 0.001{
            return 0.0;
        }

        return average_angle;
    }

    fn update_skeleton(&mut self){
        let mut counter: usize = 0;

        for vec in &self.base_skeleton{
            self.skeleton[counter] = vec_rotate(vec, self.angle) + self.center;

            counter += 1;
        }
    }

    fn update_bounding_box(&mut self){
        let mut min_vec: Vec2 = Vec2::new(100000.0, 100000.0);
        let mut max_vec: Vec2 = Vec2::new(-100000.0, -100000.0);

        for node in &self.nodes{
            if node.read_pos.x < min_vec.x{
                min_vec.x = node.read_pos.x;
            }
            else if node.read_pos.x > max_vec.x{
                max_vec.x = node.read_pos.x;
            }

            if node.read_pos.y < min_vec.y{
                min_vec.y = node.read_pos.y;
            }
            else if node.read_pos.y > max_vec.y{
                max_vec.y = node.read_pos.y;
            }
        }

        self.bounding_box.min_pos = min_vec;
        self.bounding_box.max_pos = max_vec; 
    }
}

#[derive(Clone)]
pub struct SBNode{
    pub read_pos: Vec2,
    pub write_pos: Vec2,
    pub vel: Vec2,
    pub recently_collided: bool,
}

impl SBNode{
    fn new(pos: Vec2) -> Self{
        Self{read_pos:pos, write_pos:pos, vel: Vec2::ZERO, recently_collided: false}
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


fn vec_rotate(
    vec: &Vec2,
    angle: f32,
) -> Vec2{
    let cos_angle: f32 = angle.cos();
    let sin_angle: f32 = angle.sin();

    let x_rot: f32 = vec.x * cos_angle - vec.y * sin_angle;
    let y_rot: f32 = vec.x * sin_angle + vec.y * cos_angle;

    return Vec2::new(x_rot, y_rot);
}

fn world_to_screen_coords(
    vec: Vec2
) -> Vec2{
    return Vec2::new(vec.x + HALF_DIM.x, -vec.y + HALF_DIM.y);
}

fn axis_aligned_line_overlap(
    min_l1: f32,
    max_l1: f32,
    min_l2: f32,
    max_l2: f32,
) -> bool{
    return (min_l2 >= min_l1 && min_l2 <= max_l1) || (max_l2 >= min_l1 && max_l2 <= max_l1);
}

fn bounding_box_collision(
    bb1: &mut BoundingBox,
    bb2: &mut BoundingBox,
) -> bool{
    return axis_aligned_line_overlap(bb1.min_pos.x, bb1.max_pos.x, bb2.min_pos.x, bb2.max_pos.x) && axis_aligned_line_overlap(bb1.min_pos.y, bb1.max_pos.y, bb2.min_pos.y, bb2.max_pos.y);
}

fn soft_body_collision(
    sb1: &mut SB,
    sb2: &mut SB,
){
    // if bounding_box_collision(&mut sb1.bounding_box, &mut sb2.bounding_box){

    if true{
        let mut counter = 0;
        for mut node in &mut sb1.nodes{
            // point intersection

            // println!("atlesast atleast here");
            if sb_point_intersection(node.read_pos, sb2){
                node.recently_collided = true;
                // println!("atleast heere");
                
                let (coll_pt, dist, conn_index, dot) = get_closest_edge(node.read_pos, sb1.center, sb2);

                // println!("Collision: pos {:?} center1 {:?} center2 {:?} coll_pt {:?} dist {} node_i {} conn_i {} dot {}", world_to_screen_coords(node.read_pos),world_to_screen_coords(sb1.center),world_to_screen_coords(sb2.center), world_to_screen_coords(coll_pt), dist, counter, conn_index, dot);

                let connection = &sb2.connections[conn_index];

                // the program probably found a faulty intersection
                if dist >= connection.resting_length/2.0{
                    // println!("here");
                    counter += 1;
                    continue;
                }

                // println!("{}", dot);
                
                soft_body_collision_response(&mut node, sb2, connection.i1, connection.i2, coll_pt, dot);
                
            }

            counter += 1;
        }
    }
}

fn soft_body_collision_response(
    node: &mut SBNode,
    sb2: &mut SB,
    con_pt1_index: usize,
    con_pt2_index: usize,
    coll_pos: Vec2,
    dot: f32,
){
    let vec = node.read_pos - coll_pos;

    let node_vec = -vec;
    let con_pt1_vec = vec * (1.0 - dot);
    let con_pt2_vec = vec * dot;

    node.write_pos += node_vec;
    sb2.nodes[con_pt1_index].write_pos += con_pt1_vec;
    sb2.nodes[con_pt2_index].write_pos += con_pt2_vec;

    node.vel += node_vec;
    sb2.nodes[con_pt1_index].vel += con_pt1_vec;
    sb2.nodes[con_pt2_index].vel += con_pt2_vec;
}


// true if left and false if right
fn line_pt_lateral(
    pt: Vec2,
    line_pt1: Vec2,
    line_pt2: Vec2,
) -> bool{
    let ab: Vec2;

    if line_pt2.y > line_pt1.y{
        ab = line_pt2 - line_pt1;
    }
    else{
        ab = line_pt1 - line_pt2;
    }

    let ap = pt - line_pt1;

    return ab.perp_dot(ap) > 0.0;
}

fn sb_point_intersection(
    pt: Vec2,
    sb: &mut SB,
) -> bool{
    let mut intersection_counter_y = 0;

    for connection in &sb.connections{
        // dont want to consider edges
        if !connection.is_edge{
            continue;
        }

        // make sure the line is to the left
        let lateral = line_pt_lateral(pt, sb.nodes[connection.i1].read_pos, sb.nodes[connection.i2].read_pos);

        if lateral{
            let y1 = sb.nodes[connection.i1].read_pos.y;
            let y2 = sb.nodes[connection.i2].read_pos.y;

            let mut intersect : bool = false;

            // if (y1 - y2).abs() < 0.01 && (pt.y == y1 || pt.y == y2){
            //     continue;
            // } 

            if y1 > y2{
                if pt.y <= y1 && pt.y >= y2{
                    intersection_counter_y += 1;
                    intersect = true;
                }
            }
            else if y1 < y2{
                if pt.y >= y1 && pt.y <= y2{
                    intersection_counter_y += 1;
                    intersect = true;
                }
            }

            // if intersect{
            //     println!("True: pt:{} n1:{} n2:{}", world_to_screen_coords(pt), world_to_screen_coords(sb.nodes[connection.i1].read_pos), world_to_screen_coords(sb.nodes[connection.i2].read_pos));
            // }
            // else{
            //     println!("False pt:{} n1:{} n2:{}", world_to_screen_coords(pt), world_to_screen_coords(sb.nodes[connection.i1].read_pos), world_to_screen_coords(sb.nodes[connection.i2].read_pos));
            // }
        }
    }
    // println!("{}", intersection_counter_y);

    return intersection_counter_y % 2 == 1;
}

// returns the distance from edge and 
// how far along the edge
fn point_line_dist(
    node_pt: Vec2,
    line_pt1: Vec2,
    line_pt2: Vec2,
) -> (Vec2, f32){
  let ab = line_pt2 - line_pt1;
  let ap = node_pt - line_pt1;
  
  let t = ap.dot(ab) / ab.dot(ab);

  return (line_pt1 + (line_pt2 - line_pt1) * t, t);
}

fn get_closest_edge(
    node_pos: Vec2,
    center: Vec2, 
    sb: &mut SB,
) -> (Vec2, f32, usize, f32){
    let mut min_dist : f32 = 10000000.0; // distance to edge
    let mut best_pt : Vec2 = Vec2::ZERO; // point on edge
    let mut connection_index: usize = 0; // edge index
    let mut fin_dot:f32 = 0.0; // how far along the edge

    let mut counter: usize = 0;

    for connection in &sb.connections{

        // edges cannot be colliding
        if !connection.is_edge{
            counter += 1;
            continue;
        }

        let pt1 = &sb.nodes[connection.i1];
        let pt2 = &sb.nodes[connection.i2];

        // dis from edge and how far along the edge is
        let (closest_pt, dot) = point_line_dist(node_pos, pt1.read_pos, pt2.read_pos);

        // make sure the point is near the line
        if dot > 1.1 || dot < -0.1{
            counter += 1;
            // println!("dot skip");
            continue;
        }

        let mut connection_normal = (pt1.read_pos - pt2.read_pos).normalize().perp();

        // make sure normal is facing outwards
        if (sb.center - closest_pt).normalize().dot(connection_normal) > 0.0{
            connection_normal = -connection_normal;
        }

        let center_to_point = (center - closest_pt).normalize();

        // make sure the center is facing the outside
        if connection_normal.dot(center_to_point) < 0.2{
            counter += 1;
            continue;
        }

        let dist = (closest_pt - node_pos).length_squared();

        if dist < min_dist{
            best_pt = closest_pt;
            min_dist = dist;
            connection_index = counter;
            fin_dot = dot;
            // println!("new min dist {}, new conn index {}", min_dist, connection_index);
        }

        counter += 1;
    }
    // println!("pt1:{:?} dist:{:?} conn_i:{} dot:{}", best_pt, min_dist.sqrt(), connection_index, fin_dot);
    return (best_pt, min_dist.sqrt(), connection_index, fin_dot);
}

fn interact(
    mut commands: Commands,
    mut SB_query: Query<&mut SB>,
    time: Res<Time>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mouseInput: Res<ButtonInput<MouseButton>>,
    keyInput: Res<ButtonInput<KeyCode>>,
){
    let mut position = Vec2::new(0.0, 0.0);

    if let Some(mouse_position) = q_windows.single().cursor_position() {
        // println!("Cursor is inside the primary window, at {:?}", position);
        position = Vec2::new(mouse_position.x, mouse_position.y);
    } else {
        // println!("Cursor is not in the game window.");
    }

    let mut rel_position: Vec2 = position - HALF_DIM;
    rel_position.y = -rel_position.y;



    // is this ugly? yes. But hey I acknowledged it, and thats what matters
    if mouseInput.pressed(MouseButton::Left) {
        let mut min_dist : f32 = 100000.0;        
        
        for sb in &mut SB_query{
            for node in &sb.nodes{
                let dist: f32 = (rel_position - node.read_pos).length();
                
                if dist < min_dist{
                    min_dist = dist
                }
            }
        }
        
        for mut sb in &mut SB_query{
            for mut node in &mut sb.nodes{
                let dist: f32 = (rel_position - node.read_pos).length();
                
                if dist == min_dist{
                    node.write_pos = rel_position;
                    node.vel = Vec2::ZERO;
                    break;
                }
            }
        }
    }
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

    let node_vec2 = vec![
        SBNode::new(Vec2::new(-40.0, 350.0)),
        SBNode::new(Vec2::new(60.0, 350.0)),
        SBNode::new(Vec2::new(-40.0, 250.0)),
        SBNode::new(Vec2::new(60.0, 250.0)),
    ];

    let connection_vec = vec![
        SBConnection::new(0, 1, true, DEFAULT_RESTING_LENGTH),
        SBConnection::new(0, 2, true, DEFAULT_RESTING_LENGTH),
        SBConnection::new(1, 3, true, DEFAULT_RESTING_LENGTH),
        SBConnection::new(2, 3, true, DEFAULT_RESTING_LENGTH),
        SBConnection::new(0, 3, false, 141.0),
        SBConnection::new(1, 2, false, 141.0)
    ];

    // let base_skeleton = vec![];
    // let skeleton = vec![];

    for i in 0..2{
        let soft_body : SB;

        if i == 0{
            soft_body = SB::new(&node_vec, &connection_vec);
        }
        else{
            soft_body = SB::new(&node_vec2, &connection_vec);
        }

        commands.spawn((SpatialBundle::default(), soft_body, Name::new("Soft Body"))).with_children(|parent|{
            // I guess Im a noob for not using enumerate
            let mut counter: usize = 0;

            for node in &node_vec {
                parent.spawn((
                    MaterialMesh2dBundle{
                        mesh: shape.clone(),
                        material: materials.add(color),
                        transform: Transform{
                            translation: node.read_pos.extend(0.0),
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
                let start = node_vec[connection.i1].read_pos;
                let end = node_vec[connection.i2].read_pos;

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
    }  

    info!("Spawned new Soft Body");
}

fn update_processes(
    mut SB_query: Query<&mut SB>,
    time: Res<Time>,
){
    for mut sbObject in &mut SB_query{
        for mut node in &mut sbObject.nodes{
            node.recently_collided = false;
        } 
    }

    for i in 0..ITERATION_COUNT{
        update_sb(&mut SB_query, 0.1 as f32);
        update_sb_collisions(&mut SB_query, 0.1 as f32);
    }

    // println!("new");

    let mut counter = 0;
    for sbObject in &SB_query{
        // for node in &sbObject.nodes{
        //     println!("{} {:?} {:?}", counter, world_to_screen_coords(node.read_pos), Vec2::new(node.vel.x, -node.vel.y));
        // }
        counter += 1;  
    }

    
}

fn update_sb_read_pos(
    mut sb: &mut SB, 
){
    for mut node in &mut sb.nodes{
        node.read_pos = node.write_pos;
    }
}

fn update_sb(
    mut sbObjectQuery: &mut Query<&mut SB>,
    dt: f32,
){
    for mut sbObject in sbObjectQuery{
        simulation_update(&mut sbObject, dt as f32);
        skeleton_simulation(&mut sbObject, dt as f32);

        for mut node in &mut sbObject.nodes{
            node.vel -= GRAVITY * dt * ITERATION_DELTA;
            node.write_pos += node.vel * dt * ITERATION_DELTA;
        }
        update_sb_read_pos(&mut sbObject);

        sbObject.update_bounding_box();

        container_collision(&mut sbObject);

        sbObject.center = sbObject.get_center();
        sbObject.angle = sbObject.get_angle();

        // println!("c:{:?} a:{}", world_to_screen_coords(sbObject.center), sbObject.angle);
        sbObject.update_skeleton();
    }
}

fn update_sb_collisions(
    mut sbObjectQuery: &mut Query<&mut SB>,
    dt: f32,
){
    let mut iter = sbObjectQuery.iter_combinations_mut();

    while let Some([mut SB1, mut SB2]) =
        iter.fetch_next()
    {
        soft_body_collision(&mut SB1, &mut SB2);
        soft_body_collision(&mut SB2, &mut SB1);
    }
    
}

fn container_collision(
    mut sbObject: &mut SB,
){
    for mut node in &mut sbObject.nodes{
        if node.read_pos.y < -HALF_DIM.y{
            node.write_pos.y = -HALF_DIM.y;
            node.vel.y = 0.0;
        }

        if node.read_pos.x > HALF_DIM.x{
            node.write_pos.x = HALF_DIM.x;
            node.vel.x = 0.0;
        }

        else if node.read_pos.x < -HALF_DIM.x{
            node.write_pos.x = -HALF_DIM.x;
            node.vel.x = 0.0;
        }
    }
}

fn simulation_update(
    mut sbObject: &mut SB,
    dt: f32,
){
    for connection in &sbObject.connections{
        let node1 = &sbObject.nodes[connection.i1];
        let node2 = &sbObject.nodes[connection.i2];

        let vec = node2.read_pos - node1.read_pos;
        let vec_norm = vec.normalize();

        if vec_norm.is_nan(){
            continue;
        }

        let vec_length = vec.length();

        let vel_diff = node2.vel - node1.vel;
        
        let dot = vec_norm.dot(vel_diff);

        let spring_strength = connection.resting_length - vec_length;

        let force = ((DEFAULT_STIFFNESS * spring_strength) - (dot * 0.5 * DEFAULT_DAMPENING)).clamp(-1000.0, 1000.0);

        let vector_force = vec_norm * force * dt * ITERATION_DELTA;

        // println!("f:{} f1:{} f2:{} p1:{:?} p2:{:?} final_f:{:?}", force, DEFAULT_STIFFNESS * spring_strength, dot * 0.5 * DEFAULT_DAMPENING,world_to_screen_coords(node1.read_pos),world_to_screen_coords(node2.read_pos), Vec2::new(vector_force.x, -vector_force.y));

        sbObject.nodes[connection.i1].vel -= vec_norm * force * dt * ITERATION_DELTA;
        sbObject.nodes[connection.i2].vel += vec_norm * force * dt * ITERATION_DELTA;

        // println!("new vec1 {:?} new vec2 {:?}", Vec2::new(sbObject.nodes[connection.i1].vel.x, -sbObject.nodes[connection.i1].vel.y), Vec2::new(sbObject.nodes[connection.i2].vel.x, -sbObject.nodes[connection.i2].vel.y));
    }
}

fn skeleton_simulation(
    mut sbObject: &mut SB,
    dt: f32,
){
    for index in 0..(sbObject.node_num as usize){
        let mut node1 = &mut sbObject.nodes[index];
        let skeleton_pos = &sbObject.skeleton[index];

        let vec = *skeleton_pos - node1.read_pos;
        let vec_norm = vec.normalize();

        if vec_norm.is_nan(){
            continue;
        }

        // println!("skel_pos:{:?} node_pos:{:?} vec:{:?}", world_to_screen_coords(*skeleton_pos), world_to_screen_coords(node1.read_pos), Vec2::new(vec.x, -vec.y));

        let force = (10.0 * -vec.length()).clamp(-1000.0, 1000.0);
        // let force = 1.0;

        // println!("pos {}", vec_norm);

        node1.vel -= vec_norm * force * dt * ITERATION_DELTA;
    }
}

fn update_sb_draw(
    soft_body_query: Query<(&SB, &Children)>,
    mut param_set: ParamSet<(
        Query<(&mut Transform, &NodeIndex)>,
        Query<(&mut Transform, &ConnectionIndex)>,
    )>,
) {
    for (soft_body, children) in &soft_body_query {
        for child in children {

            if let Ok((mut transform, point_marker)) = param_set.p0().get_mut(*child) {
                // Update the position of the node
                let node = &soft_body.nodes[point_marker.i1];
                transform.translation = node.read_pos.extend(0.0);

                if node.recently_collided{
                    transform.scale = Vec3::new(2.0, 2.0, 2.0); 
                }
                else{
                    transform.scale = Vec3::new(1.0, 1.0, 1.0);
                }
            }

            else if let Ok((mut transform, line_marker)) = param_set.p1().get_mut(*child) {
                // Update the position and length of the line
                let start = soft_body.nodes[line_marker.i1].read_pos;
                let end = soft_body.nodes[line_marker.i2].read_pos;

                let mid_point = (start + end) / 2.0;
                let direction = end - start;
                let length = direction.length();
                let angle = direction.y.atan2(direction.x);

                transform.translation = mid_point.extend(0.0);
                transform.rotation = Quat::from_rotation_z(angle);
                transform.scale = Vec3::new(length, 2.0, 1.0)
            }
        }
    }
}