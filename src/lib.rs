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
//pub type ConsoleResult = Result<ConsoleVal, ConsoleError>;
struct ConsoleResult(Result<ConsoleVal, ConsoleError>);

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

/*struct BaseConsole {
    root: Box<dyn cvar::IVisit + Send + Sync>,
}*/

trait NodeExt {
    fn details(&mut self, path: &str, out: &mut String);
    fn kind(&mut self) -> CmdType;
}

impl<'a> NodeExt for dyn cvar::INode + 'a{
    fn details(&mut self, path: &str, out: &mut String) {
        let desc = self.description().to_string();
        match self.as_node_mut() {
            //cvar::NodeMut::Prop(prop) => out.push_str(&format!("{}: {}\n\t{} (Default: {})\n", path, desc, prop.get(), prop.default())),
            //cvar::NodeMut::Action(_) => out.push_str(&format!("{} [*]: {}\n", path, desc)),
            cvar::NodeMut::Prop(prop) => out.push_str(&format!("{}: {} (Default: {})\n\t{}\n", path, prop.get(), prop.default(), desc)),
            cvar::NodeMut::Action(_) => {
                let (args, desc) = {
                    let mut parts = desc.split("\n");
                    let part1 = parts.next().unwrap_or("").to_string();
                    let part2 = parts.collect::<Vec<_>>().join("\n");
                    if part2.len() > 0 {
                        (part1, part2)
                    } else {
                        ("".to_string(), part1)
                    }
                };

                out.push_str(path);
                if args.len() > 0 {
                    out.push_str(&format!(" {}", args));
                }
                out.push_str(&format!(":\n\t{}\n", desc));
            }
            _ => {},
        }
    }

    fn kind(&mut self) -> CmdType {
        match self.as_node_mut() {
            cvar::NodeMut::Prop(_) => CmdType::Prop,
            cvar::NodeMut::List(_) => CmdType::List,
            cvar::NodeMut::Action(_) => CmdType::Action,
        }
    }
}

trait Console {
    fn get(&mut self, var: &str) -> ConsoleResult;
    fn set(&mut self, var: &str, val: &str) -> ConsoleResult;
    fn call(&mut self, cmd: &str, args: &[&str]) -> ConsoleResult;
    fn reset(&mut self, var: &str) -> ConsoleResult;
    fn reset_all(&mut self) -> ConsoleResult;
    fn find<F>(&mut self, filter: F) -> ConsoleResult where F: Fn(&str)->bool;
    fn help(&mut self, var: &str) -> ConsoleResult;
    fn cmdtype(&mut self, var: &str) -> CmdType;
    fn exec(&mut self, cmd: &str, args: Vec<&str>) -> ConsoleResult;
}

//impl BaseConsole {

impl<T: cvar::IVisit> Console for T {
    fn get(&mut self, var: &str) -> ConsoleResult {
        if let Some(val) = cvar::console::get(&mut *self, var) {
            ConsoleResult(Ok(val))
        } else {
            ConsoleResult(Err(ConsoleError::UnknownProperty))
        }
    }

    fn set(&mut self, var: &str, val: &str) -> ConsoleResult {
        match cvar::console::set(&mut *self, var, val) {
            Ok(success) => {
                if success {
                    ConsoleResult(Ok("".to_string()))
                } else {
                    ConsoleResult(Err(ConsoleError::UnknownProperty))
                }
            }
            Err(e) => ConsoleResult(Err(ConsoleError::InvalidValue(e.to_string())))
        }
    }

    fn call(&mut self, cmd: &str, args: &[&str]) -> ConsoleResult {
        let mut buf = String::new();
        if cvar::console::invoke(&mut *self, cmd, &args, &mut buf) {
            ConsoleResult(Ok(buf))
        } else {
            ConsoleResult(Err(ConsoleError::UnknownCommand))
        }
    }

    fn reset(&mut self, var: &str) -> ConsoleResult {
        if cvar::console::reset(&mut *self, var) {
            ConsoleResult(Ok("".to_string()))
        } else {
            ConsoleResult(Err(ConsoleError::UnknownProperty))
        }
    }

    fn reset_all(&mut self) -> ConsoleResult {
        cvar::console::reset_all(&mut *self);
        ConsoleResult(Ok("OK".to_string()))
    }

    fn find<F>(&mut self, filter: F) -> ConsoleResult where F: Fn(&str)->bool {
        let mut out = String::new();
        cvar::console::walk(&mut *self, |path, node| {
            if filter(path) {
                node.details(path, &mut out);
            }
        });

        if out.len() > 0 {
            ConsoleResult(Ok(out))
        } else {
            ConsoleResult(Err(ConsoleError::NoResults))
        }
    }

