use std::f32::consts::PI;

use bevy::{DefaultPlugins, prelude::*, sprite::{MaterialMesh2dBundle, Mesh2dHandle}, render::mesh, utils::petgraph::stable_graph::GraphIndex, input::common_conditions::input_pressed, ecs::system::Command};
use create_mode::EditPlugin;
use soft_body::SoftBodyPlugin;
mod create_mode;
mod soft_body;
mod helper;
mod test;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
       
        .add_state::<GameState>()
        .add_plugins(EditPlugin)
        .add_plugins(SoftBodyPlugin)
        .add_systems(Startup, setup)
        
        .add_systems(Update, keyboard_input)
       
        .run();
}


#[derive(Debug, States,Default,Hash,PartialEq,Eq,Clone,Copy)]
pub enum GameState{
    #[default]
    Create,
    Run
}

fn keyboard_input(
    keys: Res<Input<KeyCode>>,
    mut state: ResMut<NextState<GameState>>,
) {
    if keys.just_released(KeyCode::I) {
 
        state.set(GameState::Create);
    }
    else if keys.just_released(KeyCode::R) {
        // Left Ctrl was released
        state.set(GameState::Run);
    }
   
}
fn setup(mut commands: Commands){
    commands.spawn(Camera2dBundle::default());

}