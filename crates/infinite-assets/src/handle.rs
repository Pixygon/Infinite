use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique identifier for a loaded asset.
pub type AssetId = u64;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Allocate a new unique asset ID.
pub(crate) fn next_asset_id() -> AssetId {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// A typed handle referencing a loaded asset in the AssetServer.
#[derive(Debug)]
pub struct AssetHandle<T> {
    id: AssetId,
    _marker: PhantomData<T>,
}

impl<T> AssetHandle<T> {
    pub(crate) fn new(id: AssetId) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    /// The unique ID of this asset.
    pub fn id(&self) -> AssetId {
        self.id
    }
}

impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for AssetHandle<T> {}

impl<T> PartialEq for AssetHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for AssetHandle<T> {}

impl<T> std::hash::Hash for AssetHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
