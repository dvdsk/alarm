use iced::{executor, Application, Command, Element, Settings};
use iced::button;
use iced::{Row, Column, Text, Space, Button};
use std::mem;

pub fn main() -> iced::Result {
    Alarm::run(Settings::default())
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
struct AlarmTime(Option<(i8,i8)>);
impl AlarmTime {
    fn adjust_min(&mut self, n: i8) {
        let (mut hour, mut min) = self.0.unwrap_or((12,0));
        min = min + n;
        hour += min/60;
        min = i8::min(min % 60, 59);
        if min.is_negative() {
            min= 60 + min;
            hour -= 1;
        }
        hour = Self::fix_hour(hour);
        self.0.replace((hour,min));
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
        let (mut hour, min) = self.0.unwrap_or((12,0));
        hour += n;
        hour = Self::fix_hour(hour);
        self.0.replace((hour,min));
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
        String::from("A cool application")
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
        let Alarm {edit_tomorrow, edit_usually, buttons, ..} = self;
        let (row1, row2, row3) = Self::borrow_rows(buttons);
        match &self.editing {
            Clocks::Tomorrow(time) => {
                Column::new()
                    .push(Text::new("Tomorrow"))
                    .push(clock(&time, 20))
                    .push(view_row(row1,1,1))
                    .push(view_row(row2,3,5))
                    .push(view_row(row3,9,15))
                    .push(Text::new("Usually"))
                    .push(clock_button(self.other.inner(),20, edit_usually))
                    .into()
            }
            Clocks::Usually(time) => {
                Column::new()
                    .push(Text::new("Tomorrow"))
                    .push(clock_button(self.other.inner(),20, edit_tomorrow))
                    .push(Text::new("Usually"))
                    .push(clock(&time, 20))
                    .push(view_row(row1,1,1))
                    .push(view_row(row2,3,5))
                    .push(view_row(row3,9,15))
                    .into()
            }
        }
    }
}

fn view_row(row: &mut [button::State], hour_mul: i8, min_mul: i8) -> Row<Message> {
    use Message::*;
    let (mplus, rest) = row.split_first_mut().unwrap();
    let (mmin, rest) = rest.split_first_mut().unwrap();
    let (hplus, hmin) = rest.split_first_mut().unwrap();
    let hmin = &mut hmin[0];

    Row::new()
        .push(Button::new(mplus, Text::new("+")).on_press(AdjMinute(min_mul)))
        .push(Button::new(mmin, Text::new("-")).on_press(AdjMinute(-1*min_mul)))
        .push(Text::new(" "))
        .push(Button::new(hplus, Text::new("+")).on_press(AdjHour(hour_mul)))
        .push(Button::new(hmin, Text::new("-")).on_press(AdjHour(-1*hour_mul)))
}

impl Alarm {
    fn borrow_rows(rows: &mut [button::State]) -> (&mut [button::State],&mut [button::State],&mut [button::State]) {
        let (row1, rest) = rows.split_at_mut(4);
        let (row2, row3) = rest.split_at_mut(4);
        (row1,row2,row3)
    }
}

fn clock(hour_min: &AlarmTime, size: u16) -> Text {
    let text = format!("{}", hour_min);
    let text = Text::new(text).size(size);
    text
}
fn clock_button<'a>(hour_min: &AlarmTime, size: u16, edit: &'a mut button::State) -> Button<'a, Message> {
    let text = format!("{}", hour_min);
    let text = Text::new(text).size(size);
    Button::new(edit, text)
        .on_press(Message::SwapEdit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_cases() {
        let mut hour_min = AlarmTime(Some((0i8,0i8)));
        hour_min.adjust_min(-1i8);
        assert_eq!(String::from("23:59"), format!("{}", hour_min));

        let mut hour_min = AlarmTime(Some((23i8,59i8)));
        hour_min.adjust_min(1i8);
        assert_eq!(String::from("00:00"), format!("{}", hour_min));
    }

    #[test]
    fn symmetry() {
        for m in 0..59i8 {
            for h in 0..23i8 {
                for i in &[1,3,9i8] {
                    let org = AlarmTime(Some((h,m)));
                    let mut hm = org.clone();
                    hm.adjust_hour(*i);
                    hm.adjust_hour(-1*i);
                    assert_eq!(org, hm,
                        "symmetry test failed for h: {}, m:{} and i: {}", h,m,i);
                }
                for i in &[1,5,15i8] {
                    let org = AlarmTime(Some((h,m)));
                    let mut hm = org.clone();
                    hm.adjust_min(*i);
                    hm.adjust_min(-1*i);
                    assert_eq!(org, hm,
                        "symmetry test failed for h: {}, m:{} and i: {}", h,m,i);
                }
            }
        }
    }
}
