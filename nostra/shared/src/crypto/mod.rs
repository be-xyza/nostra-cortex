use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;
use thiserror::Error;
use x25519_dalek::{PublicKey, StaticSecret};

pub const ALG_HPKE_X25519_CHACHA20POLY1305: &str = "hpke-x25519-chacha20poly1305";
pub const ENC_VERSION_V1: u32 = 1;

#[derive(Debug, Clone)]
pub struct CryptoEnvelope {
    pub alg: String,
    pub enc_version: u32,
    pub key_id: String,
    pub ephemeral_pub_key: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("invalid public key length: expected 32 bytes")]
    InvalidPublicKeyLength,
    #[error("invalid private key length: expected 32 bytes")]
    InvalidPrivateKeyLength,
    #[error("invalid envelope: missing ephemeral public key")]
    MissingEphemeralPublicKey,
    #[error("encryption failed: {0}")]
    EncryptFailed(String),
    #[error("decryption failed: {0}")]
    DecryptFailed(String),
    #[error("hkdf expand failed")]
    HkdfExpandFailed,
}

fn derive_key_nonce(shared_secret: &[u8]) -> Result<(Key, Nonce), CryptoError> {
    let hk = Hkdf::<Sha256>::new(None, shared_secret);
    let mut okm = [0u8; 44]; // 32 key + 12 nonce
    hk.expand(b"nostra-hpke-v1", &mut okm)
        .map_err(|_| CryptoError::HkdfExpandFailed)?;
    let key = Key::from_slice(&okm[..32]).to_owned();
    let nonce = Nonce::from_slice(&okm[32..]).to_owned();
    Ok((key, nonce))
}

pub fn encrypt_hpke_x25519_chacha20poly1305(
    plaintext: &[u8],
    recipient_public_key: &[u8],
    key_id: &str,
) -> Result<CryptoEnvelope, CryptoError> {
    let recipient_pub: [u8; 32] = recipient_public_key
        .try_into()
        .map_err(|_| CryptoError::InvalidPublicKeyLength)?;
    let recipient_pub = PublicKey::from(recipient_pub);

    let mut eph_bytes = [0u8; 32];
    getrandom::getrandom(&mut eph_bytes).map_err(|e| CryptoError::EncryptFailed(e.to_string()))?;
    let eph_secret = StaticSecret::from(eph_bytes);
    let eph_public = PublicKey::from(&eph_secret);
    let shared = eph_secret.diffie_hellman(&recipient_pub);

    let (key, nonce) = derive_key_nonce(shared.as_bytes())?;
    let cipher = ChaCha20Poly1305::new(&key);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| CryptoError::EncryptFailed(e.to_string()))?;

    Ok(CryptoEnvelope {
        alg: ALG_HPKE_X25519_CHACHA20POLY1305.to_string(),
        enc_version: ENC_VERSION_V1,
        key_id: key_id.to_string(),
        ephemeral_pub_key: eph_public.as_bytes().to_vec(),
        ciphertext,
    })
}

pub fn decrypt_hpke_x25519_chacha20poly1305(
    envelope: &CryptoEnvelope,
    recipient_private_key: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let recipient_priv: [u8; 32] = recipient_private_key
        .try_into()
        .map_err(|_| CryptoError::InvalidPrivateKeyLength)?;
    let recipient_priv = StaticSecret::from(recipient_priv);

    if envelope.ephemeral_pub_key.is_empty() {
        return Err(CryptoError::MissingEphemeralPublicKey);
    }

    let eph_pub: [u8; 32] = envelope
        .ephemeral_pub_key
        .as_slice()
        .try_into()
        .map_err(|_| CryptoError::InvalidPublicKeyLength)?;
    let eph_pub = PublicKey::from(eph_pub);

    let shared = recipient_priv.diffie_hellman(&eph_pub);
    let (key, nonce) = derive_key_nonce(shared.as_bytes())?;
    let cipher = ChaCha20Poly1305::new(&key);
    cipher
        .decrypt(&nonce, envelope.ciphertext.as_slice())
        .map_err(|e| CryptoError::DecryptFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hpke_roundtrip() {
        let secret = StaticSecret::from([42u8; 32]);
        let public = PublicKey::from(&secret);
        let plaintext = b"nostra-hpke-roundtrip";

        let envelope = encrypt_hpke_x25519_chacha20poly1305(
            plaintext,
            public.as_bytes(),
            "test-key-id",
        )
        .expect("encrypt");

        let decrypted =
            decrypt_hpke_x25519_chacha20poly1305(&envelope, &secret.to_bytes()).expect("decrypt");

        assert_eq!(plaintext.to_vec(), decrypted);
    }
}
