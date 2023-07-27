use crate::error::ErrorReport;
use lettre::{
    message::{Attachment, Mailbox, MultiPart, SinglePart},
    transport::smtp::{authentication::Credentials, response::Severity},
    Message, SmtpTransport, Transport,
};
use std::{fs, path::Path};

#[derive(Debug)]
pub struct Config {
    pub from_addrs: Mailbox,
    pub reply_to: Mailbox,
    pub server: String,
    pub port: u16,
    pub user_name: String,
    pub password: String,
    pub template_dir_path: String,
    pub template_name_format: String,
    pub languages: Vec<String>,
    pub default_language: String,
}

#[derive(Debug)]
pub struct MailAttachment<'a> {
    pub path: &'a str,
    pub name: &'a str,
    pub mime: &'a str,
}

#[derive(Debug)]
pub struct Mailer {
    config: Config,
}

impl Mailer {
    pub fn init(config: Config) -> Self {
        Mailer { config }
    }

    pub fn send(
        &self,
        to_addrs: Vec<Mailbox>,
        cc_addrs: Option<Vec<Mailbox>>,
        subject: &String,
        message: &String,
        language: Option<&String>,
        attachments: Option<Vec<MailAttachment>>,
    ) -> Result<(), ErrorReport> {
        let html_body = |message: &String,
                         language: Option<&String>,
                         default_lang: &String,
                         template_dir_path: &String,
                         template_name_format: &String|
         -> Result<SinglePart, ErrorReport> {
            let lang = match language {
                Some(val) => val,
                None => default_lang,
            };
            let path =
                Path::new(template_dir_path).join(template_name_format.replace("{lang}", lang));
            let template_text = fs::read_to_string(path)?;
            let body = template_text.replace("{{contents}}", message);
            Ok(SinglePart::html(body))
        };

        let attachement_part = |attachment: &MailAttachment| -> Result<SinglePart, ErrorReport> {
            Ok(Attachment::new(attachment.name.to_owned())
                .body(fs::read(attachment.path)?, attachment.mime.parse()?))
        };

        let transport = |server: &String,
                         port: &u16,
                         user_name: &String,
                         password: &String|
         -> Result<SmtpTransport, ErrorReport> {
            Ok(SmtpTransport::starttls_relay(server)?
                .port(*port)
                .credentials(Credentials::new(user_name.clone(), password.clone()))
                .build())
        };

        let mut builder = Message::builder()
            .from(self.config.from_addrs.clone())
            .reply_to(self.config.reply_to.clone())
            .subject(subject);

        for addr in to_addrs {
            builder = builder.to(addr);
        }

        if let Some(addrs) = cc_addrs {
            for addr in addrs {
                builder = builder.cc(addr);
            }
        }

        let mut part = MultiPart::mixed().singlepart(html_body(
            message,
            language,
            &self.config.default_language,
            &self.config.template_dir_path,
            &self.config.template_name_format,
        )?);

        if let Some(attchs) = attachments {
            for attch in attchs {
                part = part.singlepart(attachement_part(&attch)?);
            }
        }

        let mail = builder.multipart(part)?;
        let res = transport(
            &self.config.server,
            &self.config.port,
            &self.config.user_name,
            &self.config.password,
        )?
        .send(&mail)?;

        match res.code().severity {
            Severity::PositiveCompletion => Ok(()),
            _ => Err(crate::error::error_mail_sent_response(
                &res.message().fold(String::new(), |t, s| t + s + "\n"),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, MailAttachment, Mailer};
    use crate::envars::{AppConfig, CONFIG_FILE_DELIMITER};
    use lettre::{message::Mailbox, Address};
    use std::path::Path;

    #[test]
    fn send_mail() {
        let root_dir = Path::new(env!("WORKSPACE_ROOT_PATH"));
        let config_path = Path::new(env!("APP_CONFIG_FILE_PATH"));
        let app_config = AppConfig::init(config_path, CONFIG_FILE_DELIMITER).unwrap();
        let config = Config {
            from_addrs: {
                let text = app_config.get_var("MAIL:FROM_ADDRS").unwrap();
                let mut split = text.split('@');
                Mailbox::new(
                    app_config
                        .get_var("MAIL:FROM_NAME")
                        .map(|val| val.to_owned()),
                    Address::new(split.next().unwrap(), split.next().unwrap()).unwrap(),
                )
            },
            reply_to: {
                let text = app_config.get_var("MAIL:REPLY_TO").unwrap();
                let mut split = text.split('@');
                Mailbox::new(
                    app_config
                        .get_var("MAIL:FROM_NAME")
                        .map(|val| val.to_owned()),
                    Address::new(split.next().unwrap(), split.next().unwrap()).unwrap(),
                )
            },
            server: app_config.get_var("MAIL:SMTP_SERVER").unwrap().to_owned(),
            port: app_config
                .get_var("MAIL:SMTP_PORT")
                .unwrap()
                .parse()
                .unwrap(),
            user_name: app_config.get_var("MAIL:SMTP_USER").unwrap().to_owned(),
            password: app_config.get_var("MAIL:SMTP_PASS").unwrap().to_owned(),
            template_dir_path: root_dir
                .join(app_config.get_var("MAIL:TEMPLATE_DIR").unwrap())
                .to_str()
                .unwrap()
                .to_owned(),
            template_name_format: app_config
                .get_var("MAIL:TEMPLATE_NAME_FORMAT")
                .unwrap()
                .to_owned(),
            languages: {
                let text = app_config.get_var("MAIL:LANGS").unwrap();
                let res: Vec<String> = text
                    .split(',')
                    .map(|val| val.trim().to_lowercase())
                    .filter(|val| !val.is_empty())
                    .collect();
                if res.is_empty() {
                    panic!("No list of mail languages was provided");
                }
                res
            },
            default_language: app_config
                .get_var("MAIL:LANG_DEFAULT")
                .unwrap()
                .to_lowercase(),
        };

        let mut to_addrs = Vec::new();
        to_addrs.push(Mailbox::new(
            Some("Catalin Mutica".to_owned()),
            Address::new("cmutica", "artemobinternational.com").unwrap(),
        ));
        to_addrs.push(Mailbox::new(
            Some("Catalin Mark".to_owned()),
            Address::new("catalin.mark", "gmail.com").unwrap(),
        ));

        let mut attachments = Vec::new();
        attachments.push(MailAttachment {
            path: &config_path.to_str().unwrap(),
            name: "config.txt",
            mime: "text/plain",
        });
        let cargo_lock_path = (&root_dir).join("Cargo.lock");
        attachments.push(MailAttachment {
            path: &cargo_lock_path.to_str().unwrap(),
            name: "Cargo.lock",
            mime: "text/plain",
        });

        let res = Mailer::init(config).send(
            to_addrs,
            None,
            &"Testare".to_owned(),
            &"Rust is the best".to_owned(),
            Some(&"ro".to_owned()),
            Some(attachments),
        );
        assert!(res.is_ok(), "Error: {}", res.err().unwrap());
    }
}
