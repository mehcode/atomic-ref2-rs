//! Provides AtomicOptionRef and AtomicRef.
//! Intended to map to [java.util.concurrent.atomic.AtomicReference](https://docs.oracle.com/javase/8/docs/api/java/util/concurrent/atomic/AtomicReference.html) in Java.

mod into_option_arc;
mod option_ref;
mod ref_;

pub use self::into_option_arc::IntoOptionArc;
pub use self::option_ref::AtomicOptionRef;
pub use self::ref_::AtomicRef;
