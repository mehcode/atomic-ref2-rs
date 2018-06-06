use super::IntoOptionArc;
use std::mem;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;

/// An atomic reference that may be updated atomically.
pub struct AtomicOptionRef<T> {
    ptr: AtomicPtr<T>,
}

impl<T> AtomicOptionRef<T> {
    /// Creates a new atomic reference with `None` initial value.
    pub fn new() -> Self {
        Self {
            ptr: AtomicPtr::new(null_mut()),
        }
    }

    /// Creates a new atomic reference from the given initial value.
    pub fn from(value: impl IntoOptionArc<T>) -> Self {
        Self {
            ptr: AtomicPtr::new(option_arc_to_ptr(value)),
        }
    }

    /// Returns `true` if the optional reference has `Some` value.
    pub fn is_some(&self) -> bool {
        self.ptr.load(Ordering::SeqCst).is_null()
    }

    /// Loads and returns a reference to the value or `None`
    /// if the value is not set.
    pub fn load(&self) -> Option<Arc<T>> {
        ptr_to_option_arc(self.ptr.load(Ordering::SeqCst))
    }

    /// Stores the value.
    pub fn store(&self, value: impl IntoOptionArc<T>) {
        self.ptr.store(option_arc_to_ptr(value), Ordering::SeqCst);
    }

    /// Swaps the value, returning the previous value.
    pub fn swap(&self, value: impl IntoOptionArc<T>) -> Option<Arc<T>> {
        ptr_to_option_arc(self.ptr.swap(option_arc_to_ptr(value), Ordering::SeqCst))
    }
}

impl<T> Drop for AtomicOptionRef<T> {
    fn drop(&mut self) {
        let ptr = self.ptr.swap(null_mut(), Ordering::SeqCst);
        if !ptr.is_null() {
            unsafe {
                // Reconstruct the Arc from the raw ptr which will trigger our destructor
                // if there is one
                let _ = Arc::from_raw(ptr);
            }
        }
    }
}

fn option_arc_to_ptr<T>(value: impl IntoOptionArc<T>) -> *mut T {
    if let Some(value) = value.into_option_arc() {
        Arc::into_raw(value) as *mut _
    } else {
        null_mut()
    }
}

fn ptr_to_option_arc<T>(ptr: *mut T) -> Option<Arc<T>> {
    if ptr.is_null() {
        // Return `None` if null is stored in the AtomicPtr
        None
    } else {
        // Otherwise, reconstruct the stored Arc
        let value = unsafe { Arc::from_raw(ptr) };

        // Increment the atomic reference count
        mem::forget(Arc::clone(&value));

        // And return our reference
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::AtomicOptionRef;

    #[test]
    fn test_store_load() {
        let m = AtomicOptionRef::<String>::new();

        // Store
        m.store(String::from("2"));

        // Load and assert
        assert_eq!(m.load().unwrap().as_ref(), "2");
    }

    #[test]
    fn test_overwrite() {
        let m = AtomicOptionRef::<String>::new();

        // Store
        m.store(String::from("Hello World"));

        // Take a reference
        let m0 = m.load();

        // Store (again)
        m.store(String::from("Goodbye World"));

        // Compare value of stored
        assert_eq!(m0.unwrap().as_ref(), "Hello World");

        // Compare value of new
        assert_eq!(m.load().unwrap().as_ref(), "Goodbye World");
    }
}
