use std::string::FromUtf8Error;

use base64::DecodeError;
use simplecrypt::DecryptionError;
use serde::{Deserialize, Serialize};
use core::result;
use std::fmt::Debug;

#[derive(Debug)]
pub enum TranslationError {
    Argon2Error(DecryptionError),
    Base64Error(DecodeError),
    Utf8Error(FromUtf8Error),
    JsonError(serde_json::error::Error),
    VerificationError(String)
}

pub type Result<T> = result::Result<T, TranslationError>;

#[derive(Serialize, Deserialize)]
pub struct SignedMessage<Data: Serialize> {
    data: Data,
    sign: String
}

pub fn serialize<T>(input: &T) -> Result<String> where T: ?Sized + Serialize {
    serde_json::to_string(input).map_err(|err| TranslationError::JsonError(err))
}

pub fn deserialize<'a, T>(input: impl Into<&'a String>) -> Result<T> where T: Deserialize<'a> {
    serde_json::from_str(input.into().as_str()).map_err(|err| TranslationError::JsonError(err))
}

fn encode<T: AsRef<[u8]>>(input: T) -> String {
    base64::encode(input)
}

fn decode(input: impl Into<String>) -> Result<Vec<u8>> {
    base64::decode(input.into()).map_err(|err| TranslationError::Base64Error(err))
}

pub fn encrypt(data: impl Into<String>, key: impl Into<String>) -> String {
    encode(simplecrypt::encrypt(data.into().as_bytes(), key.into().as_bytes()))
}

pub fn sign<T: Serialize>(data: T, key: impl Into<String>) -> Result<String> {
    let json_data = serialize(&data)?;
    let message = SignedMessage {
        data: data, sign: encrypt(json_data, key)
    };
    serialize(&message)
}

pub fn decrypt(input: impl Into<String>, key: impl Into<String>) -> Result<String> {
    simplecrypt::decrypt(decode(input)?.as_ref(), key.into().as_bytes())
        .map_or_else(
            |err| Result::Err(TranslationError::Argon2Error(err)),
            |data| String::from_utf8(data).map_err(|err| TranslationError::Utf8Error(err))
        )
}

pub fn verify<'a, T>(signed_data: impl Into<&'a String>, key: impl Into<String>) -> Result<T> where T: Deserialize<'a> + ?Sized + Serialize {
    let signed_message: SignedMessage<T> = deserialize(signed_data)?;
    let serialized_data = serialize(&signed_message.data)?;
    let key: String = key.into();
    let decrypted_sign = decrypt(&signed_message.sign, &key)
        .map_err(|_| TranslationError::VerificationError(
            format!("Key {} incorrect for decrypting sign {}", key, &signed_message.sign)
        ))?;
    if decrypted_sign == serialized_data {
        Ok(signed_message.data)
    } else {
        Err(TranslationError::VerificationError(
            format!("Decrypted sign {} does not equal the data: {}", decrypted_sign, serialized_data)
        ))
    }
}
