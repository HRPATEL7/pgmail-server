use crate::structs::Mail;
use anyhow::{Context, Ok, Result};
pub enum State {
    Fresh,
    Greeted,
    ReceivingRcpt(Mail),
    ReceivingData(Mail),
    Received(Mail),
}
pub struct StateMachine {
    state: State,
    ehlo_greeting: String,
    data: String,
}

impl StateMachine {
    pub const START: &'static [u8] = b"220 edgemail\r\n";
    pub const ACCEPTED: &'static [u8] = b"250 Ok\r\n";
    pub const ACCEPTED_DATA: &'static [u8] = b"250 Ok ACCEPTED DATA\r\n";
    pub const AUTH_ACCEPTED: &'static [u8] = b"235 Ok\r\n";
    pub const SEND_DATA_PLZ: &'static [u8] = b"354 End data with <CR><LF>.<CR><LF>\r\n";
    pub const KTHXBYE: &'static [u8] = b"221 3.0.0 closing connection - pgsmtp\r";
    pub const HOLD_YOUR_HORSES: &'static [u8] = &[];
    pub fn new(domain: &str) -> Self {
        let domain = domain;
        let ehlo_greeting = format!("220-{domain} ESMTP {domain} -pgsmtp");
        Self {
            state: State::Fresh,
            ehlo_greeting,
            data: String::from(""),
        }
    }
    pub fn command_handler(&mut self, raw_msg: &str) -> Result<&[u8]> {
        print!("{}", raw_msg);
        let mut msg = {
            if raw_msg != "\r\n" {
                raw_msg.split_whitespace()
            } else {
                "/r/r/r".split_whitespace()
            }
        };
        let command = msg.next().context("context")?.to_lowercase();
        let state = std::mem::replace(&mut self.state, State::Fresh);
        match (command.as_str(), state) {
            ("ehlo" | "helo", State::Fresh) => {
                self.state = State::Greeted;
                Ok("250 smtp.pgmail.com is readyy\r\n".as_bytes())
            },
            ("mail", State::Greeted) => {
                let from = msg.next().context("received empty MAIL")?;
                let from = from
                    .strip_prefix("FROM:")
                    .context("received incorrect MAIL")?;
                self.state = State::ReceivingRcpt(Mail {
                    from: from.to_string(),
                    ..Default::default()
                });
                Ok(StateMachine::ACCEPTED)
            },
            ("rcpt", State::ReceivingRcpt(mut mail)) => {
                let to = msg.next().context("received empty MAIL")?;
                let to = to.strip_prefix("TO:").context("received incorrect RCPT")?;
                if Self::legal_recipient(to) {
                    mail.to.push(to.to_string());
                } else {
                    print!("Illegal recipient: {to}")
                }
                self.state = State::ReceivingRcpt(mail);
                Ok(StateMachine::ACCEPTED)
            },
            ("data", State::ReceivingRcpt(mail)) => {
                self.state = State::ReceivingData(mail);
                Ok(StateMachine::SEND_DATA_PLZ)
            },
            ("subject:", State::ReceivingRcpt(_mail)) => {
                print!("hello");
                let sub = msg.next().context("received empty MAIL")?;
                let sub1 = sub
                    .strip_prefix("Subject:")
                    .context("received incorrect MAIL")?;
                self.state = State::ReceivingRcpt(Mail {
                    sub: "hello".to_string(),
                    ..Default::default()
                });
                Ok(StateMachine::HOLD_YOUR_HORSES)
            },
            ("/r/r", State::ReceivingData(mut _mail)) => {
                print!("{}", command);
                Ok(StateMachine::HOLD_YOUR_HORSES)
            },
            (_, State::ReceivingData(mut mail)) => {
                let resp = if raw_msg.ends_with(".\r\n") {
                    print!("{}\n", mail.sub);
                    StateMachine::ACCEPTED_DATA
                } else {
                    StateMachine::HOLD_YOUR_HORSES
                };
                mail.data += raw_msg;
                self.state = State::ReceivingData(mail);
                Ok(resp)
            },
            ("quit", _) => Ok(StateMachine::KTHXBYE),
            (_, _) => Ok("502 3.0.0 Unrecognized command\r\n".as_bytes()),
        }
    }
    fn legal_recipient(to: &str) -> bool {
        let to = to.to_lowercase();
        !to.contains("admin") && !to.contains("postmaster") && !to.contains("hostmaster")
    }
}
