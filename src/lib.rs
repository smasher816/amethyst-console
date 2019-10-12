#[cfg(feature = "amethyst-system")]
mod amethyst;

#[cfg(feature = "amethyst-system")]
pub use crate::amethyst::*;

use imgui::{ImString, im_str};

#[derive(Debug)]
pub enum CmdType {
    Prop, List, Action, NotFound
}

#[derive(Debug)]
pub enum ConsoleError {
    UnknownProperty,
    UnknownCommand,
    InvalidValue(String),
    InvalidUsage(String),
    NoResults,
}

pub type ConsoleVal = String;
pub type ConsoleDesc = String;
pub type ConsoleResult = Result<ConsoleVal, ConsoleError>;

impl std::fmt::Display for ConsoleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsoleError::UnknownProperty => f.write_str("Unknown property"),
            ConsoleError::UnknownCommand => f.write_str("Unknown command"),
            ConsoleError::InvalidValue(e) => write!(f, "Invalid value: {}", e),
            ConsoleError::InvalidUsage(e) => write!(f, "Usage: {}", e),
            ConsoleError::NoResults => f.write_str("No results"),
        }
    }
}

struct BaseConsole {
    root: Box<dyn cvar::IVisit + Send + Sync>,
}

impl BaseConsole {
    fn write(&self, console: &mut dyn cvar::IConsole, result: ConsoleResult) {
        let output = match result {
            Ok(text) => text,
            Err(e) => e.to_string(),
        };

        let output = output.trim_end();
        if output.len() > 0 {
            let _ = writeln!(console, "{}", output);
        }
    }

    fn details(path: &str, node: &mut dyn cvar::INode, out: &mut String) {
        let desc = node.description().to_string();
        match node.as_node_mut() {
            cvar::NodeMut::Prop(prop) => out.push_str(&format!("{}: {}\n\t{} (Default: {})\n", path, desc, prop.get(), prop.default())),
            cvar::NodeMut::Action(_) => out.push_str(&format!("{} [*]: {}\n", path, desc)),
            _ => {},
        }
    }

    pub fn get(&mut self, var: &str) -> ConsoleResult {
        if let Some(val) = cvar::console::get(&mut *self, var) {
            Ok(val)
        } else {
            Err(ConsoleError::UnknownProperty)
        }
    }

    pub fn set(&mut self, var: &str, val: &str) -> ConsoleResult {
        match cvar::console::set(&mut *self, var, val) {
            Ok(success) => {
                if success {
                    Ok("".to_string())
                } else {
                    Err(ConsoleError::UnknownProperty)
                }
            }
            Err(e) => Err(ConsoleError::InvalidValue(e.to_string()))
        }
    }

    pub fn call(&mut self, cmd: &str, args: &[&str]) -> ConsoleResult {
        let mut buf = String::new();
        if cvar::console::invoke(&mut *self, cmd, &args, &mut buf) {
            Ok(buf)
        } else {
            Err(ConsoleError::UnknownCommand)
        }
    }

    pub fn reset(&mut self, var: &str) -> ConsoleResult {
        if cvar::console::reset(&mut *self, var) {
            Ok("".to_string())
        } else {
            Err(ConsoleError::UnknownProperty)
        }
    }

    pub fn reset_all(&mut self) -> ConsoleResult {
        cvar::console::reset_all(&mut *self);
        Ok("OK".to_string())
    }

    pub fn find<F>(&mut self, filter: F) -> ConsoleResult where F: Fn(&str)->bool {
        let mut out = String::new();
        cvar::console::walk(&mut *self, |path, node| {
            if filter(path) {
                BaseConsole::details(path, node, &mut out);
            }
        });

        if out.len() > 0 {
            Ok(out)
        } else {
            Err(ConsoleError::NoResults)
        }
    }

    pub fn help(&mut self, var: &str) -> ConsoleResult {
        let mut out = String::new();
        cvar::console::find(&mut *self, var, |node| {
            BaseConsole::details(var, node, &mut out);
        });

        if out.len() > 0 {
            Ok(out)
        } else {
            Err(ConsoleError::UnknownProperty)
        }
    }

