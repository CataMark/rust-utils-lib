use crate::error::ErrorReport;
use data_encoding::BASE64URL_NOPAD;
use openssl::{
    pkey::{Private, Public},
    rsa::{Padding, Rsa},
};
use std::fs;

#[derive(Debug, Clone)]
pub struct RsaKeys {
    private: Rsa<Private>,
    public: Rsa<Public>,
}

impl RsaKeys {
    pub fn init(
        passphrase: &String,
        priv_key_path: &String,
        pub_key_path: &String,
    ) -> Result<RsaKeys, ErrorReport> {
        let priv_key_bytes = fs::read(priv_key_path)?;
        let pub_key_bytes = fs::read(pub_key_path)?;

        Ok(RsaKeys {
            private: Rsa::private_key_from_pem_passphrase(
                &priv_key_bytes[..],
                passphrase.as_bytes(),
            )?,
            public: Rsa::public_key_from_pem(&pub_key_bytes[..])?,
        })
    }

    pub fn get_private_key(&self) -> &Rsa<Private> {
        &self.private
    }

    pub fn get_public_key(&self) -> &Rsa<Public> {
        &self.public
    }

    pub fn pub_encrypt(&self, data: &String) -> Result<String, ErrorReport> {
        let mut buf = vec![0; self.public.size() as usize];
        let bytes = self
            .public
            .public_encrypt(data.as_bytes(), &mut buf, Padding::PKCS1)?;
        Ok(BASE64URL_NOPAD.encode(&buf[0..bytes]))
    }

    pub fn pub_decrypt(&self, data: &String) -> Result<String, ErrorReport> {
        let mut buf = vec![0; self.public.size() as usize];
        let bytes = self.public.public_decrypt(
            &BASE64URL_NOPAD.decode(data.as_bytes())?[..],
            &mut buf,
            Padding::PKCS1,
        )?;
        Ok(String::from_utf8(buf[0..bytes].to_vec())?)
    }

    pub fn priv_encrypt(&self, data: &String) -> Result<String, ErrorReport> {
        let mut buf = vec![0; self.private.size() as usize];
        let bytes = self
            .private
            .private_encrypt(data.as_bytes(), &mut buf, Padding::PKCS1)?;
        Ok(BASE64URL_NOPAD.encode(&buf[0..bytes]))
    }

    pub fn priv_decrypt(&self, data: &String) -> Result<String, ErrorReport> {
        let mut buf = vec![0; self.private.size() as usize];
        let bytes = self.private.private_decrypt(
            &BASE64URL_NOPAD.decode(data.as_bytes())?[..],
            &mut buf,
            Padding::PKCS1,
        )?;
        Ok(String::from_utf8(buf[0..bytes].to_vec())?)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        envars::{AppConfig, CONFIG_FILE_DELIMITER},
        rsakeys::RsaKeys,
    };
    use std::path::Path;

    #[test]
    fn rsa() {
        let root_dir = Path::new(env!("WORKSPACE_ROOT_PATH"));
        let config_path = Path::new(env!("APP_CONFIG_FILE_PATH"));
        let app_config = AppConfig::init(config_path, CONFIG_FILE_DELIMITER).unwrap();
        let rsa = RsaKeys::init(
            app_config.get_var("RSA:PASS").unwrap(),
            &root_dir
                .join(app_config.get_var("RSA:PRIV_KEY_PATH").unwrap())
                .to_str()
                .unwrap()
                .to_string(),
            &root_dir
                .join(app_config.get_var("RSA:PUB_KEY_PATH").unwrap())
                .to_str()
                .unwrap()
                .to_string(),
        )
        .unwrap();

        let text = String::from("Lorem ipsum dolor. sit amet, consectetur adipiscing elit.");

        let pub_cript = rsa.pub_encrypt(&text).unwrap();
        let priv_decrypt = rsa.priv_decrypt(&pub_cript).unwrap();
        assert_eq!(
            priv_decrypt, text,
            "Private decrypt: text not equal to input"
        );

        let priv_cript = rsa.priv_encrypt(&text).unwrap();
        let pub_decrypt = rsa.pub_decrypt(&priv_cript).unwrap();
        assert_eq!(pub_decrypt, text, "Public decrypt: text not equal to input");
    }
}
