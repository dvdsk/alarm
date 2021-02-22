use iced::{Color, Vector};
use iced::{
    button, checkbox, container, progress_bar, radio, rule, scrollable,
    slider, text_input,
};
use crate::AlarmTime;

const APP_BACKGROUND: Color = Color::from_rgb(
    0x36 as f32 / 255.0,
    0x39 as f32 / 255.0,
    0x3F as f32 / 255.0,
);

const TIME_SET_TEXT: Color = Color::from_rgb(
    0x36 as f32 / 255.0,
    0x39 as f32 / 255.0,
    0x3F as f32 / 255.0,
);

const TIME_SYNCED_TEXT: Color = Color::from_rgb(
    0x36 as f32 / 255.0,
    0x39 as f32 / 255.0,
    0x3F as f32 / 255.0,
);

pub struct Theme;
impl container::StyleSheet for Theme {
    fn style(&self) -> container::Style {
        container::Style {
            background: APP_BACKGROUND.into(),
            text_color: Color::WHITE.into(),
            ..container::Style::default()
        }
    }
}
impl button::StyleSheet for Theme {
    fn active(&self) -> button::Style {
        button::Style {
            background: None,
            border_radius: 0.0,
            shadow_offset: Vector::new(0.0, 0.0),
            text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            text_color: Color::WHITE,
            shadow_offset: Vector::new(1.0, 2.0),
            ..self.active()
        }
    }
}

impl From<&AlarmTime> for Box<dyn button::StyleSheet> {
    fn from(time: &AlarmTime) -> Self {
        match time {
            AlarmTime::Set(_) => set::Theme.into(),
            AlarmTime::Synced(_) => synced::Theme.into(),
        }
    }
}

impl From<&AlarmTime> for Box<dyn container::StyleSheet> {
    fn from(time: &AlarmTime) -> Self {
        match time {
            AlarmTime::Set(_) => set::Theme.into(),
            AlarmTime::Synced(_) => synced::Theme.into(),
        }
    }
}

mod synced {
    use super::*;

    pub struct Theme;
    impl container::StyleSheet for Theme {
        fn style(&self) -> container::Style {
            container::Style {
                background: Color::from_rgb8(0x36, 0x39, 0x3F).into(),
                text_color: Color::WHITE.into(),
                ..container::Style::default()
            }
        }
    }

    impl button::StyleSheet for Theme {
        fn active(&self) -> button::Style {
            button::Style {
                background: Color::from_rgb(0.11, 0.42, 0.87).into(),
                border_radius: 0.0,
                shadow_offset: Vector::new(0.0, 0.0),
                text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                text_color: Color::WHITE,
                shadow_offset: Vector::new(1.0, 2.0),
                ..self.active()
            }
        }
    }
}

mod set {
    use super::*;

    pub struct Theme;
    impl container::StyleSheet for Theme {
        fn style(&self) -> container::Style {
            container::Style {
                background: Color::from_rgb8(0x36, 0x39, 0x3F).into(),
                text_color: Color::WHITE.into(),
                ..container::Style::default()
            }
        }
    }

    impl button::StyleSheet for Theme {
        fn active(&self) -> button::Style {
            button::Style {
                background: Color::from_rgb(0.11, 0.42, 0.87).into(),
                border_radius: 12.0,
                shadow_offset: Vector::new(1.0, 1.0),
                text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                text_color: Color::WHITE,
                shadow_offset: Vector::new(1.0, 2.0),
                ..self.active()
            }
        }
    }
}
