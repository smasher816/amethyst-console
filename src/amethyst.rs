pub use amethyst_imgui;

use crate::{ConsoleWindow, IVisitExt};
use amethyst::{
    core::{
        SystemDesc,
        shrev::{EventChannel, ReaderId}
    },
    prelude::*,
    input::{InputEvent, StringBindings},
    ecs::{Read, System}
};
use imgui::im_str;

////////
///
/// TODO:
///   Try storing syste configs in world
///     implement IVisitExt wrapper for this
///     I don't think we can box+move a system into a node and have amethyst still own it
///     pass in root dynamically?
///
////////

/// Draws a ConsoleWindow every frame
pub struct ConsoleSystem {
    open: bool,
    console: ConsoleWindow,
    root: Box<dyn IVisitExt + Send + Sync>,
    event_reader: Option<ReaderId<InputEvent<StringBindings>>>,
}

impl ConsoleSystem {
    pub fn new(console: ConsoleWindow, root: Box<dyn IVisitExt + Send + Sync>) -> Self {
        //root.reset_all();
        ConsoleSystem { open: true, console, root, event_reader: None }
    }
}

impl<'a, 'b> SystemDesc<'a, 'b, ConsoleSystem> for ConsoleSystem {
    fn build(self, world: &mut World) -> ConsoleSystem {
        world.setup::<Read<EventChannel<InputEvent<StringBindings>>>>();
        let event_reader = world.fetch_mut::<EventChannel<InputEvent<StringBindings>>>().register_reader();
        ConsoleSystem { open: self.open, console: self.console, root: self.root, event_reader: Some(event_reader) }
    }
}

impl<'s> System<'s> for ConsoleSystem {
    type SystemData = (
            Read<'s, EventChannel<InputEvent<StringBindings>>>,
        );

    fn run(&mut self, (events, ): Self::SystemData) {
        if let Some(reader) = &mut self.event_reader {
            for event in events.read(reader) {
                if let InputEvent::ActionPressed(s) = event {
                    match s.as_ref() {
                        "toggle_console" => {
                            self.open = !self.open;
                        },
                        _ => {},
                    }
                }
            }
        }

        let open = self.open;
        amethyst_imgui::with(|ui| {
            let window = imgui::Window::new(im_str!("Console")).opened(&mut self.open);
            if open {
                self.console.build(ui, window, &mut *self.root);
            }
        });
    }
}

//fn init_system(mut console_window: ConsoleWindow) -> ConsoleSystem {
pub fn init_system<T>(mut console_window: ConsoleWindow, node: T) -> ConsoleSystem
where T: 'static + IVisitExt + Send  + Sync {
    console_window.write("Type '");
    console_window.write_colored([1., 0., 0., 1.], "HELP");
    console_window.write("' for help, press ");
    console_window.write_colored([1., 1., 0., 1.], "TAB");
    console_window.write(" to use text completion.\n");
    //ConsoleSystem::new(console_window)
    ConsoleSystem::new(console_window, Box::new(node))
}

/// Creates a system that will display your logs every frame.
/// This will automatically initialize the logger
pub fn create_system<T>(node: T) -> ConsoleSystem
where T: 'static + IVisitExt + Send  + Sync {
    let console_window = crate::init();
    init_system(console_window, node)
}
