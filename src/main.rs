use iced::button;
use iced::{executor, Application, Command, Element, Settings};
use iced::{Button, Container, Column, Length, Row, Space, Text};
use iced::{HorizontalAlignment, VerticalAlignment};
use std::mem;

mod api;
mod style;

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

#[derive(Debug, Clone)]
pub enum Clocks {
    Tomorrow(AlarmTime),
    Usually(AlarmTime),
}

impl Clocks {
    fn into_inner(self) -> AlarmTime {
        match self {
            Self::Tomorrow(a) => a,
            Self::Usually(a) => a,
        }
    }
    fn inner_mut(&mut self) -> &mut AlarmTime {
        match self {
            Self::Tomorrow(a) => a,
            Self::Usually(a) => a,
        }
    }
    fn inner(&self) -> &AlarmTime {
        match self {
            Self::Tomorrow(a) => a,
            Self::Usually(a) => a,
        }
    }
    fn set_synced(mut self) -> Self {
        self.inner_mut().set_synced();
        self
    }

    fn set_none(&mut self) {
        *self.inner_mut() = AlarmTime::Set(None);
    }
}

type Time = Option<(u8,u8)>;
#[derive(Clone, Debug, PartialEq)]
pub enum AlarmTime {
    Set(Time),
    Synced(Time),
}

impl AlarmTime {
    fn set_synced(&mut self) {
        let t = self.inner().clone();
        *self = Self::Synced(t);
    }
    fn inner_mut(&mut self) -> &mut Time {
        match self {
            Self::Set(t) => t,
            Self::Synced(t) => t,
        }
    }

    fn inner(&self) -> &Time {
        match self {
            Self::Set(t) => t,
            Self::Synced(t) => t,
        }
    }

    fn inner_or_def(&mut self) -> (i8, i8) {
        self.inner_mut()
            .map(|(t1,t2)| (t1 as i8, t2 as i8))
            .unwrap_or((12,0))
    }

    fn adjust_min(&mut self, n: i8) {
        let (mut hour, mut min) = self.inner_or_def();
        min = min + n;
        hour += min / 60;
        min = i8::min(min % 60, 59);
        if min.is_negative() {
            min = 60 + min;
            hour -= 1;
        }
        hour = Self::fix_hour(hour);
        *self = Self::Set(Some((hour as u8,min as u8)));
    }
    fn fix_hour(hour: i8) -> i8 {
        if hour > 23 {
            hour - 24
        } else if hour.is_negative() {
            24 + hour
        } else {
            hour
        }
    }
    fn adjust_hour(&mut self, n: i8) {
        let (mut hour, min) = self.inner_or_def();
        hour += n;
        hour = Self::fix_hour(hour);
        *self = Self::Set(Some((hour as u8,min as u8)));
    }
}

use std::fmt;
impl fmt::Display for AlarmTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some((hour, min)) = self.inner() {
            write!(f, "{:02}:{:02}", hour, min)
        } else {
            write!(f, "--:--")
        }
    }
}

struct Alarm {
    editing: Clocks,
    other: Clocks,
    edit_tomorrow: button::State,
    edit_usually: button::State,
    clear: button::State,
    buttons: [button::State; 12],
    error: Option<String>,
    api: api::Api,
}

#[derive(Debug, Clone)]
pub enum Message {
    AdjHour(i8),
    AdjMinute(i8),
    SwapEdit,
    ClearEdit,
    Synced(Clocks),
    RemoteAlarms(Time, Time),
    RemoteError(api::Error),
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
            error: None,
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
            AdjHour(h) => {
                self.editing.inner_mut().adjust_hour(h);
                return self.api.sync(&self.editing);
            }
            AdjMinute(m) => {
                self.editing.inner_mut().adjust_min(m);
                return self.api.sync(&self.editing);
            }
            SwapEdit => mem::swap(&mut self.editing, &mut self.other),
            ClearEdit => self.editing.set_none(),
            Synced(clock) => self.set_synced(clock),
            RemoteAlarms(t1,t2) => self.set_remote_times(t1,t2),
            RemoteError(e) => self.error = Some(e.to_string()),
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let Alarm {
            edit_tomorrow,
            edit_usually,
            buttons,
            error,
            clear,
            ..
        } = self;
        let (row1, row2, row3) = Self::borrow_rows(buttons);
        
        let column = match error {
            None => Column::new(),
            Some(msg) => Column::new().push(error_text(msg)),
        };

