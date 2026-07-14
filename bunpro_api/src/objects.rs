use std::{collections::BTreeMap, sync::Mutex};

use jiff::{
    Span, Timestamp, ToSpan,
    civil::{Date, DateTime},
    tz::TimeZone,
};
use log::trace;
use serde::{Deserialize, Deserializer, Serialize};

pub type CardCount = u32;

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TotalDue {
    pub total_due_grammar: CardCount,
    pub total_due_vocab: CardCount,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForecastDaily {
    pub grammar: ForecastDailyObject,
    pub vocab: ForecastDailyObject,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForecastHourly {
    pub grammar: ForecastHourlyObject,
    pub vocab: ForecastHourlyObject,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ForecastHourlyObject {
    #[serde(flatten)]
    pub rest: BTreeMap<Zoned, CardCount>,
}

impl<'de> Deserialize<'de> for ForecastHourlyObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // reset date on current so dates from next field don't carry over
        *CURRENT.lock().map_err(serde::de::Error::custom)? = DateTime::ZERO;

        let rest = BTreeMap::<Zoned, CardCount>::deserialize(deserializer)?;
        Ok(ForecastHourlyObject { rest })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForecastDailyObject {
    pub later: CardCount,
    pub tomorrow: CardCount,
    #[serde(flatten)]
    pub rest: BTreeMap<Date, CardCount>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Hash)]
#[serde(transparent)]
pub struct Zoned(#[serde(deserialize_with = "timestamp_to_zoned")] pub jiff::Zoned);

static CURRENT: Mutex<DateTime> = Mutex::new(DateTime::ZERO);

fn timestamp_to_zoned<'de, D>(de: D) -> Result<jiff::Zoned, D::Error>
where
    D: Deserializer<'de>,
{
    use std::sync::LazyLock;

    static CURRENT_TZ: LazyLock<TimeZone> = LazyLock::new(TimeZone::system);
    static CURRENT_OFFSET: LazyLock<Span> = LazyLock::new(|| {
        let offset = CURRENT_TZ.to_offset(Timestamp::now());
        ((offset.seconds() / 3600) as i8).hours()
    });

    let s = <&str as Deserialize>::deserialize(de)?;

    let naive_dt: DateTime = s
        .trim_end_matches('Z')
        .parse()
        .map_err(serde::de::Error::custom)?;

    let new_time = naive_dt.time() + *CURRENT_OFFSET;

    let mut current = CURRENT.lock().unwrap();
    if *current == DateTime::ZERO {
        *current = naive_dt;
    } else if new_time.hour() == 0 {
        // we overflowed into the next day
        *current += 1.day();
    }

    let date = naive_dt
        .with()
        .date(current.date())
        .time(new_time)
        .build()
        .map_err(serde::de::Error::custom)?
        .to_zoned(CURRENT_TZ.clone())
        .map_err(serde::de::Error::custom)?;

    *current = date.datetime();

    trace!(hour:% = s, date:?; "parsing hour");

    Ok(date)
}
