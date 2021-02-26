use iced::button;
use iced::{Button, Container, Length, Row, Text};
use iced::{HorizontalAlignment, VerticalAlignment};

use crate::style;
use crate::clock::{AlarmTime, Time};
use crate::Message;

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

fn style_tomorrow<'a>(hour_min: &AlarmTime, usually: &Time) -> Container<'a, Message> {
    match (hour_min.inner(), usually) {
        (Some((h,m)),_) => {
            let time_txt = format!("{:02}:{:02}", h, m);
            let time_txt = Text::new(time_txt).size(70);
            Container::new(time_txt).style(hour_min)
        }
        (None, Some((h,m))) => {
            let time_txt = format!("{:02}:{:02}", h, m);
            let time_txt = Text::new(time_txt).size(70);
            Container::new(time_txt).style(style::PlaceHolder)
        }
        (None, None) => {
            let time_txt = String::from("--");
            let time_txt = Text::new(time_txt).size(70);
            Container::new(time_txt).style(hour_min)
        }
    }
}

pub fn clock<'a>(hour_min: &AlarmTime, usually: &Time, clear: &'a mut button::State)
-> Container<'a, Message> {

    let time_txt = style_tomorrow(hour_min, usually);
    let clear_txt = Text::new("x")
        .size(70)
        .vertical_alignment(VerticalAlignment::Center);
    let clear = Button::new(clear, clear_txt)
        .on_press(Message::Clear)
        .style(hour_min);

    match hour_min.inner() {
        Some(_) => Container::new(Row::new()
            .push(time_txt)
            .push(clear)
            .spacing(10)
            .align_items(iced::Align::Center)),
        None => 
            Container::new(time_txt),
    }
}

pub fn clock_button<'a>(
    hour_min: &AlarmTime,
    usually: &Time,
    edit: &'a mut button::State,
) -> Button<'a, Message> {
    let content = style_tomorrow(hour_min, usually);
    Button::new(edit, content)
        .on_press(Message::SwapEdit)
        .style(hour_min)
}

pub fn clock_title(title: &'static str) -> Text {
    Text::new(title)
        .width(Length::Fill)
        .horizontal_alignment(HorizontalAlignment::Center)
}

pub fn view_row(row: &mut [button::State], hour_mul: i8, min_mul: i8) -> Row<Message> {
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