    pub fn cmdtype(&mut self, var: &str) -> CmdType {
        let mut t = CmdType::NotFound;
        cvar::console::find(&mut *self, var, |node| {
            t = match node.as_node_mut() {
                cvar::NodeMut::Prop(_) => CmdType::Prop,
                cvar::NodeMut::List(_) => CmdType::List,
                cvar::NodeMut::Action(_) => CmdType::Action,
            };
        });
        t
    }

    pub fn exec(&mut self, console: &mut dyn cvar::IConsole, cmd: &str, args: Vec<&str>) {
        let out = match self.cmdtype(cmd) {
            CmdType::Prop => {
                if let Some(val) = args.get(0) {
                    self.set(cmd, val)
                } else {
                    self.get(cmd)
                }
            },
            CmdType::Action => self.call(cmd, &args),
            CmdType::List => self.find(|path| path.starts_with(cmd)),
            CmdType::NotFound => {
                Err(ConsoleError::UnknownCommand)
            },
        };
        self.write(console, out);
    }

    pub fn cmd_help(&mut self, console: &mut dyn cvar::IConsole, args: &[&str]) {
        let out = {
            if let Some(var) = args.get(0) {
                self.help(var)
            } else {
                self.find(|_| true)
            }
        };
        self.write(console, out);
    }

    pub fn cmd_find(&mut self, console: &mut dyn cvar::IConsole, args: &[&str]) {
        let out = {
            if let Some(var) = args.get(0) {
                self.find(|path| path.contains(var) && path != "find")
            } else {
                Err(ConsoleError::InvalidUsage("find <name>".to_string()))
            }
        };
        self.write(console, out);
    }

    pub fn cmd_reset(&mut self, console: &mut dyn cvar::IConsole, args: &[&str]) {
        let out = {
            if let Some(var) = args.get(0) {
                self.reset(var)
            } else {
                self.reset_all()
            }
        };
        self.write(console, out);
    }
}

impl cvar::IVisit for BaseConsole {
    fn visit_mut(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode)) {
        f(&mut cvar::Action("help", "List all commands/variables", |args, console| self.cmd_help(console, args)));
        f(&mut cvar::Action("find", "find <text>", |args, console| self.cmd_find(console, args)));
        f(&mut cvar::Action("reset", "find <var>", |args, console| self.cmd_reset(console, args)));
        self.root.visit_mut(f);
    }
}

#[derive(Debug)]
pub struct TextSpan {
    color: [f32; 4],
    text: String,
}

impl<T> From<T> for TextSpan where T: Into<String> {
    fn from(t: T) -> TextSpan {
        TextSpan {
            color: [1., 1., 1., 1.],
            text: t.into(),
        }
    }
}

impl std::fmt::Display for TextSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

/// The imgui frontend for cvars
/// Call `build` during your rendering stage
pub struct ConsoleWindow {
    root: BaseConsole, //Box<dyn cvar::IVisit + Send + Sync>,
    buf: Vec<TextSpan>,
    prompt: ImString,
    history: Vec<String>,
    //colors: LogColors,
}

impl ConsoleWindow {
    pub fn new(node: Box<dyn cvar::IVisit + Send + Sync>) -> Self {
        let mut console = BaseConsole {
            root: node
        };
        let _ = console.reset_all();

        ConsoleWindow {
            root: console,
            buf: vec![],
            prompt: ImString::with_capacity(100),
            history: vec![],
            //colors: LogColors::default(),
        }
    }
}

impl ConsoleWindow {
    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub fn write<S>(&mut self, text: S) where S: Into<TextSpan> {
        self.buf.push(text.into());
    }

    /*pub fn set_colors(&mut self, colors: LogColors) {
        self.colors = colors;
    }*/

    pub fn exec(&mut self, cmd: String) {
        let mut parts = cmd.split(" "); // TODO: shellesc
        let cmd = parts.next().unwrap_or("");
        let args = parts.collect::<Vec<_>>();

        let mut out = String::new();
        self.root.exec(&mut out, cmd, args);
        self.write(out);
    }

