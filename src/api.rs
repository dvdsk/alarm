use iced::button;
use iced::{executor, Application, Command, Element, Settings};
use iced::{Button, Column, HorizontalAlignment, Length, Row, Space, Text};
use std::mem;

use super::{Message, Time, Clocks};

#[derive(thiserror::Error, Clone, Debug)]
pub enum Error {
    #[error("server error: {0}")]
    CouldNotConnect(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        let string = format!("{}", err);
        Self::CouldNotConnect(string)
    }
}



#[derive(Clone)]
pub struct Api {
    url: String,
    username: String,
    password: String,
}

impl Api {
    pub fn from(url: String, username: String, password: String) -> Self {
        Self {
            url,
            username,
            password,
        }
    }

    pub fn get_alarms(&self) -> Command<Message> {
        let api = self.clone();
        Command::perform(api.do_gets(),
            |res| match res {
                Ok((t1,t2)) => Message::RemoteAlarms(t1,t2),
                Err(e) => Message::RemoteError(e),
            })
    }

    pub fn sync(&self, clock: &Clocks) -> Command<Message> {
        let api = self.clone();
        let future = match clock {
            Clocks::Usually(t) => {
                let t = t.inner().unwrap();
                api.post("/alarm/usually", t)
            }
            Clocks::Tomorrow(t) => {
                let t = t.inner().unwrap();
                api.post("/alarm/tomorrow", t)
            }
        };

        let clock = clock.clone();
        Command::perform(future, move |res| {
            let clock = clock.clone();
            match res {
            Ok(_) => Message::Synced(clock),
            Err(e) => Message::RemoteError(e),
        }})
    }

    async fn do_gets(self) -> Result<(Time,Time), Error> {
        let client = reqwest::Client::new();

        let url = format!("{}/alarm/tomorrow", self.url);
        let tomorrow = client.get(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send().await?
            .error_for_status()?
            .bytes().await?;
        let tomorrow: Time = bincode::deserialize(&tomorrow).unwrap();

        let url = format!("{}/alarm/usually", self.url);
        let usually = client.get(&url)
            .basic_auth(self.username, Some(self.password))
            .send().await?
            .error_for_status()?
            .bytes().await?;
        let usually: Time = bincode::deserialize(&usually).unwrap();

        Ok((tomorrow, usually))
    }

    pub async fn post(self, endpoint: &str, time: (u8,u8)) -> Result<(), Error> {
        let url = format!("{}{}", self.url, endpoint);

        let body = bincode::serialize(&time).unwrap();
        let client = reqwest::Client::new();

        client.post(&url)
            .basic_auth(&self.username, Some(&self.password))
            .body(body)
            .send().await?
            .error_for_status()?;
        Ok(())
    }
}
