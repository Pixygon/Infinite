//! Infinite World - World management and time travel system
//!
//! Provides chunk-based world streaming, era/timeline system, and time portals.

pub mod chunk;
pub mod era_config;
pub mod terrain;
pub mod time_of_day;
pub mod weather;

pub use chunk::{Chunk, ChunkConfig, ChunkCoord, ChunkManager};
pub use era_config::EraTerrainConfig;
pub use terrain::{Terrain, TerrainConfig};
pub use time_of_day::{SkyColors, TimeOfDay};
pub use weather::{Weather, WeatherState};
