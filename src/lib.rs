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
    Unimplemented,
    Custom(String),
}

pub struct ConsoleResult(pub Result<String, ConsoleError>);

impl From<Result<String, ConsoleError>> for ConsoleResult {
    fn from(result: Result<String, ConsoleError>) -> ConsoleResult {
        ConsoleResult(result)
    }
}

impl From<String> for ConsoleResult {
    fn from(v: String) -> ConsoleResult {
        ConsoleResult(Ok(v))
    }
}

impl From<&str> for ConsoleResult {
    fn from(v: &str) -> ConsoleResult {
        ConsoleResult(Ok(v.to_string()))
    }
}

impl From<ConsoleError> for ConsoleResult {
    fn from(e: ConsoleError) -> ConsoleResult {
        ConsoleResult(Err(e))
    }
}

impl std::ops::Deref for ConsoleResult {
    type Target = Result<String, ConsoleError>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for ConsoleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsoleError::UnknownProperty => f.write_str("Unknown property"),
            ConsoleError::UnknownCommand => f.write_str("Unknown command"),
            ConsoleError::InvalidValue(e) => write!(f, "Invalid value: {}", e),
            ConsoleError::InvalidUsage(e) => write!(f, "Usage: {}", e),
            ConsoleError::NoResults => f.write_str("No results"),
            ConsoleError::Unimplemented => f.write_str("Unimplemented"),
            ConsoleError::Custom(e) => f.write_str(e),
        }
    }
}

impl std::error::Error for ConsoleError {}

pub trait NodeExt {
    fn details(&mut self, path: &str, out: &mut String);
    fn kind(&mut self) -> CmdType;
}

