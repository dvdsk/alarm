use iced::{executor, Application, Command, Element, Settings};
use iced::button::{self, Button};
use iced::{Row, Column, Text, Space};

pub fn main() -> iced::Result {
    Alarm::run(Settings::default())
}

enum Editing {
    Tomorrow,
    Usually,
}

impl Default for Editing {
    fn default() -> Self {
        Self::Tomorrow
    }
}

#[derive(Default)]
struct Alarm {
    tomorrow: Option<(u8,u8)>,
    usually: Option<(u8,u8)>,
    current: Editing,
    edit_tomorrow: button::State,
    edit_usually: button::State,
    buttons: [button::State; 12],
}

#[derive(Debug, Clone)]
enum Message {
    HourPlus(u8),
    HourMinus(u8),
    MinutePlus(u8),
    MinuteMinus(u8),
    EditTomorrow,
    EditUsually,
}

impl Application for Alarm {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Alarm, Command<Message>) {
        (Alarm::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("A cool application")
    }

    fn update(&mut self, _message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let Alarm {tomorrow, usually, current, edit_tomorrow, edit_usually, buttons} = self;
        let (row1, row2, row3) = Self::borrow_rows(buttons);
        match self.current {
            Editing::Tomorrow => {
                Row::new()
                    .push(Text::new("Tomorrow"))
                    .push(clock(self.tomorrow, 10, edit_tomorrow))
                    .push(view_row(row1,1,1))
                    .push(view_row(row2,3,5))
                    .push(view_row(row3,9,15))
                    .push(Text::new("Usually"))
                    .push(clock(self.usually,10, edit_usually))
                    .into()
            }
            Editing::Usually => {
                Row::new()
                    .push(Text::new("Tomorrow"))
                    .push(clock(self.tomorrow,10, edit_tomorrow))
                    .push(Text::new("Usually"))
                    .push(clock(self.usually, 10, edit_usually))
                    .push(view_row(row1,1,1))
                    .push(view_row(row2,3,5))
                    .push(view_row(row3,9,15))
                    .into()
            }
        }
    }
}

fn view_row(row: &mut [button::State], hour_mul: u8, min_mul: u8) -> Column<Message> {
    let (mplus, rest) = row.split_first_mut().unwrap();
    let (mmin, rest) = rest.split_first_mut().unwrap();
    let (hplus, hmin) = rest.split_first_mut().unwrap();
    let hmin = &mut hmin[0];

    Column::new()
        .push(Button::new(mplus, Text::new("+")))
        .push(Button::new(mmin, Text::new("-")))
        .push(Text::new(" "))
        .push(Button::new(hplus, Text::new("+")))
        .push(Button::new(hmin, Text::new("-")))
}

impl Alarm {
    fn borrow_rows(rows: &mut [button::State]) -> (&mut [button::State],&mut [button::State],&mut [button::State]) {
        let (row1, rest) = rows.split_at_mut(4);
        let (row2, row3) = rest.split_at_mut(4);
        (row1,row2,row3)
    }
}

fn space() -> Text {
    Text::new(" ")
}
// fn plus() -> Button<Message> {
//     Text::new("+").into()
// }
// fn min() -> Button<Message> {
//     Text::new("-")
// }

fn clock(hour_min: Option<(u8,u8)>, size: u16, edit: &mut button::State) -> Button<Message> {
    let text = if let Some((hour, min)) = hour_min {
        format!("{:02}:{:02}", hour, min)
    } else {
        format!("00:00")
    };
    Button::new(Text::new(text).size(size), edit)
}
