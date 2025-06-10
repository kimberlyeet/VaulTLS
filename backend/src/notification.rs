use anyhow::anyhow;
use chrono::prelude::*;
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::message::{header, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use maud::{html, Markup};
use crate::data::enums::MailEncryption;
use crate::settings::Mail;
use crate::cert::Certificate;

/// A struct representing the message to be sent to the user
pub(crate) struct MailMessage<'a> {
    pub(crate) to: String,
    pub(crate) subject: String,
    pub(crate) username: String,
    pub(crate) certificate: &'a Certificate
}

/// Generates the HTML content of the email
fn generate_certificate_email(message: &MailMessage, instance_url: &str) -> Markup {
    let datetime_created_on = DateTime::from_timestamp(message.certificate.created_on / 1000, 0).unwrap();
    let datetime_valid_until = DateTime::from_timestamp(message.certificate.created_on / 1000, 0).unwrap();
    let created_on = datetime_created_on.format("%Y-%m-%d %H:%M:%S").to_string();
    let valid_until = datetime_valid_until.format("%Y-%m-%d %H:%M:%S").to_string();

    html! {
        style {
            r#"
            .container {
                font-family: Arial, sans-serif;
                max-width: 600px;
                margin: 20px auto;
                background-color: #ffffff;
                border-radius: 8px;
                box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
                overflow: hidden;
            }
            .header {
                background-color: #e3f2fd;
                padding: 15px;
                text-align: center;
                font-size: 24px;
                color: #1976d2;
            }
            .content {
                padding: 20px;
                background-color: #ffffff;
            }
            .details {
                background-color: #f5f5f5;
                padding: 15px;
                border-radius: 4px;
                margin-top: 20px;
            }
            "#
        }
        div class="container" {
            div class="header" {
                "VaulTLS"
            }
            div class="content" {
                p {
                    "Hey " (message.username) ","
                }
                p {
                    "a new client certificate is available for you in VaulTLS! You can find it here: "
                    a href=(instance_url) { (instance_url) }
                }
                div class="details" {
                    p { "Certificate details:" }
                    p { "username: " (message.username) }
                    p { "certificate_name: " (message.certificate.name) }
                    p { "created_on: " (created_on) }
                    p { "valid_until: " (valid_until) }
                }
            }
        }
    }
}

/// Sends an email to the user with the certificate details
pub(crate) fn notify(message: MailMessage, server: &Mail) -> Result<(), anyhow::Error>{
    if !server.is_valid() { return Err(anyhow!("Invalid mail server configuration"))}

    let html_content = generate_certificate_email(
        &message,
        "http://localhost:5173",
    ).into_string();


    let email = Message::builder()
        .from(server.from.parse()?)
        .to(message.to.parse()?)
        .subject(message.subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(format!("Hello {}, greetings from VaulTLS. A new certificate is available for you. You can find it here: {}", message.username, "http://localhost:5173")),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(html_content),
                ),
        )?;

    let mut mail_builder = match server.encryption {
        MailEncryption::None => SmtpTransport::builder_dangerous(server.smtp_host.clone()).port(server.smtp_port),
        MailEncryption::TLS => {
            let param = TlsParameters::new(server.smtp_host.clone())?;
            SmtpTransport::builder_dangerous(server.smtp_host.clone()).port(server.smtp_port).tls(Tls::Wrapper(param))
        },
        MailEncryption::STARTTLS => {
            let param = TlsParameters::new(server.smtp_host.clone())?;
            SmtpTransport::builder_dangerous(server.smtp_host.clone()).port(server.smtp_port).tls(Tls::Required(param))
        }
    };

    if server.username.is_some() && server.password.is_some() {
        let cred = Credentials::new(server.username.clone().unwrap(), server.password.clone().unwrap());
        mail_builder = mail_builder.credentials(cred);
    }

    let mailer = mail_builder.build();

    println!("Sending email to {:?}", mailer);

    mailer.send(&email)?;
    Ok(())
}