    pub fn draw_prompt(&mut self) {
        self.write(TextSpan {
            text: " > ".to_string(),
            color: [0., 1., 1., 1.]
        });
    }

    pub fn build(&mut self, ui: &imgui::Ui, window: imgui::Window) {
        window.size([520., 600.], imgui::Condition::FirstUseEver)
        .build(ui, move || {
            if ui.is_item_hovered() {
                ui.popup(im_str!("context_menu"), || {
                    if imgui::MenuItem::new(im_str!("Close")).build(ui) {
                        //*open = false;
                    }
                })
            }


            let clear = ui.button(im_str!("Clear"), [0., 0.]);
            ui.same_line(0.);
            let copy = ui.button(im_str!("Copy"), [0., 0.]);
            ui.separator();

            let footer_height_to_reserve = 1.5 * ui.frame_height_with_spacing();
            let child = imgui::ChildWindow::new(imgui::Id::Str("scrolling"))
                .size([0., -footer_height_to_reserve])
                .horizontal_scrollbar(true);
            child.build(ui, || {
                if clear {
                    self.clear();
                }
                let buf = &mut self.buf;
                if copy {
                    ui.set_clipboard_text(&ImString::new(
                        buf.iter()
                            .map(|l| l.to_string())
                            .collect::<Vec<String>>()
                            .join("\n"),
                    ));
                }

                let style = ui.push_style_var(imgui::StyleVar::ItemSpacing([0., 0.]));

                for span in buf {
                    /*if span.text.contains("\r") {
                        let pos = ui.cursor_pos();
                        ui.set_cursor_pos([0., pos[1]]);
                    }*/
                    ui.text_colored(span.color, &span.text);
                    if !span.text.contains("\n") {
                        ui.same_line(0.);
                        //ui.new_line();
                    }
                }

                style.pop(ui);

                if ui.scroll_y() >= ui.scroll_max_y() {
                    ui.set_scroll_here_y_with_ratio(1.0);
                }
            });

            ui.separator();
            let mut reclaim_focus = false;
            let input = imgui::InputText::new(ui, im_str!("cmd"), &mut self.prompt)
                .enter_returns_true(true)
                //.callback_completion(true)
                //.callback_history(true)
                .build();
            if input {
                self.draw_prompt();
                self.write(format!("{}\n", self.prompt));
                self.exec(self.prompt.to_string());
                self.prompt.clear();
                reclaim_focus = true;
            }

            ui.set_item_default_focus();
            if reclaim_focus {
                ui.set_keyboard_focus_here(imgui::FocusedWidget::Previous);
            }

        });
    }
}

/// ConsoleWindow builder
///
/// Use `ConsoleConfig::default()` to intialize.
///
/// Call `.build()` to finalize.
pub struct ConsoleConfig {
    //colors: Option<LogColors>,
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        ConsoleConfig {
            //colors: None,
        }
    }
}

impl ConsoleConfig {
    /*pub fn colors(mut self, colors: LogColors) -> Self {
        self.colors = Some(colors);
        self
    }*/

    pub fn build(self, node: Box<dyn cvar::IVisit + Send  + Sync>) -> ConsoleWindow {
        ConsoleWindow::new(node)
    }
}

/// Create a window and initialize the console window.
/// Be sure to call build on the returned window during your rendering stage
pub fn init_with_config<T>(node: T, config: ConsoleConfig) -> ConsoleWindow 
where T: 'static + cvar::IVisit + Send  + Sync {
    //let mut window = LogWindow::new(log_reader);
    /*if let Some(colors) = config.colors {
        window.set_colors(colors);
    }*/

    config.build(Box::new(node))
}

/// Create a window and initialize the console window with the default config.
/// Be sure to call build on the returned window during your rendering stage
pub fn init<T>(node: T) -> ConsoleWindow
where T: 'static + cvar::IVisit + Send  + Sync {
    init_with_config(node, ConsoleConfig::default())
}
