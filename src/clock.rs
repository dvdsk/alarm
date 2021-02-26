#[derive(Debug, Clone)]
pub enum Clocks {
    Tomorrow(AlarmTime),
    Usually(AlarmTime),
}

impl Clocks {
    pub fn inner_mut(&mut self) -> &mut AlarmTime {
        match self {
            Self::Tomorrow(a) => a,
            Self::Usually(a) => a,
        }
    }
    pub fn inner(&self) -> &AlarmTime {
        match self {
            Self::Tomorrow(a) => a,
            Self::Usually(a) => a,
        }
    }
    pub fn set_synced(mut self) -> Self {
        self.inner_mut().set_synced();
        self
    }

    pub fn set_none(&mut self) {
        *self.inner_mut() = AlarmTime::Set(None);
    }
}

pub type Time = Option<(u8,u8)>;
#[derive(Clone, Debug, PartialEq)]
pub enum AlarmTime {
    Set(Time),
    Synced(Time),
}

impl AlarmTime {
    pub fn set_synced(&mut self) {
        let t = *self.inner();
        *self = Self::Synced(t);
    }
    pub fn inner_mut(&mut self) -> &mut Time {
        match self {
            Self::Set(t) => t,
            Self::Synced(t) => t,
        }
    }

    pub fn inner(&self) -> &Time {
        match self {
            Self::Set(t) => t,
            Self::Synced(t) => t,
        }
    }

    pub fn inner_or_def(&mut self) -> (i8, i8) {
        self.inner_mut()
            .map(|(t1,t2)| (t1 as i8, t2 as i8))
            .unwrap_or((12,0))
    }

    pub fn adjust_min(&mut self, n: i8) {
        let (mut hour, mut min) = self.inner_or_def();
        min += n;
        hour += min / 60;
        min = i8::min(min % 60, 59);
        if min.is_negative() {
            min += 60;
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
    pub fn adjust_hour(&mut self, n: i8) {
        let (mut hour, min) = self.inner_or_def();
        hour += n;
        hour = Self::fix_hour(hour);
        *self = Self::Set(Some((hour as u8,min as u8)));
    }
}
