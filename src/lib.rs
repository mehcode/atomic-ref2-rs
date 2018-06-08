//! Provides `AtomicOptionRef` and `AtomicRef`.
//! Intended to map to [java.util.concurrent.atomic.AtomicReference](https://docs.oracle.com/javase/8/docs/api/java/util/concurrent/atomic/AtomicReference.html) in Java.

#[cfg(test)]
extern crate rand;

mod into_option_arc;
mod option_ref;
mod ref_;
mod spinlock;

pub use self::into_option_arc::IntoOptionArc;
pub use self::option_ref::AtomicOptionRef;
pub use self::ref_::AtomicRef;
