use iced::button;
use iced::{executor, Application, Command, Element, Settings};
use iced::{Button, Column, HorizontalAlignment, Length, Row, Space, Text};
use std::mem;

pub fn main() -> iced::Result {
    let settings = build_settings();
    Alarm::run(settings)
}

fn build_settings() -> Settings<()> {
    Settings {
        window: iced::window::Settings::default(),
        flags: (),
        default_font: None,
        #[cfg(not(features = "pinephone"))]
        default_text_size: 100,
        #[cfg(features = "pinephone")]
        default_text_size: 100,
        antialiasing: false,
    }
}

enum Clocks {
    Tomorrow(AlarmTime),
    Usually(AlarmTime),
}

impl Clocks {
    fn inner(&mut self) -> &mut AlarmTime {
        match self {
            Self::Tomorrow(a) => a,
            Self::Usually(a) => a,
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq)]
struct AlarmTime(Option<(i8, i8)>);
impl AlarmTime {
    fn adjust_min(&mut self, n: i8) {
        let (mut hour, mut min) = self.0.unwrap_or((12, 0));
        min = min + n;
        hour += min / 60;
        min = i8::min(min % 60, 59);
        if min.is_negative() {
            min = 60 + min;
            hour -= 1;
        }
        hour = Self::fix_hour(hour);
        self.0.replace((hour, min));
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
        let (mut hour, min) = self.0.unwrap_or((12, 0));
        hour += n;
        hour = Self::fix_hour(hour);
        self.0.replace((hour, min));
    }
}

use std::fmt;
impl fmt::Display for AlarmTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some((hour, min)) = self.0.as_ref() {
            write!(f, "{:02}:{:02}", hour, min)
        } else {
            write!(f, "00:00")
        }
    }
}

struct Alarm {
    editing: Clocks,
    other: Clocks,
    edit_tomorrow: button::State,
    edit_usually: button::State,
    buttons: [button::State; 12],
}

#[derive(Debug, Clone)]
enum Message {
    AdjHour(i8),
    AdjMinute(i8),
    SwapEdit,
}

impl Application for Alarm {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Alarm, Command<Message>) {
        let alarm = Alarm {
            editing: Clocks::Tomorrow(AlarmTime::default()),
            other: Clocks::Usually(AlarmTime::default()),
            edit_tomorrow: button::State::default(),
            edit_usually: button::State::default(),
            buttons: [button::State::new(); 12],
        };
        (alarm, Command::none())
    }

    fn title(&self) -> String {
        String::from("set wakeup time")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        use Message::*;
        match message {
            AdjHour(h) => self.editing.inner().adjust_hour(h),
            AdjMinute(m) => self.editing.inner().adjust_min(m),
            SwapEdit => mem::swap(&mut self.editing, &mut self.other),
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let Alarm {
            edit_tomorrow,
            edit_usually,
            buttons,
            ..
        } = self;
        let (row1, row2, row3) = Self::borrow_rows(buttons);
        match &self.editing {
            Clocks::Tomorrow(time) => Column::new()
                .push(clock_title("Tomorrow"))
                .push(clock(&time, 100))
                .push(view_row(row1, 1, 1))
                .push(view_row(row2, 3, 5))
                .push(view_row(row3, 9, 15))
                .push(clock_title("Usually"))
                .push(clock_button(self.other.inner(), 100, edit_usually))
                .align_items(iced::Align::Center)
                .into(),
            Clocks::Usually(time) => Column::new()
                .push(clock_title("Tomorrow"))
                .push(clock_button(self.other.inner(), 100, edit_tomorrow))
                .push(clock_title("Usually"))
                .push(clock(&time, 100))
                .push(view_row(row1, 1, 1))
                .push(view_row(row2, 3, 5))
                .push(view_row(row3, 9, 15))
                .align_items(iced::Align::Center)
                .into(),
        }
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
        .push(adjust_button(mplus, "+", AdjMinute(min_mul)))
        .push(adjust_button(mmin, "-", AdjMinute(-1 * min_mul)))
        .push(Text::new(" "))
        .push(adjust_button(hplus, "+", AdjHour(hour_mul)))
        .push(adjust_button(hmin, "-", AdjHour(-1 * hour_mul)))
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
    Button::new(state, text).on_press(msg)
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
}

fn clock(hour_min: &AlarmTime, size: u16) -> Text {
    let text = format!("{}", hour_min);
    Text::new(text)
        .size(size)
        .width(Length::Fill)
        .horizontal_alignment(HorizontalAlignment::Center)
}

fn clock_button<'a>(
    hour_min: &AlarmTime,
    size: u16,
    edit: &'a mut button::State,
) -> Button<'a, Message> {
    let text = format!("{}", hour_min);
    let text = Text::new(text).size(size);
    // .width(Length::Fill)
    // .horizontal_alignment(HorizontalAlignment::Center);
    Button::new(edit, text).on_press(Message::SwapEdit)
    // .horizontal_alignment(HorizontalAlignment::Center)
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
