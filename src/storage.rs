pub trait StorageBackend<T> {
    fn as_slice(&self) -> &[T];
    fn capacity(&self) -> usize;
    fn len(&self) -> usize;

    fn from_fn<R>(f: impl Fn(usize) -> R) -> Self;
}
