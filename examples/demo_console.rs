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

use imgui_console::{ConsoleConfig, amethyst_imgui::RenderImgui, ConsoleError, ConsoleResult, IVisitExt, IConsoleExt, TextSpan};

#[derive(Default)]
pub struct User {
    pub name: String,
    pub age: u32,
}
impl User {
    //pub fn greet(&self, console: &mut dyn cvar::IConsole) {
    pub fn greet(&self, console: &mut dyn IConsoleExt) {
        let _ = write!(console, "Hello, {}!", self.name);
        console.write_str("Test\n");
        console.write_error(&ConsoleError::InvalidUsage("Some error\n".to_string()));
        console.write_result(ConsoleResult(Err(ConsoleError::InvalidUsage("Woo".to_string()))));
        console.write_colored([0., 1., 0., 1.], "IM GREEN\n");
    }
}

//impl cvar::IVisit for User {
impl IVisitExt for User {
    //fn visit_mut(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode)) {
    fn visit_mut(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode), console: &mut dyn IConsoleExt) {
        f(&mut cvar::Property("name", "Persons name", &mut self.name, "<Unknown>".to_string()));
        f(&mut cvar::Property("age", "Persons age", &mut self.age, 0));
        f(&mut cvar::Action("greet", "Say hi", |_args, _| self.greet(console)));
    }
}

/*impl cvar::IVisit for User {
    fn visit_mut(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode)) {
        println!("VISIT MUT");
        //let mut buf = String::new();
        //self.visit_mut2(f, &mut buf)
    }
}*/

struct Example;
impl SimpleState for Example {}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(amethyst::LoggerConfig::default());
    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("examples/display.ron");

    /*let mut user = User {
        name: String::new(),
        age: 0,
    };*/

    // Give the user a name
    //cvar::console::set(&mut user, "name", "World").unwrap();
    //assert_eq!(user.name, "World");

    // Greet the user, the message is printed to the console string
    //let mut console = String::new();
    //cvar::console::invoke(&mut user, "greet", &[""], &mut console);
    //assert_eq!(console, "Hello, World!");

    let game_data = GameDataBuilder::default()
        .with_barrier()
        .with(imgui_console::create_system(User::default()), "imgui_console", &[]) // <--- ADDED
        .with_bundle(InputBundle::<StringBindings>::default())?
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
