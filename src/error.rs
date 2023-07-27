use thiserror::Error;

#[derive(Error, Debug)]
pub enum ErrorReport {
    #[error("{:?} - {0}", .0.kind())]
    Io(#[from] std::io::Error),
    #[error("FromUtf8 - {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("MailHeaderContentType - {0}")]
    MailHeaderContentType(#[from] lettre::message::header::ContentTypeErr),
    #[error("MailTransportSmtp - {0}")]
    MailTransportSmtp(#[from] lettre::transport::smtp::Error),
    #[error("MailContent - {0}")]
    MailContent(#[from] lettre::error::Error),
    #[error("MailSentResponse - {}", .msg)]
    MailSentResponse { msg: String },
    #[error("FlexiLogger - {0}")]
    FlexiLogger(#[from] flexi_logger::FlexiLoggerError),
    #[error("OpenSslErrorStack - {0}")]
    OpenSslErrorStack(#[from] openssl::error::ErrorStack),
    #[error("DataEncodingDecode - {0}")]
    DataEncodingDecode(#[from] data_encoding::DecodeError),
}

pub fn error_mail_sent_response(msg: &(dyn ToString)) -> ErrorReport {
    ErrorReport::MailSentResponse {
        msg: msg.to_string(),
    }
}