    fn help(&mut self, var: &str) -> ConsoleResult {
        let mut out = String::new();
        cvar::console::find(&mut *self, var, |node| {
            node.details(var, &mut out);
        });

        if out.len() > 0 {
            ConsoleResult(Ok(out))
        } else {
            ConsoleResult(Err(ConsoleError::UnknownProperty))
        }
    }

    fn cmdtype(&mut self, var: &str) -> CmdType {
        let mut t = CmdType::NotFound;
        cvar::console::find(&mut *self, var, |node| {
            t = node.kind();
        });
        t
    }

    fn exec(&mut self, cmd: &str, args: Vec<&str>) -> ConsoleResult {
        match self.cmdtype(cmd) {
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
                ConsoleResult(Err(ConsoleError::UnknownCommand))
            },
        }
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

impl From<ConsoleResult> for TextSpan {
    fn from(result: ConsoleResult) -> TextSpan {
        match result.0 {
            Ok(output) => output.into(),
            Err(e) => {
                TextSpan {
                    text: e.to_string(),
                    color: [1., 0., 0., 1.]
                }
            }
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
    root: Box<dyn cvar::IVisit + Send + Sync>,
    //root: BaseConsole,
    buf: Vec<TextSpan>,
    prompt: ImString,
    history: Vec<String>,
    //colors: LogColors,
}

impl ConsoleWindow {
    pub fn new(node: Box<dyn cvar::IVisit + Send + Sync>) -> Self {
        /*let mut console = BaseConsole {
            root: node
        };
        let _ = console.reset_all();*/

        let mut console = ConsoleWindow {
            //root: console,
            root: node,
            buf: vec![],
            prompt: ImString::with_capacity(100),
            history: vec![],
            //colors: LogColors::default(),
        };
        console.reset_all();
        console
    }
}

impl ConsoleWindow {
    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub fn write<S>(&mut self, text: S) where S: Into<TextSpan> {
        self.buf.push(text.into());
    }

    pub fn writeln<S>(&mut self, text: S) where S: Into<TextSpan> {
        let mut span = text.into();
        span.text = span.text.trim_end().to_string();
        if span.text.len() > 0 {
            span.text.push_str("\n");
            self.buf.push(span);
        }
    }

    /*pub fn set_colors(&mut self, colors: LogColors) {
        self.colors = colors;
    }*/

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
                self.run_cmd(self.prompt.to_string());
                self.prompt.clear();
                reclaim_focus = true;
            }

            ui.set_item_default_focus();
            if reclaim_focus {
                ui.set_keyboard_focus_here(imgui::FocusedWidget::Previous);
            }

        });
    }

    /*fn write_cmd(&self, result: ConsoleResult) {
        match result {
            Ok(output) => {
                let output = output.trim_end();
                if output.len() > 0 {
                    self.write(format!("{}\n", output);
                }just to expose the conflicting requirements error
// the right declaration is:
// impl<'a, T> Index<usize> for Stack<T> + 'a
            }
            Err(e) => {
                self.write(TextSpan {
                    text: e.to_string(),
                    color: [1., 0., 0., 1.]
                });
            }
        }
    }*/

    pub fn run_cmd(&mut self, cmd: String) {
        let mut parts = cmd.split(" "); // TODO: shellesc
        let cmd = parts.next().unwrap_or("");
        let args = parts.collect::<Vec<_>>();

        let out = self.exec(cmd, args);
        self.writeln(out);
    }


    pub fn cmd_help(&mut self, args: &[&str]) {
        let out = {
            if let Some(var) = args.get(0) {
                self.help(var)
            } else {
                self.find(|_| true)
            }
        };
        self.writeln(out);
    }

    pub fn cmd_find(&mut self, args: &[&str]) {
        let out = {
            if let Some(var) = args.get(0) {
                self.find(|path| path.contains(var) && path != "find")
            } else {
                ConsoleResult(Err(ConsoleError::InvalidUsage("find <name>".to_string())))
            }
        };
        self.writeln(out);
    }

    pub fn cmd_reset(&mut self, args: &[&str]) {
        let out = {
            if let Some(var) = args.get(0) {
                self.reset(var)
            } else {
                self.reset_all()
            }
        };
        self.writeln(out);
    }
}

impl cvar::IVisit for ConsoleWindow {
    fn visit_mut(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode)) {
        f(&mut cvar::Action("help", "\nList all commands and properties", |args, _| self.cmd_help(args)));
        f(&mut cvar::Action("clear", "\nClear the screen", |_, _| self.clear()));
        f(&mut cvar::Action("find", "<text>\nSearch for matching commands", |args, _| self.cmd_find(args)));
        f(&mut cvar::Action("reset", "<var>\nSet a property to its default", |args, _| self.cmd_reset(args)));
        self.root.visit_mut(f);
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
