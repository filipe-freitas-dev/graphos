use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use rand::Rng;
use rand::rngs::OsRng;
use serde::{Serialize, de::DeserializeOwned};
use std::fs;

const MAGIC: &[u8; 4] = b"GRPH";
const VERSION: u8 = 1;

#[derive(Debug)]
pub enum PersistError {
    Io(String),
    Serde(String),
    Crypto(String),
    // Format(String),
}

impl From<std::io::Error> for PersistError {
    fn from(e: std::io::Error) -> Self {
        PersistError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for PersistError {
    fn from(e: serde_json::Error) -> Self {
        PersistError::Serde(e.to_string())
    }
}

fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], PersistError> {
    let salt_str = SaltString::encode_b64(salt).map_err(|e| PersistError::Crypto(e.to_string()))?;
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(passphrase.as_bytes(), &salt_str)
        .map_err(|e| PersistError::Crypto(e.to_string()))?;
    let hash = password_hash
        .hash
        .ok_or_else(|| PersistError::Crypto("missing hash".into()))?;
    let mut key_bytes = [0u8; 32];
    let take = key_bytes.len().min(hash.as_bytes().len());
    key_bytes[..take].copy_from_slice(&hash.as_bytes()[..take]);
    Ok(key_bytes)
}

pub fn save_encrypted<T: Serialize>(value: &T, path: &str, passphrase: &str) -> Result<(), String> {
    let mut salt = [0u8; 16];
    let mut nonce = [0u8; 12];
    OsRng.fill(&mut salt);
    OsRng.fill(&mut nonce);
    let key_bytes = derive_key(passphrase, &salt).map_err(|e| format!("{:?}", e))?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_bytes));
    let nonce = Nonce::from_slice(&nonce);

    let json = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    let ciphertext = cipher
        .encrypt(nonce, json.as_ref())
        .map_err(|e| format!("encrypt: {}", e))?;

    let mut out = Vec::with_capacity(4 + 1 + 16 + 12 + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&salt);
    out.extend_from_slice(nonce);
    out.extend_from_slice(&ciphertext);

    if let Some(parent) = std::path::Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, out).map_err(|e| e.to_string())
}

pub fn load_encrypted<T: DeserializeOwned>(path: &str, passphrase: &str) -> Result<T, String> {
    let data = fs::read(path).map_err(|e| e.to_string())?;
    if data.len() < 4 + 1 + 16 + 12 {
        return Err("file too short".into());
    }
    if &data[0..4] != MAGIC {
        return Err("bad magic".into());
    }
    let _version = data[4];
    let salt = &data[5..21];
    let nonce_bytes = &data[21..33];
    let ciphertext = &data[33..];

    let key_bytes = derive_key(passphrase, salt).map_err(|e| format!("{:?}", e))?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_bytes));
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("decrypt: {}", e))?;
    serde_json::from_slice::<T>(&plaintext).map_err(|e| e.to_string())
}
