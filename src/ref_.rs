use super::AtomicOptionRef;
use std::sync::Arc;

/// An atomic reference that may be updated atomically.
pub struct AtomicRef<T> {
    inner: AtomicOptionRef<T>
}

impl<T> AtomicRef<T> {
    /// Creates a new atomic reference with a default initial value.
    pub fn new() -> Self where T: Default {
        Self::from(T::default())
    }

    /// Creates a new atomic reference from the given initial value.
    pub fn from(value: impl Into<Arc<T>>) -> Self {
        Self { inner: AtomicOptionRef::from(value.into()) }
    }

    /// Loads and returns a reference to the value.
    pub fn load(&self) -> Arc<T> {
        self.inner.load().unwrap()
    }

    /// Stores the value.
    pub fn store(&self, value: impl Into<Arc<T>>) {
        self.inner.store(value.into())
    }

    /// Swaps the value, returning the previous value.
    pub fn swap(&self, value: impl Into<Arc<T>>) -> Arc<T> {
        self.inner.swap(value.into()).unwrap()
    }
}
