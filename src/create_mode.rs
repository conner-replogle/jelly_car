use bevy::{prelude::*, render::camera::RenderTarget, window::PrimaryWindow, input::common_conditions::{input_pressed, input_just_released}};
use bevy_egui::{EguiPlugin, EguiContexts, egui};

use crate::{GameState, soft_body::{MassPoint, Spring, JellyCar, Gravity}};




pub struct EditPlugin;

#[derive(Resource,Default )]
struct DraggingSpring{
    first: Option<Entity>,
    dragging:bool,

}

#[derive(Resource,Default )]
struct HighlightedPoint(Option<Entity>);

#[derive(Resource,Default )]
struct EditableJellyCar{
    ent: Option<Entity>,
    gravity:bool,
}


impl Plugin for EditPlugin {
    fn build(&self, app: &mut App) {
        app
        
        .insert_resource(DraggingSpring::default())
        .insert_resource(HighlightedPoint::default())
        .insert_resource(EditableJellyCar::default())
        .add_plugins(EguiPlugin)
        .add_systems(Update, ui_edit_mode.run_if(in_state(GameState::Create)))

        
   
        .add_systems(Update, check_selected_points.before(show_points))
        .add_systems(Update, (show_points,show_springs))
        .add_systems(Update, add_points.before(ui_edit_mode).run_if(input_just_released(MouseButton::Left)).run_if(in_state(GameState::Create)))
        .add_systems(Update, drag_add_spring.before(ui_edit_mode).run_if(in_state(GameState::Create)))
        ;

            
    }
}

fn ui_edit_mode(mut contexts: EguiContexts, mut edit_car: ResMut<EditableJellyCar>,mut commands: Commands,jelly_cars: Query<&JellyCar>) {
    egui::Window::new("Jelly Car Maker").show(contexts.ctx_mut(), |ui| {
        if edit_car.ent.is_some(){
            let ent =edit_car.ent.unwrap();
            if ui.button("Doen").clicked(){
                edit_car.ent = None;
            }
            if ui.button("Delete").clicked(){
                let car = jelly_cars.get(ent).unwrap();
                car.mass_points.iter().for_each(|ent|{
                    commands.entity(*ent).despawn_recursive();
                });
                car.springs.iter().for_each(|ent|{
                    commands.entity(*ent).despawn_recursive();
                });
                commands.entity(ent).despawn_recursive();
                edit_car.ent = None;
            }
            if edit_car.gravity{
                if ui.button("Gravity").clicked(){
                    edit_car.gravity = false;
                }

            }else{
                if ui.button("No Gravity").clicked(){
                    edit_car.gravity = true;
                }
            }
            
        }else{
            if ui.button("Start").clicked(){
                edit_car.ent = Some(commands.spawn(JellyCar{
                    mass_points:Vec::new(),
                    springs:Vec::new()
                }).id());
            }

        }
        
    });
}


fn check_selected_points(windows: Query<&mut Window,With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,mut highlighted_point:ResMut<HighlightedPoint>, jelly_cars: Query<(& MassPoint,Entity)>){
    let mouse_pos = get_mouse_pos(&windows,&camera_q);
    let mut selected = None;

    for (point,ent) in jelly_cars.iter(){
        if mouse_pos.is_some() && point.transform.distance(mouse_pos.unwrap()) < 4.0{
            selected = Some(ent);
        }
    }
    highlighted_point.0 = selected.map(|p| p);

}

fn show_points(mut gizmos: Gizmos, mass_points: Query<(& MassPoint,Entity)>,h_point: Res<HighlightedPoint>){

    let selected_point =h_point.0.map(|a| mass_points.get(a).unwrap().1);
    for (point,ent) in mass_points.iter(){
        if selected_point.is_some() && selected_point.unwrap() == ent {
            gizmos.circle(point.transform.extend(0.0),Vec3::Z, 5., Color::RED);
        }else{
            gizmos.circle(point.transform.extend(0.0),Vec3::Z, 3., Color::GREEN);

        }
    }
}
fn show_springs(mut gizmos: Gizmos, mass_points: Query<& MassPoint>,springs:Query<& Spring> ){

    for spring in springs.iter(){
        let from = mass_points.get(spring.from).unwrap().transform;
        let to = mass_points.get(spring.to).unwrap().transform;

        gizmos.line_2d(from, to, if spring.inside { Color::RED} else{Color::GREEN} );
    }
}


fn get_mouse_pos(windows: &Query<&mut Window,With<PrimaryWindow>>,
    camera_q: &Query<(&Camera, &GlobalTransform)>) -> Option<Vec2>{
    let (camera, camera_transform) = camera_q.single();

    // get the window that the camera is displaying to (or the primary window)
    let window = windows.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate()) else{
            return None;
        };
    return Some(world_position);
}

fn add_points(mut commands: Commands,windows: Query<&mut Window,With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,car: Res<EditableJellyCar>,mut jelly_cars: Query<&mut JellyCar>,dragging_spring:Res<DraggingSpring>,point: Res<HighlightedPoint>){
    if dragging_spring.first.is_some() || point.0.is_some() || car.ent.is_none(){
        return;
    }
    let mouse_pos = get_mouse_pos(&windows,&camera_q).unwrap();
    println!("{}", mouse_pos);
    if (mouse_pos.x < -300.){
        return;
    }
    let ent;
    if car.gravity{
        ent = commands.spawn((MassPoint{
            transform: mouse_pos,
            mass: 1.0,
            velocity: Vec2::ZERO,
            force: Vec2::ZERO,
            is_static: false
        },Gravity)).id();

    }else{
        ent = commands.spawn((MassPoint{
            transform: mouse_pos,
            mass: 1.0,
            velocity: Vec2::ZERO,
            is_static: true,

            force: Vec2::ZERO,
        })).id();
    }
    

    jelly_cars.get_mut(car.ent.unwrap()).unwrap().mass_points.push(ent);

    
        
        
 

}
fn drag_add_spring(input: Res<Input<MouseButton>>,car: Res<EditableJellyCar>,mut jelly_cars: Query<&mut JellyCar>,mut commands: Commands,mass_points: Query<& MassPoint>,mut dragging_spring:ResMut<DraggingSpring>,point: Res<HighlightedPoint>){
    if car.ent.is_none(){
        return;
    }
    if input.just_pressed(MouseButton::Left) && point.0.is_some(){
        dragging_spring.first = point.0;
        dragging_spring.dragging = true;

    }else if dragging_spring.first.is_some(){
        if input.pressed(MouseButton::Left){
            return;
        }

        let Some(point) = point.0 else{
            dragging_spring.first = None;
            return;
        };
        if point == dragging_spring.first.unwrap(){
            dragging_spring.first = None;
            return;
        }
        let from = mass_points.get(dragging_spring.first.unwrap()).unwrap().transform;
        let to = mass_points.get(point).unwrap().transform;

        let ent = commands.spawn(Spring{
            from: dragging_spring.first.unwrap(),
            to:point,
            distance: from.distance(to),
            spring_constant: 500.0,
            stiffness:8.0,
            inside: false
            
        }).id();
        jelly_cars.get_mut(car.ent.unwrap()).unwrap().springs.push(ent);


    }


    

    
    


}