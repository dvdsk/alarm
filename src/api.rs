use iced::Command;
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
                Err(e) => Message::RemoteError(e, Box::new(Message::ReTryGetAlarms)),
            })
    }

    pub fn sync(&self, clock: Clocks) -> Command<Message> {
        let api = self.clone();
        let future = match clock {
            Clocks::Usually(t) => {
                let t = t.inner();
                api.post("/alarm/usually", *t)
            }
            Clocks::Tomorrow(t) => {
                let t = t.inner();
                api.post("/alarm/tomorrow", *t)
            }
        };

        Command::perform(future, move |res| {
            match res {
            Ok(_) => Message::Synced(clock),
            Err(e) => Message::RemoteError(e, Box::new(Message::ReTrySync(clock))),
        }})
    }

    async fn do_gets(self) -> Result<(Time,Time), Error> {
        let client = reqwest::Client::new();

        let tomorrow = self.clone().get(&client, "/alarm/tomorrow");
        let usually = self.get(&client, "/alarm/usually");
        let (tomorrow, usually) = tokio::join!(tomorrow, usually);

        Ok((tomorrow?, usually?))
    }

    pub async fn get(self, client: &reqwest::Client, endpoint: &str) -> Result<Time, Error> {
        let url = format!("{}{}", self.url, endpoint);
        let bytes = client.get(&url)
            .basic_auth(self.username, Some(self.password))
            .timeout(std::time::Duration::from_secs(2))
            .send().await?
            .error_for_status()?
            .bytes().await?;
        let time: Time = bincode::deserialize(&bytes).unwrap();
        Ok(time)
    }

    pub async fn post(self, endpoint: &str, time: Time) -> Result<(), Error> {
        let url = format!("{}{}", self.url, endpoint);

        let body = bincode::serialize(&time).unwrap();
        let client = reqwest::Client::new();

        client.post(&url)
            .basic_auth(&self.username, Some(&self.password))
            .timeout(std::time::Duration::from_secs(2))
            .body(body)
            .send().await?
            .error_for_status()?;
        Ok(())
    }
}
