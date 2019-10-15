/// Constructs a simple ExampleSystem and GameConfig.
/// You can continue to add more config options by implementing IVisitExt for each struct.
///
/// `cargo run --example demo_console --features amethyst-system`
///
/// Type `help` see available commands.
///
/// Examples:
///  * `width 120` - Set width to 120
///  * `width` - Print the current width
///  * `reset width` - Reset width to its default value (100)
///  * `find a` - Find all commands with `a` in their name
///  * `reset` - Reset all variables to their defaults
use amethyst::{
    ecs::{Read, System},
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{bundle::RenderingBundle, types::DefaultBackend, RenderToWindow},
    utils::application_root_dir,
};

use amethyst_console::{amethyst_imgui::RenderImgui, IConsoleExt, IVisitExt};

pub struct ArenaConfig {
    pub height: f32,
    pub width: f32,
}

// NOTE: Your structs must implement the default trait, and be Send+Sync capable
impl Default for ArenaConfig {
    fn default() -> Self {
        ArenaConfig {
            height: 100.0,
            width: 100.0,
        }
    }
}
impl IVisitExt for ArenaConfig {
    // visit_mut_ext will be called any time a command is typed in order see what
    // properties/actions are available.
    fn visit_mut_ext(
        &mut self,
        f: &mut dyn FnMut(&mut dyn cvar::INode),
        _console: &mut dyn IConsoleExt,
    ) {
        let default = Self::default();
        f(&mut cvar::Property(
            "width",
            "Arena width",
            &mut self.width,
            default.width,
        ));
        f(&mut cvar::Property(
            "height",
            "Arena height",
            &mut self.height,
            default.height,
        ));
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
    fn visit_mut_ext(
        &mut self,
        f: &mut dyn FnMut(&mut dyn cvar::INode),
        _console: &mut dyn IConsoleExt,
    ) {
        let default = Self::default();
        f(&mut cvar::Property(
            "velocity",
            "paddle velocity",
            &mut self.velocity,
            default.velocity,
        ));
        f(&mut cvar::Property(
            "color",
            "paddle color",
            &mut self.color,
            default.color,
        ));
    }
}

// Commands do not have to be part of a struct
pub fn color_test(console: &mut dyn IConsoleExt) {
    for r in (0..=2).rev() {
        for g in (0..=2).rev() {
            for b in (0..=2).rev() {
                console.write_colored(
                    [(r as f32) / 2., (g as f32) / 2., (b as f32) / 2., 1.],
                    " ■",
                );
            }
        }
    }
    console.write("\n");
}

#[derive(Default)]
pub struct GameConfig {
    pub arena: ArenaConfig,
    pub paddle: PaddleConfig,
}
impl IVisitExt for GameConfig {
    fn visit_mut_ext(
        &mut self,
        f: &mut dyn FnMut(&mut dyn cvar::INode),
        console: &mut dyn IConsoleExt,
    ) {
        f(&mut cvar::Action(
            "color_test",
            "Test console colors",
            |_, _| color_test(console),
        ));

        // Calls to children will add their entries the the available command list.
        // This allows for deeply nested data structures.
        self.arena.visit_mut_ext(f, console);
        self.paddle.visit_mut_ext(f, console);
    }
}

#[derive(Default)]
pub struct ExampleSystem {}

impl<'s> System<'s> for ExampleSystem {
    type SystemData = (
        // Use Read to grab the resource from the World
        // ConsoleSystem will automatically keep it up to date as commands are ran.
        Read<'s, GameConfig>,
    );

    fn run(&mut self, (game_config,): Self::SystemData) {
        let arena_config = &game_config.arena;

        // This print statement will change the moment a user types a set command,
        // providing instant feedback that the command worked
        println!("{} x {}", arena_config.width, arena_config.height);
    }
}

struct Example;
impl SimpleState for Example {}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(amethyst::LoggerConfig::default());
    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("examples/display.ron");

    // Construct a console system, specifying the type of resource you would like to update.
    // This will:
    //    - Initialize the struct to its default value
    //    - Add it to the world so other services can read in their run loops
    //    - Create a console window with everything added by `visit_mut_ext`
    let console_system = amethyst_console::create_system::<GameConfig>();

    let game_data = GameDataBuilder::default()
        .with_barrier()
        .with_system_desc(console_system, "imgui_console", &[]) // <--- ADDED
        .with(ExampleSystem::default(), "example", &[])
        .with_bundle(
            InputBundle::<StringBindings>::new().with_bindings_from_file("examples/input.ron")?,
        )?
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
