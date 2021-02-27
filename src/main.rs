use iced::button;
use iced::{executor, Application, Command, Element, Settings};
use iced::{Container, Column, Length};
use std::mem;

mod api;
mod style;
mod clock;
mod elements;
use clock::{Clocks, AlarmTime, Time};

#[derive(structopt::StructOpt)]
struct Args {
    /// base url to use, should have endpoints: 
    ///   url/next_alarm (GET POST)
    ///   url/usual_alarm (GET POST)
    url: String,

    /// http basic authentication username
    username: String,
    /// http basic authentication password
    password: String,
}

#[paw::main]
pub fn main(args: Args) {
    let settings = build_settings(args);
    Alarm::run(settings).unwrap();
}

fn build_settings(args: Args) -> Settings<Args> {
    Settings {
        window: iced::window::Settings::default(),
        flags: args,
        default_font: None,
        #[cfg(not(features = "pinephone"))]
        default_text_size: 70,
        #[cfg(features = "pinephone")]
        default_text_size: 70,
        antialiasing: false,
    }
}

struct Alarm {
    editing: Clocks,
    other: Clocks,
    edit_tomorrow: button::State,
    edit_usually: button::State,
    clear: button::State,
    buttons: [button::State; 12],
    api: api::Api,
}

#[derive(Debug, Clone)]
pub enum Message {
    Clear,
    AdjHour(i8),
    AdjMinute(i8),
    ReTryGetAlarms,
    SwapEdit,
    ReTrySync(Clocks),
    Synced(Clocks),
    RemoteAlarms(Time, Time),
    RemoteError(api::Error, Box<Message>),
}

async fn delay_update_inner(msg: Message) -> Message {
    use std::time::Duration;
    use tokio::time::sleep;

    sleep(Duration::from_millis(2000)).await;
    msg
}

fn delay_update(msg: Message) -> Command<Message> {
    Command::perform(delay_update_inner(msg), |msg| msg)
}

impl Application for Alarm {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Args;

    fn new(flags: Args) -> (Alarm, Command<Message>) {
        let Args {url, username, password} = flags;
        let api = api::Api::from(url, username, password);
        let alarm = Alarm {
            editing: Clocks::Tomorrow(AlarmTime::Set(None)),
            other: Clocks::Usually(AlarmTime::Set(None)),
            edit_tomorrow: button::State::default(),
            edit_usually: button::State::default(),
            clear: button::State::default(),
            buttons: [button::State::new(); 12],
            api: api.clone(),
        };
        (alarm, api.get_alarms())
    }

    fn title(&self) -> String {
        String::from("set wakeup time")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        use Message::*;
        match message {
            Clear => {
                self.editing.set_none();
                return self.api.sync(self.editing);
            }
            AdjHour(h) => {
                self.tomorrow_to_default_if_none();
                self.editing.inner_mut().adjust_hour(h);
                self.tomorrow_to_none_if_default();
                return self.api.sync(self.editing);
            }
            AdjMinute(m) => {
                self.tomorrow_to_default_if_none();
                self.editing.inner_mut().adjust_min(m);
                self.tomorrow_to_none_if_default();
                return self.api.sync(self.editing);
            }
            ReTryGetAlarms => return self.api.get_alarms(),
            ReTrySync(c) => {
                let c = self.current_clock(c);
                return self.api.sync(c);
            }
            SwapEdit => mem::swap(&mut self.editing, &mut self.other),
            Synced(clock) => self.set_synced(clock),
            RemoteAlarms(t1,t2) => self.set_remote_times(t1,t2),
            RemoteError(_, failed) => return delay_update(*failed),
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let Alarm {
            edit_tomorrow,
            edit_usually,
            buttons,
            clear,
            ..
        } = self;
        let (row1, row2, row3) = Self::borrow_rows(buttons);

        let column = Column::new().push(elements::clock_title("Tomorrow"));
        let column = match &self.editing {
            Clocks::Tomorrow(time) => column
                .push(elements::clock(&time, self.other.inner().inner(), clear))
                .push(elements::view_row(row1, 1, 1))
                .push(elements::view_row(row2, 3, 5))
                .push(elements::view_row(row3, 9, 15))
                .push(elements::clock_title("Usually"))
                .push(elements::clock_button(self.other.inner_mut(), &None, edit_usually))
                .align_items(iced::Align::Center),
            Clocks::Usually(time) => column
                .push(elements::clock_button(self.other.inner_mut(), &None, edit_tomorrow))
                .push(elements::clock_title("Usually"))
                .push(elements::clock(&time, &None, clear))
                .push(elements::view_row(row1, 1, 1))
                .push(elements::view_row(row2, 3, 5))
                .push(elements::view_row(row3, 9, 15))
                .align_items(iced::Align::Center),
        };

        Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(style::Theme)
            .into()
    }
    fn mode(&self) -> iced::window::Mode {
        #[cfg(features = "pinephone")]
        return iced::window::Mode::Fullscreen;
        #[cfg(not(features = "pinephone"))]
        return iced::window::Mode::Windowed;
    }
}

impl Alarm {
    fn borrow_rows(
        rows: &mut [button::State],
    ) -> (
        &mut [button::State],
        &mut [button::State],
        &mut [button::State],
    ) {
        let (row1, rest) = rows.split_at_mut(4);
        let (row2, row3) = rest.split_at_mut(4);
        (row1, row2, row3)
    }

    fn set_remote_times(&mut self, tomorrow: Time, usually: Time) {
        use Clocks::*;
        use AlarmTime::*;

        match self.editing {
            Tomorrow(_) => {
                self.editing = Tomorrow(Synced(tomorrow));
                self.other = Usually(Synced(usually));
            }
            Usually(_) => {
                self.editing = Usually(Synced(usually));
                self.other = Tomorrow(Synced(tomorrow));
            }
        }
    }

    fn current_clock(&mut self, clock: Clocks) -> Clocks {
        use Clocks::*;
        match (clock, &self.editing) {
            (Tomorrow(_), Tomorrow(_)) => self.editing,
            (Usually(_), Usually(_)) => self.editing,
            (Tomorrow(_), Usually(_)) => self.other,
            (Usually(_), Tomorrow(_)) => self.other,
        }
    }

    fn set_synced(&mut self, clock: Clocks) {
        use Clocks::*;

        let t3 = self.other.inner();
        match (&clock, &self.editing) {
            (Tomorrow(t1), Tomorrow(t2)) if t1 == t2 => 
                self.editing = clock.set_synced(),
            (Tomorrow(t1), Usually(_)) if t1 == t3 => 
                self.other = clock.set_synced(),
            (Usually(t1), Usually(t2)) if t1 == t2 => 
                self.editing = clock.set_synced(),
            (Usually(t1), Tomorrow(_)) if t1 == t3 =>
                self.other = clock.set_synced(),
            _ => (), // time setting has changed, not synced
        }
    }
    
    /// if the clock usually is set and tomorrow is not set the value
    /// of tomorrow to usually
    pub fn tomorrow_to_default_if_none(&mut self) {
        let (editing, default) = (&mut self.editing, self.other.inner().inner());
        if let (Clocks::Tomorrow(ref mut t1), Some(default)) = (editing, default) {
            if t1.inner().is_none() {
                *t1.inner_mut() = Some(*default);
            }
        }
    }

    pub fn tomorrow_to_none_if_default(&mut self) {
        let (editing, default) = (self.editing.inner().inner(), self.other.inner().inner());
        if editing == default {
            *self.editing.inner_mut().inner_mut() = None;
        }
    }
}
