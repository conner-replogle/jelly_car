use bevy::{prelude::*, sprite::Mesh2dHandle, render::mesh, input::common_conditions::input_just_released};
use bevy_egui::{EguiContexts, egui};

use crate::{GameState, helper::*, test};


pub struct SoftBodyPlugin;
impl Plugin for SoftBodyPlugin{
    fn build(&self, app: &mut App) {
        app
        .insert_resource(test::TestResource::default())
        .add_systems(Update, test::clear_test.run_if(input_just_released(KeyCode::Space)))
        .add_systems(Update, test::test_egui)
        .add_systems(Update, debug_forces)
        .add_systems(Update, test::test2.after(test::clear_test).run_if(input_just_released(KeyCode::Space)))
        //.add_systems(Update, update_jelly_mesh)
        .add_systems(FixedUpdate, clear_force.before(add_gravity))
        .add_systems(FixedUpdate, add_gravity.run_if(in_state(GameState::Run)))
        .add_systems(FixedUpdate, move_points.after(update_spring))
        .add_systems(FixedUpdate, (update_spring,jelly_collision).run_if(in_state(GameState::Run)).after(add_gravity))
        .insert_resource(FixedTime::new_from_secs(0.1));
    }
}

#[derive(Component)]
pub struct JellyCar{
    pub mass_points: Vec<Entity>,
    pub  springs: Vec<Entity>,
}


#[derive(Component)]
pub struct Gravity;

#[derive(Component,Debug,Clone, Copy)]
pub struct MassPoint{
    pub transform:Vec2,
    pub mass: f32,
    pub force: Vec2,
    pub velocity: Vec2,
    pub is_static: bool,
}

#[derive(Component)]
pub struct Spring{
    pub spring_constant: f32,
    pub stiffness:f32,
    pub distance:f32,
    pub from:Entity,
    pub to:Entity,
    pub inside: bool

}



fn clear_force(mut mass_points: Query<&mut MassPoint>){
    for mut point in mass_points.iter_mut(){
        point.force = Vec2::ZERO;
    }
}

