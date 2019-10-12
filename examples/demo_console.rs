/// Uses the amethyst-system feature of igmui_log to display all console output
///
/// `cargo run --example demo_log --features amethyst-system`
///

use amethyst::{
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{bundle::RenderingBundle, types::DefaultBackend, RenderToWindow},
    utils::application_root_dir,
};

use imgui_console::{amethyst_imgui::RenderImgui, ConsoleError, ConsoleResult, IVisitExt, IConsoleExt, VisitMutExt};

#[derive(Default)]
pub struct User {
    pub name: String,
    pub age: u32,
}
impl User {
    pub fn greet(&self, console: &mut dyn IConsoleExt) {
        let _ = writeln!(console, "Hello, {}!", self.name);
        console.write("Test\n");
        console.write_error(&ConsoleError::InvalidUsage("Some error\n".to_string()));
        console.write_result(ConsoleResult(Err(ConsoleError::InvalidUsage("Woo".to_string()))));
        console.write_colored([0., 1., 0., 1.], "IM GREEN\n");
    }
}

impl IVisitExt for User {
    fn visit_mut_ext(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode), console: &mut dyn IConsoleExt) {
        f(&mut cvar::Property("name", "Persons name", &mut self.name, "<Unknown>".to_string()));
        f(&mut cvar::Property("age", "Persons age", &mut self.age, 0));
        f(&mut cvar::Action("greet", "Say hi", |_args, _| self.greet(console)));
    }
}

pub struct Foobar {
}

impl Foobar {
    pub fn colors(&self, console: &mut dyn IConsoleExt) {
        console.write_colored([1., 0., 0., 1.], "RED ");
        console.write_colored([0., 1., 0., 1.], "GREEN ");
        console.write_colored([0., 0., 1., 1.], "BLUE\n");
    }
}

impl IVisitExt for Foobar {
    fn visit_mut_ext(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode), console: &mut dyn IConsoleExt) {
        f(&mut cvar::Action("colors", "Test colors", |_, _| self.colors(console)));
    }
}

pub fn red(console: &mut dyn IConsoleExt) {
    console.write_colored([1., 0., 0., 1.], "RED\n");
}

pub fn green(console: &mut dyn IConsoleExt) {
    console.write_colored([0., 1., 0., 1.], "GREEN\n");
}

pub fn blue(console: &mut dyn IConsoleExt) {
    console.write_colored([0., 0., 1., 1.], "BLUE\n");
}

struct Example;
impl SimpleState for Example {}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(amethyst::LoggerConfig::default());
    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("examples/display.ron");

    let mut user = User::default();
    let mut foobar = Foobar { };
    let root = VisitMutExt(move |f, console| {
        f(&mut cvar::Action("red", "Test red", |_, _| red(console)));
        f(&mut cvar::Action("green", "Test green", |_, _| green(console)));
        f(&mut cvar::Action("blue", "Test blue", |_, _| blue(console)));
        foobar.visit_mut_ext(f, console);
        user.visit_mut_ext(f, console);
    });

    let console_system = imgui_console::create_system(root);

    let game_data = GameDataBuilder::default()
        .with_barrier()
        .with_system_desc(console_system, "imgui_console", &[]) // <--- ADDED
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
