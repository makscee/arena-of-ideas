use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use rand::{rng, Rng};
use sha2::{Digest, Sha256};

const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
    abcdefghijklmnopqrstuvwxyz\
    0123456789-.~_";

pub fn pkce_verifier() -> Vec<u8> {
    let mut rng = rng();
    let length = rng.random_range(43..=128);
    (0..length)
        .map(|_| {
            let i = rng.random_range(0..CHARS.len());
            CHARS[i]
        })
        .collect()
}

fn base64_url_encode(input: &[u8]) -> String {
    let b64 = base64::engine::general_purpose::STANDARD.encode(input);
    b64.chars()
        .filter_map(|c| match c {
            '=' => None,
            '+' => Some('-'),
            '/' => Some('_'),
            x => Some(x),
        })
        .collect()
}

pub fn pkce_challenge(verifier: &[u8]) -> String {
    let mut sha = Sha256::new();
    sha.update(verifier);
    let result = sha.finalize();
    base64_url_encode(&result[..])
}

pub fn generate_csrf_state() -> String {
    let random_bytes: Vec<u8> = (0..16).map(|_| rng().random::<u8>()).collect();
    BASE64_URL_SAFE_NO_PAD.encode(random_bytes)
}
