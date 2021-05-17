use bevy::prelude::*;

mod diagnostics;
use diagnostics::DiagnosticsPlugin;

mod game;
use game::GamePlugin;

pub fn run() {
    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .query_selector("#game")
        .unwrap()
        .unwrap();

    let mut builder = App::build();
    let mut win_desc = WindowDescriptor {
        resizable: true,
        width: canvas.client_width() as f32,
        height: canvas.client_height() as f32,
        cursor_locked: true,
        ..Default::default()
    };

    #[cfg(target_arch = "wasm32")]
    {
        win_desc.canvas = Some("#game".to_string());
    }

    builder
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(win_desc)
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .add_plugin(DiagnosticsPlugin);

    #[cfg(target_arch = "wasm32")]
    {
        builder.add_plugin(bevy_webgl2::WebGL2Plugin);
    }

    builder.run();
}
