/// Uses the amethyst-system feature of igmui_log to display all console output
///
/// `cargo run --example demo_log --features amethyst-system`
///

use amethyst::{
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{bundle::RenderingBundle, types::DefaultBackend, RenderToWindow},
    utils::application_root_dir,
    ecs::{Read, System}
};

use imgui_console::{amethyst_imgui::RenderImgui, IVisitExt, IConsoleExt};

pub struct ArenaConfig {
    pub height: f32,
    pub width: f32,
}
impl Default for ArenaConfig {
    fn default() -> Self {
        ArenaConfig {
            height: 100.0,
            width: 100.0,
        }
    }
}
impl IVisitExt for ArenaConfig {
    fn visit_mut_ext(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode), _console: &mut dyn IConsoleExt) {
        let default = Self::default();
        f(&mut cvar::Property("width", "Arena width", &mut self.width, default.width));
        f(&mut cvar::Property("height", "Arena height", &mut self.height, default.height));
    }
}

pub struct PaddleConfig {
    pub velocity: f32,
    pub color: String,
}
impl Default for PaddleConfig {
    fn default() -> Self {
        PaddleConfig {
            velocity: 3.0,
            color: "white".to_string(),
        }
    }
}
impl IVisitExt for PaddleConfig {
    fn visit_mut_ext(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode), _console: &mut dyn IConsoleExt) {
        let default = Self::default();
        f(&mut cvar::Property("velocity", "paddle velocity", &mut self.velocity, default.velocity));
        f(&mut cvar::Property("color", "paddle color", &mut self.color, default.color));
    }
}

pub fn color_test(console: &mut dyn IConsoleExt) {
    for r in (0..=2).rev() {
        for g in (0..=2).rev() {
            for b in (0..=2).rev() {
                console.write_colored([(r as f32) / 2., (g as f32) / 2., (b as f32) / 2., 1.], " ■");
            }
        }
    }
}

#[derive(Default)]
pub struct GameConfig {
    pub arena: ArenaConfig,
    pub paddle: PaddleConfig,
}
impl IVisitExt for GameConfig {
    fn visit_mut_ext(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode), console: &mut dyn IConsoleExt) {
        self.arena.visit_mut_ext(f, console);
        self.paddle.visit_mut_ext(f, console);
        f(&mut cvar::Action("color_test", "Test console colors", |_, _| color_test(console)));
    }
}


#[derive(Default)]
pub struct ExampleSystem {}

impl<'s> System<'s> for ExampleSystem {
    type SystemData = (
            Read<'s, GameConfig>,
        );

    fn run(&mut self, (game_config, ): Self::SystemData) {
        let arena_config = &game_config.arena;
        println!("{} x {}", arena_config.width, arena_config.height);
    }
}


struct Example;
impl SimpleState for Example {}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(amethyst::LoggerConfig::default());
    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("examples/display.ron");

    let console_system = imgui_console::create_system::<GameConfig>();

    let game_data = GameDataBuilder::default()
        .with_barrier()
        .with_system_desc(console_system, "imgui_console", &[]) // <--- ADDED
        .with(ExampleSystem::default(), "example", &[])
        .with_bundle(InputBundle::<StringBindings>::new()
                     .with_bindings_from_file("examples/input.ron")?)?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderImgui::<StringBindings>::default()), // <--- ADDED
        )?;

    Application::build("/", Example)?.build(game_data)?.run();

    Ok(())
}
