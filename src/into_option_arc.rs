use std::sync::Arc;

pub trait IntoOptionArc<T> {
    fn into_option_arc(self) -> Option<Arc<T>>;
}

impl<T> IntoOptionArc<T> for T {
    fn into_option_arc(self) -> Option<Arc<Self>> {
        Some(Arc::new(self))
    }
}

impl<T> IntoOptionArc<T> for Arc<T> {
    fn into_option_arc(self) -> Option<Self> {
        Some(self)
    }
}

impl<T> IntoOptionArc<T> for Option<Arc<T>> {
    fn into_option_arc(self) -> Self {
        self
    }
}
