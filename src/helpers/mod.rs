/// An interface for mutable collections that can be popped conditionally.
///
/// Inspired by `peekable().next_if()`
pub trait PopIf {
    type Item;

    /// Pops the next element only if it passes `f`
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x: Vec<u32> = vec![0, 1, 2, 3];
    ///
    /// let e = x.pop_if(|x| *x < 2u32);
    /// assert_eq!(e, Some(0));
    /// let e = x.pop_if(|x| *x < 2u32);
    /// assert_eq!(e, Some(1));
    /// let e = x.pop_if(|x| *x < 2u32);
    /// assert_eq!(e, None);
    /// ```
    fn pop_if<F>(&mut self, f: F) -> Option<Self::Item>
    where
        F: FnOnce(&Self::Item) -> bool;
}

impl<A> PopIf for Vec<A> {
    type Item = A;

    /// Testing doc tests
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x: Vec<u32> = Vec::new();
    /// x.push(1);
    /// assert_eq!(2, 2);
    /// ```
    fn pop_if<F>(&mut self, f: F) -> Option<Self::Item>
    where
        F: FnOnce(&Self::Item) -> bool,
    {
        self.get(0).filter(|x| f(*x))?;
        self.pop()
    }
}
