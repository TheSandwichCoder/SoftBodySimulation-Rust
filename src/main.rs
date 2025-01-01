use bevy::{
    color::palettes::css::GOLD,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use settings::*;
use soft_body::*;

// CRATES
mod settings;
mod soft_body;

#[derive(Component)]
struct FpsText;


fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
            primary_window: Some(Window{
                title: "Ball Simulation".into(),
                resolution: (SCREENSIZE.x, SCREENSIZE.y).into(),
                resizable:false,
                ..default()
            }),
            ..default()
        })
        .build(),
        FrameTimeDiagnosticsPlugin
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, text_update_system)
    .add_plugins(SBPlugin)
    .run();
}

#[derive(Asset, TypePath, Default, Debug, Clone)]
struct LineMaterial {
    color: LinearRgba,
}

#[derive(Debug, Clone)]
struct LineList {
    lines: Vec<(Vec3, Vec3)>,
}


fn setup(mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
){

    let camera = Camera2dBundle::default();
    commands.spawn(camera);    

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font_size: 30.0,
                    ..default()
                },
            ),
            TextSection::from_style(
                TextStyle {
                    font_size: 30.0,
                    color: GOLD.into(),
                    ..default()
                }
            ),
        ]),
        FpsText,
    ));
}


fn text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut fps_text : Query<&mut Text, With<FpsText>>,
    
) {
    let mut fps_text = fps_text.single_mut();

    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            // Update the value of the second section
            fps_text.sections[1].value = format!("{:.2}", value);
        }
    }

}
