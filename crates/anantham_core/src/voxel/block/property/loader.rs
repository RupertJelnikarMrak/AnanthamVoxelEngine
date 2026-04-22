use bevy::asset::{Asset, AssetLoader, LoadContext, io::Reader};
use bevy::reflect::TypePath;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PropertyLoaderError {
    #[error("Could not read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse RON: {0}")]
    Ron(#[from] ron::error::SpannedError),
}

#[derive(TypePath)]
pub struct BlockPropertyLoader<A> {
    extension: &'static str,
    _marker: PhantomData<A>,
}

impl<A> BlockPropertyLoader<A> {
    pub fn new(extension: &'static str) -> Self {
        Self {
            extension,
            _marker: PhantomData,
        }
    }
}

impl<A: Asset + DeserializeOwned> AssetLoader for BlockPropertyLoader<A> {
    type Asset = A;
    type Settings = ();
    type Error = PropertyLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset = ron::de::from_bytes::<A>(&bytes)?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        std::slice::from_ref(&self.extension)
    }
}
