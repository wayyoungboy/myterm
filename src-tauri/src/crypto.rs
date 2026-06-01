use aes::Aes256;
use cbc::{Decryptor, Encryptor};
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use sha2::{Sha256, Digest};

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

const SALT: &[u8] = b"myterm-app-v1-salt";

fn derive_key(master_password: &str) -> ([u8; 32], [u8; 16]) {
    let mut hasher = Sha256::new();
    hasher.update(SALT);
    hasher.update(master_password.as_bytes());
    let key_hash = hasher.finalize();

    let mut hasher2 = Sha256::new();
    hasher2.update(b"iv");
    hasher2.update(key_hash.as_slice());
    let iv_hash = hasher2.finalize();

    let mut key = [0u8; 32];
    key.copy_from_slice(&key_hash[..32]);

    let mut iv = [0u8; 16];
    iv.copy_from_slice(&iv_hash[..16]);

    (key, iv)
}

pub fn encrypt_password(plaintext: &str, master_password: &str) -> String {
    let (key, iv) = derive_key(master_password);
    let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
    let encrypted = cipher.encrypt_padded_vec_mut::<Pkcs7>(plaintext.as_bytes());
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(&encrypted)
}

pub fn decrypt_password(encrypted: &str, master_password: &str) -> Result<String, String> {
    let (key, iv) = derive_key(master_password);
    use base64::Engine;
    let encrypted_bytes = base64::engine::general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    let cipher = Aes256CbcDec::new(&key.into(), &iv.into());
    let decrypted = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted_bytes)
        .map_err(|e| format!("Decrypt failed: {}", e))?;

    String::from_utf8(decrypted).map_err(|e| format!("UTF-8 decode failed: {}", e))
}

pub fn get_master_password() -> String {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "myterm-default".to_string());
    format!("myterm-{}", hostname)
}
