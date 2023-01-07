use std::sync::atomic::AtomicUsize;

static TOTAL_STEPS: AtomicUsize = AtomicUsize::new(0);
static MAX_STEPS_PER_SAMPLE: AtomicUsize = AtomicUsize::new(0);

mod cli;

pub use cli::CliRenderer;
