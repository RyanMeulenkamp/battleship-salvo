#[macro_use]
#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use crate::messaging::translate::{
        serialize, encrypt, decrypt, verify, sign, TranslationError, Result
    };
    use lazy_static::lazy_static;
    use std::ops::Deref;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Test {
        a: String,
        c: String
    }

    lazy_static! {
        static ref DATA: Test = Test {a: String::from("b"), c: String::from("d")};
        static ref KEY: String = String::from("cockadoodledoo");
    }

    #[test]
    fn test_successful_crypt() {
        let json_data = serialize(DATA.deref()).expect("Unable to serialize!");
        let decrypted = decrypt(
            encrypt(&json_data, KEY.deref()), KEY.deref()).expect("Unable to decrypt!"
        );

        info!("Decrypted: {:?}", decrypted);
        assert_eq!(json_data, decrypted);
    }

    #[test]
    fn test_unsuccessful_crypt() {
        let json_data = serialize(DATA.deref()).expect("Unable to serialize!");
        let decrypted = decrypt(encrypt(&json_data, KEY.deref()), String::from("Cockadoodlesploo"));

        info!("Decrypted: {:?}", decrypted);
        assert!(match decrypted {
            Ok(_) => false,
            Err(error) => match error {
                TranslationError::Argon2Error(_) => true,
                _ => false
            }
        });
    }

    #[test]
    fn test_successful_sign() {
        let json_data = sign(DATA.deref(), KEY.deref()).expect("Unable to serialize!");
        let verified: Result<Test> = verify(&json_data, KEY.deref());

        info!("Verified: {:?}", verified);
        assert!(verified.is_ok());
    }

    #[test]
    fn test_unsuccessful_sign() {
        let json_data = sign(DATA.deref(), KEY.deref()).expect("Unable to serialize!");
        let verified: Result<Test> = verify(&json_data, String::from("Cockadoodlesploo"));

        info!("Verified: {:?}", verified);
        assert!(match verified {
            Ok(_) => false,
            Err(error) => match error {
                TranslationError::VerificationError(_) => true,
                _ => false
            }
        });
    }
}
