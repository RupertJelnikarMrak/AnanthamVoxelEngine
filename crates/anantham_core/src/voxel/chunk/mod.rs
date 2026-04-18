pub mod compression;
pub mod data;
pub mod local;

pub use compression::attempt_compression;
pub use data::{CHUNK_SIZE, CHUNK_VOLUME, Chunk, ChunkData};
pub use local::ChunkError;
