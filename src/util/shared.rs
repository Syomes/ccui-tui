use parking_lot::Mutex;
use std::sync::Arc;

/// Shared state container (Arc + Mutex).
///
/// Used to share state between multiple event listeners.
///
/// # Example
/// ```rust
/// use ccui::Shared;
///
/// let counter = Shared::new(0);
///
/// // Clone reference (not cloning data)
/// let counter2 = counter.clone_ref();
///
/// doc.add_event_listener("btn", EventType::Click, {
///     let counter = counter.clone_ref();
///     move |_| {
///         counter.with(|c| *c += 1);
///     }
/// })?;
/// ```
pub struct Shared<T>(Arc<Mutex<T>>);

impl<T> Shared<T> {
    /// Create a new shared state container.
    pub fn new(value: T) -> Self {
        Shared(Arc::new(Mutex::new(value)))
    }

    /// Clone the reference.
    ///
    /// The two `Shared` instances will point to the same data.
    pub fn clone_ref(&self) -> Self {
        Shared(Arc::clone(&self.0))
    }

    /// Access the inner data.
    ///
    /// # Example
    /// ```rust
    /// counter.with(|c| *c += 1);
    /// ```
    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(&mut self.0.lock())
    }

    /// Read the inner data (immutable).
    ///
    /// # Example
    /// ```rust
    /// let value = counter.read(|c| *c);
    /// ```
    pub fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&self.0.lock())
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        self.clone_ref()
    }
}

impl<T> Default for Shared<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}