fn jelly_collision(mut gizmos: Gizmos, jelly_cars: Query<&JellyCar>,mut mass_points: Query<&mut MassPoint>,springs: Query<&Spring>){
    //Check for collisions between bounding boxes


    let mut detected_collision_indexes: Vec<(usize,usize)> = Vec::new();
    for (index,car_a) in jelly_cars.iter().enumerate(){
        let a_points = mass_points.iter_many(&car_a.mass_points).map(|a| *a).collect();
        let a_box = bounding_box(&a_points);
        for (index_b,car_b) in jelly_cars.iter().enumerate(){
            if index == index_b || detected_collision_indexes.contains(&(index,index_b))|| detected_collision_indexes.contains(&(index_b,index)){
                continue;
            }
            let b_points = mass_points.iter_many(&car_b.mass_points).map(|a| *a).collect();
            let b_box = bounding_box(&b_points);

            if !a_box.intersect(b_box).is_empty(){
                gizmos.rect_2d(a_box.center(), 0.0, a_box.size(), Color::RED);
                gizmos.rect_2d(b_box.center(), 0.0, b_box.size(), Color::RED);
                detected_collision_indexes.push((index,index_b));
            }
        }
    }
    let jelly_cars = jelly_cars.iter().collect::<Vec<_>>();
    for collision in detected_collision_indexes{
        
        
        
        let  jelly_b = jelly_cars.get(collision.1).unwrap();
        
        let b_points:Vec<MassPoint> = mass_points.iter_many(&jelly_b.mass_points).map(|a| *a).collect();
        let b_bound: Rect = bounding_box(&b_points);

        let  jelly_a = jelly_cars.get(collision.0).unwrap();
       
        let a_points:Vec<MassPoint> = mass_points.iter_many(&jelly_a.mass_points).map(|a| *a).collect();
        
        let a_bound: Rect = bounding_box(&a_points);

        let mut handle_collision =|points : &Vec<MassPoint>,other_points : &Vec<MassPoint>,bound:Rect,jelly_car: &JellyCar,other_car: &JellyCar| {
            for (index,point) in points.iter().enumerate(){
              
                if !bound.contains(point.transform){
                    continue;
                }
                
            
                let line_to_outside: (Vec2, Vec2) = (point.transform,Vec2::new(bound.max.x+10.,point.transform.y));
                let (n_collisions, nearest_point,nearest_line) = count_line_intersections_horizontal(&lines(&mass_points,springs.iter_many(&other_car.springs).filter(|a| a.inside).collect()),line_to_outside,point.transform,&mut gizmos);
                if n_collisions % 2 == 1 {
                    //println!("Colliding");
                    if nearest_point == point.transform{
                        println!("SAME SPOT");
                        continue;
                    }
                    gizmos.line_2d(line_to_outside.0, line_to_outside.1, Color::GREEN);
                    gizmos.circle_2d(nearest_point, 10.0, Color::BLUE);

                    let (ent_a, ent_b) = nearest_line.2;

                    let point_a = mass_points.get(ent_a).unwrap();
                    let point_b = mass_points.get(ent_b).unwrap();

                    let mut avg_point = MassPoint{
                        transform:( point_a.transform + point_b.transform) / 2.0,
                        mass: (point_a.mass + point_b.mass)/2.0,
                        velocity: (point_a.velocity + point_b.velocity)/2.0,
                        force: Vec2::default(),
                        is_static:  point_a.is_static,
                    };
                    //println!("Nearest Point: {:?}",nearest_point);
                    let norm = (point.transform - nearest_point).normalize();
                    let r = point.velocity - 2.0 * (point.velocity * norm)* norm;

                    let v1 = norm * 1.0;
                    if !point.is_static{
                        let mut point: Mut<'_, MassPoint> = mass_points.get_mut(jelly_car.mass_points[index]).unwrap();
                    
                        point.transform = nearest_point;
                        gizmos.circle_2d(point.transform, 10.0, Color::WHITE);

                        point.velocity = r;
                    }
            
                    let line_distance = nearest_line.0.distance(nearest_line.1);

                    {
                        
                        let mut mass_point = mass_points.get_mut(ent_a).unwrap();
                        let distance = mass_point.transform.distance(nearest_point);

                        if !mass_point.is_static{
                            mass_point.transform -= (nearest_point - point.transform);//* (distance/line_distance);
                            //mass_point.velocity = -r;
                           
                        }
                    }
                    {
                        let mut mass_point = mass_points.get_mut(ent_b).unwrap();
                        let distance = mass_point.transform.distance(nearest_point);

                        if !mass_point.is_static{
                            mass_point.transform -= (nearest_point - point.transform) ;//* (distance/line_distance);
                           // mass_point.velocity = -r;
                        }
                    }
                    

                    
                }else{
                    gizmos.line_2d(line_to_outside.0, line_to_outside.1, Color::RED);

                }
            }
        };
        handle_collision(&b_points,&a_points,a_bound,&jelly_b,&jelly_a);
        handle_collision(&a_points,&b_points,b_bound,&jelly_a,&jelly_b);


    }
}


fn add_gravity(mut jelly_cars: Query<&mut MassPoint,With<Gravity>>){
    for mut points in jelly_cars.iter_mut(){
        let mass = points.mass;
        points.velocity += (Vec2::NEG_Y * 9.8) * mass;
    }
}

