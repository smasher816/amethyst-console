pub use amethyst_imgui;

use std::marker::PhantomData;
use crate::{ConsoleWindow, IVisitExt, VisitMutExt};
use amethyst::{
    core::{
        SystemDesc,
        shrev::{EventChannel, ReaderId}
    },
    prelude::*,
    input::{InputEvent, StringBindings},
    ecs::{Read, Write, System}
};
use imgui::im_str;

/// Draws a ConsoleWindow every frame
pub struct ConsoleSystem<T> {
    open: bool,
    console: ConsoleWindow,
    event_reader: Option<ReaderId<InputEvent<StringBindings>>>,
    _marker: PhantomData<T>,
}

impl<T> ConsoleSystem<T> {
    pub fn new(console: ConsoleWindow) -> ConsoleSystem<T> {
        ConsoleSystem { open: true, console, event_reader: None, _marker: PhantomData }
    }
}

impl<'a, 'b, T> SystemDesc<'a, 'b, ConsoleSystem<T>> for ConsoleSystem<T>
where T: 'static + std::marker::Send + std::marker::Sync + std::default::Default + IVisitExt {
    fn build(self, world: &mut World) -> ConsoleSystem<T> {
        world.insert(T::default());
        world.setup::<Read<EventChannel<InputEvent<StringBindings>>>>();
        let event_reader = world.fetch_mut::<EventChannel<InputEvent<StringBindings>>>().register_reader();
        ConsoleSystem { open: self.open, console: self.console, event_reader: Some(event_reader), _marker: PhantomData }
    }
}

impl<'s, T> System<'s> for ConsoleSystem<T>
where T: 'static + std::marker::Send + std::marker::Sync + std::default::Default + IVisitExt {
    type SystemData = (
            Read<'s, EventChannel<InputEvent<StringBindings>>>,
            Write<'s, T>,
        );

    fn run(&mut self, (events, mut config): Self::SystemData) {
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

        let mut root = VisitMutExt(move |f, console| {
            config.visit_mut_ext(f, console);
        });

        let open = self.open;
        amethyst_imgui::with(|ui| {
            let window = imgui::Window::new(im_str!("Console")).opened(&mut self.open);
            if open {
                let console = &mut self.console;
                console.build(ui, window, &mut root);
            }
        });
    }
}

fn init_system<T>(mut console_window: ConsoleWindow) -> ConsoleSystem<T> {
    console_window.write("Type '");
    console_window.write_colored([1., 0., 0., 1.], "HELP");
    console_window.write("' for help, press ");
    console_window.write_colored([1., 1., 0., 1.], "TAB");
    console_window.write(" to use text completion.\n");
    ConsoleSystem::new(console_window)
}

/// Creates a system that will display your logs every frame.
/// This will automatically initialize the logger
pub fn create_system<T>() -> ConsoleSystem<T> {
    let console_window = crate::init();
    init_system(console_window)
}
