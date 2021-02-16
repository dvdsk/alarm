use iced::{executor, Application, Command, Element, Settings};
use iced::button;
use iced::{Row, Column, Text, Space, Button};

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

impl Default for Clocks {
    fn default() -> Self {
        Self::Tomorrow(AlarmTime::default())
    }
}

#[derive(Default)]
struct AlarmTime(Option<(i8,i8)>);
impl AlarmTime {
    fn adjust_min(&mut self, n: i8) {
        let (mut hour, mut min) = self.0.unwrap_or((12,0));
        min = min + n;
        hour += min/60;
        min = i8::min(min % 60, 59);
        self.0.replace((hour,min));
    }
    fn adjust_hour(&mut self, n: i8) {
        let (mut hour, mut min) = self.0.unwrap_or((12,0));
        hour += n;
        hour += min/60;
        min = i8::min(min % 60, 59);
        self.0.replace((hour,min));
    }
}


#[derive(Default)]
struct Alarm {
    editing: Clocks,
    other: Clocks,
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

    fn update(&mut self, message: Message) -> Command<Message> {
        use Message::*;
        // match message {
        //     HourPlus(h),
        //     HourMinus(h),
        //     MinutePlus(m),
        //     MinuteMinus(m),
        //     EditTomorrow,
        //     EditUsually,
        // }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let Alarm {edit_tomorrow, edit_usually, buttons, ..} = self;
        let (row1, row2, row3) = Self::borrow_rows(buttons);
        match self.editing {
            Clocks::Tomorrow(time) => {
                Row::new()
                    .push(Text::new("Tomorrow"))
                    .push(clock(&time, 10, edit_tomorrow, Message::EditTomorrow))
                    .push(view_row(row1,1,1))
                    .push(view_row(row2,3,5))
                    .push(view_row(row3,9,15))
                    .push(Text::new("Usually"))
                    .push(clock(self.other.inner(),10, edit_usually, Message::EditUsually))
                    .into()
            }
            Clocks::Usually(time) => {
                Row::new()
                    .push(Text::new("Tomorrow"))
                    .push(clock(self.other.inner(),10, edit_tomorrow, Message::EditUsually))
                    .push(Text::new("Usually"))
                    .push(clock(&time, 10, edit_usually, Message::EditTomorrow))
                    .push(view_row(row1,1,1))
                    .push(view_row(row2,3,5))
                    .push(view_row(row3,9,15))
                    .into()
            }
        }
    }
}

fn view_row(row: &mut [button::State], hour_mul: u8, min_mul: u8) -> Column<Message> {
    use Message::*;
    let (mplus, rest) = row.split_first_mut().unwrap();
    let (mmin, rest) = rest.split_first_mut().unwrap();
    let (hplus, hmin) = rest.split_first_mut().unwrap();
    let hmin = &mut hmin[0];

    Column::new()
        .push(Button::new(mplus, Text::new("+")).on_press(MinutePlus(min_mul)))
        .push(Button::new(mmin, Text::new("-")).on_press(MinuteMinus(min_mul)))
        .push(Text::new(" "))
        .push(Button::new(hplus, Text::new("+")).on_press(HourPlus(hour_mul)))
        .push(Button::new(hmin, Text::new("-")).on_press(HourMinus(hour_mul)))
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

fn clock<'a>(hour_min: &AlarmTime, size: u16, edit: &'a mut button::State, msg: Message) -> Button<'a, Message> {
    let text = if let Some((hour, min)) = hour_min.0.as_ref() {
        format!("{:02}:{:02}", hour, min)
    } else {
        format!("00:00")
    };
    let text = Text::new(text).size(size);
    Button::new(edit, text)
        .on_press(msg)
}
