// mod brightness;
// mod manager;
mod config_migrator;
mod display_config;
mod display_handler;
mod display_matcher;
mod display_registry;
mod display_state;
mod manager;

#[cfg(test)]
mod tests;

pub use config_migrator::*;
pub use display_config::*;
pub use display_matcher::*;
pub use display_registry::*;
pub use display_state::*;

// pub use brightness::*;
pub use manager::*;
