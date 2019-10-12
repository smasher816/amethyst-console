pub use amethyst_imgui;

use crate::{ConsoleWindow, ConsoleConfig, TextSpan, IVisitExt};
use amethyst::ecs::System;
use imgui::im_str;
use std::fmt::Write;

/// Draws a ConsoleWindow every frame
pub struct ConsoleSystem {
    open: bool,
    console: ConsoleWindow,
}

impl ConsoleSystem {
    pub fn new(console: ConsoleWindow) -> Self {
        ConsoleSystem { open: true, console }
    }
}

impl<'s> System<'s> for ConsoleSystem {
    type SystemData = ();

    fn run(&mut self, _: Self::SystemData) {
        let open = self.open;
        amethyst_imgui::with(|ui| {
            let window = imgui::Window::new(im_str!("Console")).opened(&mut self.open);
            if open {
                self.console.build(ui, window);
            }
        });
    }
}

/// Creates a customized system that will display a comamnd console
/// This will automatically initialize the logger
pub fn create_system_with_config<T>(node: T, config: ConsoleConfig) -> ConsoleSystem
//where T: 'static + cvar::IVisit + Send  + Sync {
where T: 'static + IVisitExt + Send  + Sync {
    let mut console_window = crate::init_with_config(node, config);
    console_window.console.write_str("Type '");
    console_window.write(TextSpan {
        text: "HELP".to_string(),
        color: [1., 0., 0., 1.],
    });
    console_window.console.write_str("' for help, press ");
    console_window.write(TextSpan {
        text: "TAB".to_string(),
        color: [1., 1., 0., 1.],
    });
    console_window.console.write_str(" to use text completion.\n");
    ConsoleSystem::new(console_window)
}

/// Creates a system that will display your logs every frame.
/// This will automatically initialize the logger
pub fn create_system<T>(node: T) -> ConsoleSystem
//where T: 'static + cvar::IVisit + Send  + Sync {
where T: 'static + IVisitExt + Send  + Sync {
    create_system_with_config(node, ConsoleConfig::default())
}
