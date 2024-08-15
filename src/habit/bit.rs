use std::collections::HashMap;
use std::default::Default;

use chrono::{Date, Datelike, Duration, Local, NaiveDate, NaiveTime, TimeZone};
use serde::{Deserialize, Serialize};

use crate::command::GoalKind;
use crate::habit::prelude::default_auto;
use crate::habit::traits::Habit;
use crate::habit::{InnerData, TrackEvent};
use crate::CONFIGURATION;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct CustomBool(bool);

use std::fmt;
impl fmt::Display for CustomBool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:^3}",
            if self.0 {
                CONFIGURATION.look.true_chr
            } else {
                CONFIGURATION.look.false_chr
            }
        )
    }
}

impl From<bool> for CustomBool {
    fn from(b: bool) -> Self {
        CustomBool(b)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bit {
    name: String,
    stats: HashMap<NaiveDate, CustomBool>,
    goal: CustomBool,

    #[serde(default = "default_auto")]
    auto: bool,

    #[serde(skip)]
    inner_data: InnerData,
}

impl Bit {
    pub fn new(name: impl AsRef<str>, auto: bool) -> Self {
        return Bit {
            name: name.as_ref().to_owned(),
            stats: HashMap::new(),
            goal: CustomBool(false),
            auto,
            inner_data: Default::default(),
        };
    }
}

impl Habit for Bit {
    type HabitType = CustomBool;
    fn name(&self) -> String {
        return self.name.clone();
    }
    fn set_name(&mut self, n: impl AsRef<str>) {
        self.name = n.as_ref().to_owned();
    }
    fn kind(&self) -> GoalKind {
        GoalKind::Bit
    }
    fn set_goal(&mut self, g: Self::HabitType) {
        self.goal = g;
    }
    fn get_by_date(&self, date: NaiveDate) -> Option<&Self::HabitType> {
        self.stats.get(&date)
    }
    fn insert_entry(&mut self, date: NaiveDate, val: Self::HabitType) {
        *self.stats.entry(date).or_insert(val) = val;
    }

    fn backfill(&mut self) {
        // Loop through them all and if a date is missing,
        // then set it to false
        let dates: Vec<&NaiveDate> = self.stats.keys().collect();

        // If there are no dates, then there's nothing to backfill
        if dates.is_empty(){
            return;
        }

        let mut target_date: NaiveDate = *dates.get(0).unwrap().clone();
        let tyy = Local::today().year();
        let tmm = Local::today().month();
        let tdd = Local::today().day();
        let today: NaiveDate = NaiveDate::from_ymd(tyy, tmm, tdd);

        while target_date < today {
            // Insert false for a date that isn't already in the history
            if self.stats.get(&target_date).is_some() {
                target_date += Duration::days(1);
                continue
            }
            self.insert_entry(target_date.clone(), CustomBool(false));
            target_date += Duration::days(1);
        }
    }

    fn rename(&mut self, new_name: &str) {
        self.name = String::from(new_name);
    }

    fn reached_goal(&self, date: NaiveDate) -> bool {
        if let Some(val) = self.stats.get(&date) {
            if val.0 >= self.goal.0 {
                return true;
            }
        }
        return false;
    }

    fn goal_not_reached(&self, date: NaiveDate) -> bool {
        // Could probably be just
        // return self.stat.get(&date).is_some();
        if let Some(val) = self.stats.get(&date){
            // return val.0;
            return true;
        }
        return false;
    }

    fn remaining(&self, date: NaiveDate) -> u32 {
        if let Some(val) = self.stats.get(&date) {
            if val.0 {
                return 0;
            } else {
                return 1;
            }
        } else {
            return 1;
        }
    }

    fn goal(&self) -> u32 {
        return 1;
    }

    fn modify(&mut self, date: NaiveDate, event: TrackEvent) {
        if let Some(val) = self.stats.get_mut(&date) {
            match event {
                TrackEvent::Increment => *val = (val.0 ^ true).into(),
                TrackEvent::Decrement => {
                    if val.0 {
                        *val = false.into();
                    } else {
                        self.stats.remove(&date);
                    }
                }
            }
        } else {
            if event == TrackEvent::Increment {
                self.insert_entry(date, CustomBool(true));
            }
        }
    }
    fn inner_data_ref(&self) -> &InnerData {
        &self.inner_data
    }
    fn inner_data_mut_ref(&mut self) -> &mut InnerData {
        &mut self.inner_data
    }
    fn is_auto(&self) -> bool {
        self.auto
    }
}
