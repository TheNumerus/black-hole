use clap::Parser;

use blackhole::marcher::RayMarcher;

mod app;
mod args;
mod renderer;
mod scene_loader;
mod shaders;

use app::App;
use args::ArgsInteractive;
use renderer::{InteractiveRenderer, Scaling};
use scene_loader::SceneLoader;

fn main() {
    // clion needs help in trait annotation
    let args = <ArgsInteractive as Parser>::parse();

    let loader = SceneLoader::new();

    let renderer = InteractiveRenderer {
        ray_marcher: RayMarcher {
            mode: args.mode.into(),
            ..Default::default()
        },
        samples: args.samples,
        threads: args.threads,
        scaling: Scaling::X1,
        ..Default::default()
    };

    let app = App::new(renderer, loader).unwrap();

    app.run();
}
