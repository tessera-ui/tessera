use std::any::Any;

/// Every draw command is a command that can be executed by the drawer.
pub trait DrawCommand: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}