fn shape_matching(mut jelly_cars: Query<&mut JellyCar>,mut mass_points: Query<&mut MassPoint>){
    for car in jelly_cars.iter_mut(){
        let mass_points_vec:Vec<MassPoint> = mass_points.iter_many(&car.mass_points).copied().collect();
        //Detect if any of these points are within a certian radius of eachother
        for (index,point) in mass_points_vec.iter().enumerate(){
            for (index_b,point_b) in mass_points_vec.clone().iter().enumerate(){
                if index == index_b{
                    continue;
                }
                if point.transform.distance(point_b.transform) < 3.0{
                    let norm = (point.transform - point_b.transform).normalize();
                    let r: Vec2 = point.velocity - 2.0 * (point.velocity * norm)* norm;
                    let mut point = mass_points.get_mut(*car.mass_points.get(index).unwrap()).unwrap();
                    point.force += r;
                    point.transform -= norm *3.0
                }
            }
        }
    }

}

fn update_spring(mut jelly_cars: Query<&mut JellyCar>,mut mass_points: Query<&mut MassPoint>, springs: Query<& Spring>){
    for car in jelly_cars.iter_mut(){
        let springs:Vec<&Spring> = springs.iter_many(&car.springs).collect();
        for spring in springs.iter(){
            let  from: &MassPoint = mass_points.get(spring.from).unwrap();
            let  to: &MassPoint = mass_points.get(spring.to).unwrap();
            //println!("From: {:?} To {:?}",from.transform,to.transform);
            let distance_delta = to.transform.distance(from.transform);
            //println!("Distance {:?}",distance_delta);

            let force = (distance_delta- spring.distance)* spring.spring_constant;
            let norm_vec = (to.transform - from.transform).normalize();
            let vel_diff = to.velocity - from.velocity; 
            let dot = norm_vec.dot(vel_diff);
            let total_force =  force + dot * spring.stiffness;
            let from_force = norm_vec * total_force;
            let to_force = (from.transform - to.transform).normalize() * total_force;
            
            drop(from);
            drop(to);

            let mut from= mass_points.get_mut(spring.from).unwrap();
            from.force += from_force;
            drop(from);
            let mut to= mass_points.get_mut(spring.to).unwrap();
            //println!("Force: {:?}",to.force);
            to.force += to_force;

            
            
        }
    }
}


fn debug_forces(mut gizmos: Gizmos,mass_points: Query<&mut MassPoint>){
    for mut point in mass_points.iter(){
        gizmos.line_2d(point.transform, point.transform + point.velocity, Color::BLUE);
    }
}

fn move_points(mut mass_points: Query<&mut MassPoint>,time: Res<Time>){
    for mut point in mass_points.iter_mut(){
        if point.is_static{
            continue;
        }
        
        let force = point.force;
        let mass = point.mass;
        point.velocity += (force / mass)* time.delta_seconds_f64() as f32;
        let velocity =  point.velocity;
        point.transform += velocity * time.delta_seconds_f64() as f32;
        //println!("Velocity: {:?} Position {:? }",velocity,point.transform);
    }
}

    
    

fn update_jelly_mesh(mut assets: ResMut<Assets<Mesh>>,mut jelly_cars: Query<(&mut JellyCar,&mut Mesh2dHandle)>,mass_points: Query<&MassPoint>){

    for (mut car,mesh_han) in jelly_cars.iter_mut(){
        let Some(mesh) = assets.get_mut(&mesh_han.0) else{
            continue;
        };
        let points:Vec<&MassPoint> = mass_points.iter_many(&car.mass_points).collect();

        let points_transform: (f32,f32) = points.iter().map(|f| (f.transform.x,f.transform.y)).reduce(|a,b| (a.0 + b.0,a.1 + b.1)).unwrap();

        
        
        // transform.x = average.0;
        // transform.y = average.1;



        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            points.iter().map(|a| [a.transform.x,a.transform.y,0.]).collect::<Vec<_>>(),
        );
    
        // In this example, normals and UVs don't matter,
        // so we just use the same value for all of them
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; 4]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; 4]);
        
        mesh.set_indices(Some(mesh::Indices::U32(vec![0, 3, 2,0,2,1])));
    }

}






    
    

    
    
    

    