impl<'a> NodeExt for dyn cvar::INode + 'a{
    fn details(&mut self, path: &str, out: &mut String) {
        let desc = self.description().to_string();
        match self.as_node_mut() {
            cvar::NodeMut::Prop(prop) => out.push_str(&format!("{}: {} (Default: {})\n\t{}\n", path, prop.get(), prop.default(), desc)),
            cvar::NodeMut::Action(_) => {
                let (args, desc) = {
                    let mut parts = desc.split('\n');
                    let part1 = parts.next().unwrap_or("").to_string();
                    let part2 = parts.collect::<Vec<_>>().join("\n");
                    if !part2.is_empty() {
                        (part1, part2)
                    } else {
                        ("".to_string(), part1)
                    }
                };

                out.push_str(path);
                if !args.is_empty() {
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

pub trait Console {
    fn get(&mut self, var: &str) -> ConsoleResult;
    fn set(&mut self, var: &str, val: &str) -> ConsoleResult;
    fn call(&mut self, cmd: &str, args: &[&str], console: &mut dyn cvar::IConsole) -> ConsoleResult;
    fn reset(&mut self, var: &str) -> ConsoleResult;
    fn reset_all(&mut self) -> ConsoleResult;
    fn find<F>(&mut self, filter: F) -> ConsoleResult where F: Fn(&str)->bool;
    fn help(&mut self, var: &str) -> ConsoleResult;
    fn cmdtype(&mut self, var: &str) -> CmdType;
    fn exec(&mut self, cmd: &str, args: Vec<&str>, console: &mut ColoredConsole);
}

impl<T: cvar::IVisit> Console for T {
    fn get(&mut self, var: &str) -> ConsoleResult {
        if let Some(val) = cvar::console::get(&mut *self, var) {
            val.into()
        } else {
            ConsoleError::UnknownProperty.into()
        }
    }

    fn set(&mut self, var: &str, val: &str) -> ConsoleResult {
        match cvar::console::set(&mut *self, var, val) {
            Ok(success) => {
                if success {
                    Ok("".to_string())
                } else {
                    Err(ConsoleError::UnknownProperty)
                }
            }
            Err(e) => Err(ConsoleError::InvalidValue(e.to_string()))
        }.into()
    }

    fn call(&mut self, cmd: &str, args: &[&str], console: &mut dyn cvar::IConsole) -> ConsoleResult {
        if cvar::console::invoke(&mut *self, cmd, &args, console) {
            "".into()
        } else {
            ConsoleError::UnknownCommand.into()
        }
    }

    fn reset(&mut self, var: &str) -> ConsoleResult {
        if cvar::console::reset(&mut *self, var) {
            "".into()
        } else {
            ConsoleError::UnknownProperty.into()
        }
    }

    fn reset_all(&mut self) -> ConsoleResult {
        cvar::console::reset_all(&mut *self);
        "OK".into()
    }

    fn find<F>(&mut self, filter: F) -> ConsoleResult where F: Fn(&str)->bool {
        let mut out = String::new();
        cvar::console::walk(&mut *self, |path, node| {
            if filter(path) {
                node.details(path, &mut out);
            }
        });

        if !out.is_empty() {
            out.into()
        } else {
            ConsoleError::NoResults.into()
        }
    }

    fn help(&mut self, var: &str) -> ConsoleResult {
        let mut out = String::new();
        cvar::console::find(&mut *self, var, |node| {
            node.details(var, &mut out);
        });

        if !out.is_empty() {
            out.into()
        } else {
            ConsoleError::UnknownProperty.into()
        }
    }

    fn cmdtype(&mut self, var: &str) -> CmdType {
        let mut t = CmdType::NotFound;
        cvar::console::find(&mut *self, var, |node| {
            t = node.kind();
        });
        t
    }

    fn exec(&mut self, cmd: &str, args: Vec<&str>, console: &mut ColoredConsole) {
        let ret = match self.cmdtype(cmd) {
            CmdType::Prop => {
                if let Some(val) = args.get(0) {
                    self.set(cmd, val)
                } else {
                    self.get(cmd)
                }
            },
            CmdType::Action => self.call(cmd, &args, console),
            CmdType::List => self.find(|path| path.starts_with(cmd)),
            CmdType::NotFound => {
                ConsoleError::UnknownCommand.into()
            },
        };

        console.write_result(ret);
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

impl From<ConsoleError> for TextSpan {
    fn from(e: ConsoleError) -> TextSpan {
        TextSpan {
            color: [1., 0., 0., 1.],
            text: e.to_string(),
        }
    }
}

pub trait IConsoleExt: cvar::IConsole {
    fn write(&mut self, text: &str);
    fn write_result(&mut self, result: ConsoleResult);
    fn write_colored(&mut self, c: [f32; 4], t: &str);
}

impl std::fmt::Display for TextSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

pub struct ColoredConsole {
    buf: Vec<TextSpan>,
}

impl ColoredConsole {
    pub fn write<S>(&mut self, text: S) where S: Into<TextSpan> {
        self.buf.push(text.into());
    }

    pub fn writeln<S>(&mut self, text: S) where S: Into<TextSpan> {
        let mut span = text.into();
        span.text = span.text.trim_end().to_string();
        if !span.text.is_empty() {
            span.text.push_str("\n");
            self.write(span);
        }
    }
}

impl IConsoleExt for ColoredConsole {
    fn write(&mut self, text: &str) {
        use std::fmt::Write;
        let _ = self.write_str(&text);
    }

    fn write_result(&mut self, result: ConsoleResult) {
        match &*result {
            Ok(output) => {
                self.writeln(output);
            },
            Err(e) => {
                use cvar::IConsole;
                self.write_error(e);
            }
        };
    }

    fn write_colored(&mut self, c: [f32; 4], t: &str) {
        self.write(TextSpan {
            text: t.to_string(),
            color: c,
        });
    }
}

impl std::fmt::Write for ColoredConsole {
    fn write_str(&mut self, s: &str) -> Result<(), std::fmt::Error> {
        self.write(s);
        Ok(())
    }
}

impl cvar::IConsole for ColoredConsole {
    fn write_error(&mut self, err: &(dyn std::error::Error + 'static)) {
        self.writeln(ConsoleError::Custom(err.to_string()));
    }
}

/// The imgui frontend for cvars
/// Call `build` during your rendering stage
pub struct ConsoleWindow {
    root: Box<dyn IVisitExt + Send + Sync>,
    console: ColoredConsole,
    prompt: ImString,
    //history: Vec<String>,
}

impl ConsoleWindow {
    pub fn new(node: Box<dyn IVisitExt + Send + Sync>) -> Self {
        let mut console = ConsoleWindow {
            root: node,
            console: ColoredConsole{ buf: vec![] },
            prompt: ImString::with_capacity(100),
            //history: vec![],
        };
        console.reset_all();
        console
    }
}

impl ConsoleWindow {
    pub fn clear(&mut self) {
        self.console.buf.clear();
    }

    pub fn write<S>(&mut self, text: S) where S: Into<TextSpan> {
        self.console.write(text);
    }

    pub fn writeln<S>(&mut self, text: S) where S: Into<TextSpan> {
        self.console.writeln(text);
    }

    fn write_colored(&mut self, c: [f32; 4], t: &str) {
        self.console.write_colored(c, t);
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
                        //self.close();
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
                let buf = &mut self.console.buf;
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
                    if !span.text.contains('\n') {
                        ui.same_line(0.);
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
                self.write(&format!("{}\n", self.prompt));
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

    pub fn run_cmd(&mut self, cmd: String) {
        let mut parts = cmd.split(' '); // TODO: shellesc
        let cmd = parts.next().unwrap_or("");
        let args = parts.collect::<Vec<_>>();

        let mut console = ColoredConsole{ buf: vec![] };
        self.exec(cmd, args, &mut console);
        self.console.buf.append(&mut console.buf);
    }


    pub fn cmd_help(&mut self, args: &[&str]) {
        let out = {
            if let Some(var) = args.get(0) {
                self.help(var)
            } else {
                self.find(|_| true)
            }
        };
        self.console.write_result(out);
    }

    pub fn cmd_find(&mut self, args: &[&str]) {
        let out = {
            if let Some(var) = args.get(0) {
                self.find(|path| path.contains(var) && path != "find")
            } else {
                Err(ConsoleError::InvalidUsage("find <name>".to_string())).into()
            }
        };
        self.console.write_result(out);
    }

    pub fn cmd_reset(&mut self, args: &[&str]) {
        let out = {
            if let Some(var) = args.get(0) {
                self.reset(var)
            } else {
                self.reset_all()
            }
        };
        self.console.write_result(out);
    }

    pub fn close(&mut self) {
        use cvar::IConsole;
        self.console.write_error(&ConsoleError::Unimplemented);

        //self.open = false;
    }
}

pub trait IVisitExt {
    fn visit_mut(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode), console: &mut dyn IConsoleExt);
}

#[derive(Copy, Clone, Debug)]
pub struct VisitMutExt<F: FnMut(&mut dyn FnMut(&mut dyn cvar::INode), &mut dyn IConsoleExt)>(pub F);
impl<F: FnMut(&mut dyn FnMut(&mut dyn cvar::INode), &mut dyn IConsoleExt)> IVisitExt for VisitMutExt<F> {
	fn visit_mut(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode), console: &mut dyn IConsoleExt) {
		(self.0)(f, console)
	}
}

impl cvar::IVisit for ConsoleWindow {
    fn visit_mut(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode)) {
        //f(&mut cvar::Action("close", "Close the console", |_, _| self.close()));
        f(&mut cvar::Action("help", "List all commands and properties", |args, _| self.cmd_help(args)));
        f(&mut cvar::Action("clear", "Clear the screen", |_, _| self.clear()));
        f(&mut cvar::Action("find", "<text>\nSearch for matching commands", |args, _| self.cmd_find(args)));
        f(&mut cvar::Action("reset", "<var>\nSet a property to its default", |args, _| self.cmd_reset(args)));
        self.root.visit_mut(f, &mut self.console);
    }
}

/// Create a window and initialize the console window with the default config.
/// Be sure to call build on the returned window during your rendering stage
pub fn init<T>(node: T) -> ConsoleWindow
where T: 'static + IVisitExt + Send  + Sync {
    ConsoleWindow::new(Box::new(node))
}
