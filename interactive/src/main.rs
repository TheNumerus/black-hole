use clap::Parser;

use blackhole::marcher::RayMarcher;
use blackhole_common::scene_loader::SceneLoader;

mod app;
mod args;
mod renderer;

use app::App;
use args::ArgsInteractive;
use renderer::{InteractiveRenderer, Scaling};

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
        scaling: Scaling::X2,
        ..Default::default()
    };

    let app = App::new(renderer, loader).unwrap();

    app.run();
}