        let column = column.push(clock_title("Tomorrow"));
        let column = match &self.editing {
            Clocks::Tomorrow(time) => column
                .push(clock(&time, clear))
                .push(view_row(row1, 1, 1))
                .push(view_row(row2, 3, 5))
                .push(view_row(row3, 9, 15))
                .push(clock_title("Usually"))
                .push(clock_button(self.other.inner_mut(), 70, edit_usually))
                .align_items(iced::Align::Center),
            Clocks::Usually(time) => column
                .push(clock_button(self.other.inner_mut(), 70, edit_tomorrow))
                .push(clock_title("Usually"))
                .push(clock(&time, clear))
                .push(view_row(row1, 1, 1))
                .push(view_row(row2, 3, 5))
                .push(view_row(row3, 9, 15))
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

fn clock_title(title: &'static str) -> Text {
    Text::new(title)
        .width(Length::Fill)
        .horizontal_alignment(HorizontalAlignment::Center)
}

fn view_row(row: &mut [button::State], hour_mul: i8, min_mul: i8) -> Row<Message> {
    use Message::*;
    let (mplus, rest) = row.split_first_mut().unwrap();
    let (mmin, rest) = rest.split_first_mut().unwrap();
    let (hplus, hmin) = rest.split_first_mut().unwrap();
    let hmin = &mut hmin[0];

    Row::new()
        .push(adjust_button(hplus, "+", AdjHour(hour_mul)))
        .push(adjust_button(hmin, "-", AdjHour(-1 * hour_mul)))
        .push(Text::new(" "))
        .push(adjust_button(mplus, "+", AdjMinute(min_mul)))
        .push(adjust_button(mmin, "-", AdjMinute(-1 * min_mul)))
        .align_items(iced::Align::Center)
}

fn adjust_button<'a>(
    state: &'a mut button::State,
    c: &'static str,
    msg: Message,
) -> Button<'a, Message> {
    let text = Text::new(c)
        .width(Length::Fill)
        .horizontal_alignment(HorizontalAlignment::Center);
    Button::new(state, text).on_press(msg).style(style::Theme)
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
}

fn error_text<'a>(msg: &String) -> Container<'a, Message> {
    let text = Text::new(msg.clone())
        .size(20)
        .width(Length::Fill)
        .horizontal_alignment(HorizontalAlignment::Center);
    Container::new(text)
        .style(style::Error)
}

fn clock<'a>(hour_min: &AlarmTime, clear: &'a mut button::State) -> Container<'a, Message> {
    let time_txt = format!("{}", hour_min);
    let time_txt = Text::new(time_txt)
        .size(70);
    let time_txt = Container::new(time_txt)
        .style(hour_min);

    let clear_txt = Text::new("x")
        .size(70)
        .vertical_alignment(VerticalAlignment::Center);
    let clear = Button::new(clear, clear_txt)
        .on_press(Message::ClearEdit)
        .style(hour_min);

    let row = Row::new()
        .push(time_txt)
        .push(clear)
        .spacing(10)
        .align_items(iced::Align::Center);
    Container::new(row)
}

fn clock_button<'a>(
    hour_min: &AlarmTime,
    size: u16,
    edit: &'a mut button::State,
) -> Button<'a, Message> {
    let text = format!("{}", &hour_min);
    let text = Text::new(text).size(size);
    Button::new(edit, text)
        .on_press(Message::SwapEdit)
        .style(hour_min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_cases() {
        let mut hour_min = AlarmTime(Some((0i8, 0i8)));
        hour_min.adjust_min(-1i8);
        assert_eq!(String::from("23:59"), format!("{}", hour_min));

        let mut hour_min = AlarmTime(Some((23i8, 59i8)));
        hour_min.adjust_min(1i8);
        assert_eq!(String::from("00:00"), format!("{}", hour_min));
    }

    #[test]
    fn symmetry() {
        for m in 0..59i8 {
            for h in 0..23i8 {
                for i in &[1, 3, 9i8] {
                    let org = AlarmTime(Some((h, m)));
                    let mut hm = org.clone();
                    hm.adjust_hour(*i);
                    hm.adjust_hour(-1 * i);
                    assert_eq!(
                        org, hm,
                        "symmetry test failed for h: {}, m:{} and i: {}",
                        h, m, i
                    );
                }
                for i in &[1, 5, 15i8] {
                    let org = AlarmTime(Some((h, m)));
                    let mut hm = org.clone();
                    hm.adjust_min(*i);
                    hm.adjust_min(-1 * i);
                    assert_eq!(
                        org, hm,
                        "symmetry test failed for h: {}, m:{} and i: {}",
                        h, m, i
                    );
                }
            }
        }
    }
}
