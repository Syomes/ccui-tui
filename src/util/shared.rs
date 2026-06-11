use parking_lot::Mutex;
use std::sync::Arc;

/// Shared state container (Arc + Mutex).
///
/// Used to share state between multiple event listeners.
///
/// # Example
/// ```rust
/// use ccui::util::shared::Shared;
///
/// let counter = Shared::new(0);
///
/// // Write to the shared value
/// counter.with(|c| *c += 1);
///
/// // Read from it
/// let val = counter.read(|c| *c);
/// assert_eq!(val, 1);
///
/// // Clone the reference (both point to the same data)
/// let counter2 = counter.clone_ref();
/// counter2.with(|c| *c += 1);
/// assert_eq!(counter.read(|c| *c), 2);
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
    /// # use ccui::util::shared::Shared;
    /// # let counter = Shared::new(0);
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
    /// # use ccui::util::shared::Shared;
    /// # let counter = Shared::new(0);
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
