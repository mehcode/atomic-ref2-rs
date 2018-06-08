use super::spinlock::SpinRwLock;
use super::IntoOptionArc;
use std::mem;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;

/// An atomic reference that may be updated atomically.
pub struct AtomicOptionRef<T> {
    ptr: AtomicPtr<T>,
    lock: SpinRwLock,
}

impl<T> AtomicOptionRef<T> {
    /// Creates a new atomic reference with `None` initial value.
    pub fn new() -> Self {
        Self::from(None)
    }

    /// Creates a new atomic reference from the given initial value.
    pub fn from(value: impl IntoOptionArc<T>) -> Self {
        Self {
            ptr: AtomicPtr::new(option_arc_to_ptr(value)),
            lock: SpinRwLock::new(),
        }
    }

    /// Returns `true` if the optional reference has `Some` value.
    pub fn is_some(&self) -> bool {
        self.ptr.load(Ordering::SeqCst).is_null()
    }

    /// Loads and returns a reference to the value or `None`
    /// if the value is not set.
    pub fn load(&self) -> Option<Arc<T>> {
        let _guard = self.lock.read();
        ptr_to_option_arc(self.ptr.load(Ordering::SeqCst), true)
    }

    /// Stores the value.
    pub fn store(&self, value: impl IntoOptionArc<T>) {
        self.swap(value);
    }

    /// Swaps the value, returning the previous value.
    pub fn swap(&self, value: impl IntoOptionArc<T>) -> Option<Arc<T>> {
        let _guard = self.lock.write();
        ptr_to_option_arc(
            self.ptr.swap(option_arc_to_ptr(value), Ordering::SeqCst),
            false,
        )
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

fn ptr_to_option_arc<T>(ptr: *mut T, increment: bool) -> Option<Arc<T>> {
    if ptr.is_null() {
        // Return `None` if null is stored in the AtomicPtr
        None
    } else {
        // Otherwise, reconstruct the stored Arc
        let value = unsafe { Arc::from_raw(ptr) };

        if increment {
            // Increment the atomic reference count
            mem::forget(Arc::clone(&value));
        }

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

    #[test]
    fn test_drop() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static DROPS: AtomicUsize = AtomicUsize::new(0);

        struct Foo;

        impl Drop for Foo {
            fn drop(&mut self) {
                DROPS.fetch_add(1, Ordering::SeqCst);
            }
        }

        let m = AtomicOptionRef::<Foo>::new();

        m.swap(Foo);
        m.swap(Foo);

        assert_eq!(DROPS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_threads() {
        use rand::{thread_rng, Rng};
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use std::thread;
        use std::time::Duration;

        const THREADS: usize = 100;
        const ITERATIONS: usize = 100;

        static DROPS: AtomicUsize = AtomicUsize::new(0);

        #[derive(Default)]
        struct Foo {
            dropped: AtomicUsize,
        };

        impl Drop for Foo {
            fn drop(&mut self) {
                self.dropped.fetch_add(1, Ordering::SeqCst);
                DROPS.fetch_add(1, Ordering::SeqCst);
            }
        }

        let m = Arc::new(AtomicOptionRef::<Foo>::new());
        m.store(Foo::default());

        let mut threads = Vec::new();

        for _ in 0..THREADS {
            let m0 = Arc::clone(&m);
            threads.push(thread::spawn(move || {
                for _ in 0..ITERATIONS {
                    let value = m0.load().unwrap();

                    assert_eq!(value.dropped.load(Ordering::SeqCst), 0);

                    let ms = thread_rng().gen_range(0, 10);
                    thread::sleep(Duration::from_millis(ms));
                }
            }));

            let m1 = Arc::clone(&m);
            threads.push(thread::spawn(move || {
                for _ in 0..ITERATIONS {
                    m1.swap(Foo::default());

                    let ms = thread_rng().gen_range(0, 10);
                    thread::sleep(Duration::from_millis(ms));
                }
            }));
        }

        for thread in threads {
            let _ = thread.join();
        }

        assert_eq!(DROPS.load(Ordering::SeqCst), (THREADS * ITERATIONS));
    }
}
