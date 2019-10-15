[![Latest release on crates.io](https://meritbadge.herokuapp.com/amethyst-console)](https://crates.io/crates/amethyst-console)
[![Documentation on docs.rs](https://docs.rs/amethyst-console/badge.svg)](https://docs.rs/amethyst-console)

# amethyst-console

A framework around `cvar` and `imgui` that allows you to easily modify
system configurations at runtime in a user configurable way.

![preview](https://i.imgur.com/c0u1ixc.png)

### Examples:
 * `width 120` - Set width to 120
 * `width` - Print the current width
 * `reset width` - Reset width to its default value (100)
 * `find a` - Find all commands with `a` in their name
 * `reset` - Reset all variables to their defaults

## Setup

Add this to your `Cargo.toml`

```toml
[dependencies]
amethyst-console = "0.1.0"
```

## Basic Example

### Create your config

Be sure it supports the default trait

```rust
#[derive(Default)]
pub struct MyConfig {
    pub height: f32,
    pub width: f32,
}
```

### Implement the visit trait

```rust
impl IVisitExt for MyConfig {
    fn visit_mut_ext(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode), _console: &mut dyn IConsoleExt) {
        // You can add variables
        f(&mut cvar::Property("width", "Arena width", &mut self.width, 100);
        // Or callable functions
        f(&mut cvar::Action("color_test", "Test console colors", |_, _| color_test(console)));
    }
}
```

### Create a system

* Call `create_system` using a type parameter to specify the name of your config.

```rust
/// This will:
///    - Initialize the struct to its default value
///    - Add it to the world so other services can read in their run loops
///    - Create a console window with everything added by `visit_mut_ext`
let console_system = imgui_console::create_system::<MyConfig>();
```

* Add the system to your app initialization.

```rust
let game_data = GameDataBuilder::default()
    .with_system_desc(console_system, "imgui_console", &[]) // <--- ADDED
    // ....
```

### Use the config in your systems

```rust
impl<'s> System<'s> for ExampleSystem {
    // Use Read to grab the resource from the World
    type SystemData = (
            Read<'s, GameConfig>,
        );

    fn run(&mut self, (game_config, ): Self::SystemData) {
        // This print statement will change the moment a user types a set command,
        println!("width={}", &config.width);
    }
}
```

### Add a console binding

Update your `input.ron` file. This will let users open/close the console.

```
(
    axes: {},
    actions: {
        "toggle_console": [[Key(Escape)]],
    },
)
```

### Done

That's it. Your system is now configurable by the user intiated commands. Have fun!

See `examples/demo_console.rs` for a complete example with more comments.

## Standalone usage

 * Disable the `amethyst-system` feature.

```toml
[dependencies.amethyst-console ]
version = "0.1.0"
default-features = false
features = []
```

 * Create a console window and your config

```rust
let mut console = imgui_console::create_console();

let mut config = MyConfig::default();
```

 * Call build in your render loop

```rust
let ui: imgui::Ui = ... ;

loop {
    // Some other redering code
    // ...

    // Draw the console.
    // Pass in the config you would like to be updated.
    let window = imgui::Window::new(im_str!("Console")).opened(&mut self.open);
    conosle.build(ui, window, &mut config);
}
```
