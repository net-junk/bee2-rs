/// The `Hasher` trait specifies an interface common for hasher functions as in original Bee2.
pub trait Hasher {
    fn new() -> Self;

    /// Hashing block of data.
    /// Hash-value from state is recalculated with new data.
    fn step_h(&mut self, buf: impl AsRef<[u8]>);

    // Getting of hash_value.
    //
    fn step_g(&mut self, hash: &mut [u8]);

    ///
    fn step_v(&mut self, hash: impl AsRef<[u8]>) -> bool;

    ///
    fn hash(hash:&mut [u8], src: impl AsRef<[u8]>);

    /// Get output size of the hasher
    fn output_size(&self) -> usize;
}
