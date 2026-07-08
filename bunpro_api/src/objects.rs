use std::collections::BTreeMap;

use jiff::{Timestamp, civil::Date, tz::TimeZone};
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForecastHourlyObject {
    #[serde(flatten)]
    pub rest: BTreeMap<Zoned, CardCount>,
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

fn timestamp_to_zoned<'de, D>(de: D) -> Result<jiff::Zoned, D::Error>
where
    D: Deserializer<'de>,
{
    use std::sync::LazyLock;
    static CURRENT_TZ: LazyLock<TimeZone> = LazyLock::new(TimeZone::system);

    let ts = Timestamp::deserialize(de)?.to_zoned(CURRENT_TZ.clone());

    Ok(ts)
}
