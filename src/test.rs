use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::soft_body::*;

#[derive(Resource)]
pub struct TestResource{
    test_bodies: Vec<Entity>,
    stiffness: f32,
    spring_constant: f32,
    mass:f32,

}
impl Default for TestResource{
    fn default() -> Self {
        Self{
            test_bodies: Vec::new(),
            stiffness: 8.0,
            spring_constant: 500.0,
            mass: 1.0
        }
    }
}


pub fn test_egui(mut contexts: EguiContexts,mut test_resouce: ResMut<TestResource>,mut time: ResMut<FixedTime>){
    egui::Window::new("Test Edit").show(&contexts.ctx_mut(), |ui| {
        ui.label("Test");
        ui.add(egui::Slider::new(&mut test_resouce.stiffness, 0.0..=1000.0).text("Stiffness"));
        ui.add(egui::Slider::new(&mut test_resouce.spring_constant, 0.0..=1000.0).text("Spring Constant"));
        ui.add(egui::Slider::new(&mut test_resouce.mass, 0.0..=1000.0).text("Mass"));
        let mut time_delta = time.period.as_secs_f32();
        if ui.add(egui::Slider::new(&mut time_delta, 0.0f32..=1.0f32).text("Time Delta")).changed(){
            time.period = Duration::from_secs_f32(time_delta);
        }
    });
}
#[derive(Debug)]
pub struct ScaffoldBox{
    pub start: Vec2,
    pub size: Vec2,
    pub density: (u32,u32),
    pub mass: f32,
    pub spring_const: f32,
    pub spring_damp: f32,
    pub internal_scaffold: bool,
    pub is_static:bool

}
//refactor this to use scaffold box
fn spawn_scaffold_square(commands: &mut Commands,config: ScaffoldBox) -> Entity{
    let ScaffoldBox{start,size,density,mass,spring_const,spring_damp,internal_scaffold,is_static} = config;
    println!("Spawning Scaffold Box: {:?} at {:?}",config,start);
    let mut first_box_ids = Vec::new();
    let mut positions = Vec::new(); 
    let gap = size / Vec2::new(density.0 as f32,density.1 as f32);
 
    for x in 0..=density.0 as u32{
        for y in 0..=density.1 as u32{
            
            let position = Vec2::new(start.x + (x as f32  * gap.x) ,start.y - (y as f32  * gap.y));
            println!("Position: {:?} at {x} ,{y}",position);
            positions.push(position);
            first_box_ids.push(commands.spawn((Gravity,MassPoint{
                transform: position,
                mass,
                velocity: Vec2::ZERO,
                force: Vec2::ZERO,
                
            is_static:is_static 
            })).id(),);
        }
    }
    let mut spring_ids = Vec::new();
    
    for x in 0..=density.0{
        for y in 0..=density.1{
            let index = ((x  * (density.1+1) ) + y ) as usize;
            if y != density.1 as u32{
                spring_ids.push(commands.spawn(Spring{
                    from: first_box_ids[index],
                    to: first_box_ids[index+1],
                    distance: positions[index].distance(positions[index+1]),
                    spring_constant: spring_const,
                    stiffness: spring_damp,
                    inside: x == 0 || x == density.0
                }).id());
            }
            if x != density.0 as u32{
                spring_ids.push(commands.spawn(Spring{
                    from: first_box_ids[index],
                    to: first_box_ids[index+((density.1+1) as usize)],
                    distance: positions[index].distance(positions[index+((density.1+1) as usize)]),
                    spring_constant: spring_const,
                    stiffness: spring_damp,
                    inside: y == 0 || y == density.1
                }).id());
            }
            if internal_scaffold{
                if x != density.0 as u32 && y != density.1 as u32{
                    spring_ids.push(commands.spawn(Spring{
                        from: first_box_ids[index],
                        to: first_box_ids[index+((density.1+2) as usize)],
                        distance: positions[index].distance(positions[index+((density.1+2) as usize)]),
                        spring_constant: spring_const,
                        stiffness: spring_damp,
                        inside: false
                    }).id());
                }
                if x != density.0 as u32 && y != 0 as u32{
                    spring_ids.push(commands.spawn(Spring{
                        from: first_box_ids[index],
                        to: first_box_ids[index+density.1 as usize],
                        distance: positions[index].distance(positions[index+density.1 as usize]),
                        spring_constant: spring_const,
                        stiffness: spring_damp,
                        inside: false
                    }).id());
                }

            }
            
        }
    }
    
    return commands.spawn(JellyCar{
        mass_points: first_box_ids.clone(),
        springs: spring_ids.clone()
    }).push_children(&spring_ids).push_children(&first_box_ids).id();
}

pub fn clear_test(mut commands: Commands,mut test_resouce: ResMut<TestResource>){
    for ent in test_resouce.test_bodies.iter(){
        commands.entity(*ent).despawn_recursive();
    }
    test_resouce.test_bodies = Vec::new();
}
pub fn test2(
    mut commands: Commands,
    mut test_resouce: ResMut<TestResource>
) {
    {
        let config = ScaffoldBox{
            start: Vec2::new(-400.,-200.),
            density: (2,2),
            size: Vec2::new(300.,100.),
            spring_const: test_resouce.spring_constant,
            spring_damp: test_resouce.stiffness,
            mass: test_resouce.mass,
            internal_scaffold: false,
            is_static: true
        };
        
       
        test_resouce.test_bodies.push(spawn_scaffold_square(&mut commands, config));
    }
    {
        let config = ScaffoldBox{
            start: Vec2::new(100.,-150.),
            density: (2,2),
            size: Vec2::new(300.,100.),
            spring_const: test_resouce.spring_constant,
            spring_damp: test_resouce.stiffness,
            mass: test_resouce.mass,
            internal_scaffold: true,
            is_static: true
        };
        
       
        test_resouce.test_bodies.push(spawn_scaffold_square(&mut commands, config));
    }
    
   
    {
        let config = ScaffoldBox{
            start: Vec2::new(-200.,0.),
            density: (1,1),
            size: Vec2::new(400.,100.),
            spring_const: test_resouce.spring_constant,
            spring_damp: test_resouce.stiffness,
            mass: test_resouce.mass,
            internal_scaffold: true,
            is_static: false
        };
        
       
        test_resouce.test_bodies.push(spawn_scaffold_square(&mut commands, config));
    }
}

