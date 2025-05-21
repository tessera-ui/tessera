mod renderer;
mod tokio_runtime;

use renderer::Renderer;

fn main() -> Result<(), impl std::error::Error> {
    env_logger::init();
    Renderer::run()
}
