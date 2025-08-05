use once_cell::sync::OnceCell;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha3::Digest;
use std::fs;

pub static FILENAME: &'static str = "config.yaml";
pub static CONFIG: OnceCell<Config> = OnceCell::new();

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub geth_url: String,
    pub pg_url: String,
    pub eth_priv_key: String,
    pub uniswab: String,
    pub preferred_base_token: String,
    pub preferred_coin_token: String,
    pub minimum_out: f64,
}

impl Config {
    pub fn public_key_bytes(&self) -> [u8; 20] {
        let secp = Secp256k1::new();
        let priv_key_bytes = hex::decode(&self.eth_priv_key).unwrap();
        let secret_key =
            SecretKey::from_slice(&priv_key_bytes).expect("32 bytes, within curve order");
        let public_key: [u8; 65] =
            PublicKey::from_secret_key(&secp, &secret_key).serialize_uncompressed();
        let mut hasher = sha3::Keccak256::new();
        hasher.update(&public_key[1..]);
        hasher.finalize()[12..32].try_into().unwrap()
    }

    pub fn public_key(&self) -> String {
        hex::encode(self.public_key_bytes().to_vec())
    }
}
pub fn read_type<T>(filename: &str) -> T
where
    T: DeserializeOwned,
{
    let filepath = path(filename);
    let yaml = fs::read_to_string(&filepath)
        .unwrap_or_else(|err| panic!("{} -> {} {}", filename, &filepath, err));
    let obj: T = serde_yaml::from_str(&yaml).unwrap_or_else(|err| panic!("{} {}", &filepath, err));
    obj
}

pub fn path(filename: &str) -> String {
    std::path::Path::new(filename)
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}
