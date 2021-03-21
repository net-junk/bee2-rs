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
    fn hash(hash: &mut [u8], src: impl AsRef<[u8]>);

    /// Get output size of the hasher
    fn output_size() -> usize;
}

/// The `PrgHasher` trait specifies an interface common for all PrgHashers.
pub trait PrgHasher {
    /// PrgHasher initializing.
    /// ann - annotation, ann.len() % 4 == 0 && ann.len() <= 60
    fn new(ann: impl AsRef<[u8]>) -> Self;

    /// Calculate hash.
    /// # Arguments
    /// data - data(message) to hash
    /// hash - output container.
    fn hash(&mut self, data: impl AsRef<[u8]>, hash: &mut [u8]);
}

/// The `PrgAEAD` trait specifies an interface common for all AEADs(authenticated encryption with associated data).
pub trait PrgAEAD {
    /// PrgAE initializing.
    /// ann - annotation, ann.len() % 4 == 0 && ann.len() <= 60
    /// key - key, key.len() % 4 == 0 && key.len() <= 60, key.len() == 0 || key.len() >= l / 8
    fn new(ann: impl AsRef<[u8]>, key: impl AsRef<[u8]>) -> Self;

    /// Encrypt data.
    /// plaintext - data to encrypt.
    /// header - associated data.
    /// ciphertext - to store result, len(ciphertext) == len(plaintext).
    /// tag - authentication tag(message authentication code).
    ///
    fn encrypt(
        &mut self,
        plaintext: impl AsRef<[u8]>,
        header: impl AsRef<[u8]>,
        ciphertext: &mut [u8],
        tag: &mut [u8],
    );

    /// Decrypt data.
    /// ciphertext - data to decrypt.
    /// header - associated data.
    /// tag - authentication tag(message authentication code).
    /// plaintext - to store result.
    ///
    fn decrypt(
        &mut self,
        ciphertext: impl AsRef<[u8]>,
        header: impl AsRef<[u8]>,
        tag: impl AsRef<[u8]>,
        plaintext: &mut [u8],
    );
}

/// The `PrgStart` trait specifies an interface for command `start`.
pub trait PrgStart {
    /// Automaton initializing.
    ///
    /// # Arguments
    /// l - security level, l == 128 || l == 192 || l == 256
    /// d - capacity, d == 1 || d == 2
    /// ann - annotation, ann.len() % 4 == 0 && ann.len() <= 60
    /// key - key, key.len() % 4 == 0 && key.len() <= 60, key.len() == 0 || key.len() >= l / 8
    ///
    fn start(l: usize, d: usize, ann: impl AsRef<[u8]>, key: impl AsRef<[u8]>) -> Self;
}
/// The `PrgRestart` trait specifies an interface for command `restart`.
pub trait PrgRestart {
    /// Automaton re-initializing.
    ///
    /// # Arguments
    /// ann - annotation, ann.len() % 4 == 0 && ann.len() <= 60
    /// key - key, key.len() % 4 == 0 && key.len() <= 60, key.len() == 0 || key.len() >= l / 8
    ///
    /// # Panics
    /// `PrgStart::start()` < `PrgRestart::restart()`
    fn restart(&mut self, ann: impl AsRef<[u8]>, key: impl AsRef<[u8]>);
}
/// The `PrgAbsorb` trait specifies an interface for command `absorb`.
pub trait PrgAbsorb {
    /// Initializing of data loading into automaton.  
    ///
    /// # Panics
    /// `PrgStart::start()` < `PrgAbsorb::absorb_start()`
    fn absorb_start(&mut self);

    /// Loading step into automaton.  
    ///
    /// # Arguments
    /// buf - data to load
    ///
    /// # Panics
    /// `PrgAbsorb::absorb_start()` < `PrgAbsorb::absorb_step()`
    fn absorb_step(&mut self, buf: impl AsRef<[u8]>);

    /// Load data into automaton.  
    ///
    /// # Arguments
    /// buf - data to load
    ///
    /// # Panics
    /// `PrgStart::start()` < `PrgAbsorb::absorb()`
    fn absorb(&mut self, buf: impl AsRef<[u8]>);
}

/// The `PrgSqueeze` trait specifies an interface for command `squeeze`.
pub trait PrgSqueeze {
    /// Initializing of data unloading from automaton.  
    ///
    /// # Panics
    /// `PrgStart::start()` < `PrgSqueeze::squeeze_start()`
    fn squeeze_start(&mut self);

    /// Unloading step from automaton.  
    ///
    /// # Arguments
    /// buf - data to unload
    ///
    /// # Panics
    /// `PrgSqueeze::squeeze_start()` < `PrgSqueeze::squeeze_step()`
    fn squeeze_step(&mut self, buf: &mut [u8]);

    /// Unload data from automaton.  
    ///
    /// # Arguments
    /// buf - data to unload
    ///
    /// # Panics
    /// `PrgStart::start()` < `PrgSqueeze::squeeze()`
    fn squeeze(&mut self, buf: &mut [u8]);
}

/// The `PrgEncr` trait specifies an interface for command `encr`.
pub trait PrgEncr {
    /// Initializing of data encryption using automaton.  
    ///
    /// # Panics
    /// `PrgStart::start()` < `PrgEncr::encr_start()`
    fn encr_start(&mut self);

    /// Encryption step using automaton.  
    ///
    /// # Arguments
    /// buf - data to encrypt
    ///
    /// # Panics
    /// `PrgEncr::encr_start()` < `PrgEncr::encr_step()`
    fn encr_step(&mut self, buf: &mut [u8]);

    /// Encryption using automaton.  
    ///
    /// # Arguments
    /// buf - data to encrypt
    ///
    /// # Panics
    /// `PrgStart::start()` < `PrgEncr::encr()`
    fn encr(&mut self, buf: &mut [u8]);
}

/// The `PrgDecr` trait specifies an interface for command `decr`.
pub trait PrgDecr {
    /// Initializing of data decryption using automaton.  
    ///
    /// # Panics
    /// `PrgStart::start()` < `PrgDecr::decr_start()`
    fn decr_start(&mut self);

    /// Decryption step using automaton.  
    ///
    /// # Arguments
    /// buf - data to decrypt
    ///
    /// # Panics
    /// `PrgDecr::decr_start()` < `PrgDecr::decr_step()`
    fn decr_step(&mut self, buf: &mut [u8]);

    /// Decryption using automaton.  
    ///
    /// # Arguments
    /// buf - data to decrypt
    ///
    /// # Panics
    /// `PrgStart::start() < PrgDecr::decr()`
    fn decr(&mut self, buf: &mut [u8]);
}

/// The `PrgRatchet` trait specifies an interface for command `ratchet`.
pub trait PrgRatchet {
    /// Automaton changing.
    /// The state of automaton changes so that it is difficult to determine the previous state.
    ///
    /// # Panics
    /// `PrgStart::start()` < `PrgRatchet::ratchet()`
    fn ratchet(&mut self);
}

/// The `Prg` trait specifies an interface common for Automaton.
/// Implements `PrgStart` + `PrgRestart` + `PrgSqueeze` + `PrgAbsorb` + `PrgEncr` + `PrgDecr` + `PrgRatchet`
pub trait Prg:
    PrgStart + PrgRestart + PrgSqueeze + PrgAbsorb + PrgEncr + PrgDecr + PrgRatchet
{
    /// Get output size of the automaton.
    fn output_size() -> usize;
}
