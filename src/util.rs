use sha2::{Digest, Sha512};

pub fn hash(str: &str) -> String {
    let mut hasher = Sha512::new();
    hasher.update(str);
    let hash = hasher.finalize();
    data_encoding::HEXLOWER.encode(&hash)
}
