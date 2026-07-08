use std::{collections::BTreeMap, marker::PhantomData};

use bunpro_api::objects::{CardCount as BpCardCount, ForecastDailyObject, ForecastHourlyObject};
use jiff::{ToSpan, Zoned, civil::Date};

struct RepeatLast<I, F, Z>
where
    I: Iterator,
    F: FnMut(&mut ForecastItem<Z>),
{
    iter: I,
    last: Option<I::Item>,
    f: F,
    len: usize,
    x: usize,
    z: PhantomData<Z>,
}

impl<I, F, Z> Iterator for RepeatLast<I, F, Z>
where
    I: Iterator<Item = ForecastItem<Z>>,
    F: FnMut(&mut ForecastItem<Z>),
    Z: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.iter.next() {
            self.x += 1;
            if self.x == self.len {
                self.last = Some(item.clone());
            }

            Some(item)
        } else {
            if let Some(last) = self.last.as_mut() {
                (self.f)(last);
            }

            self.last.clone()
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Forecast {
    pub daily: ForecastEntry<Date>,
    pub hourly: ForecastEntry<Zoned>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ForecastEntry<T> {
    pub grammar: ForecastObject<T>,
    pub vocab: ForecastObject<T>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ForecastObject<T>(pub BTreeMap<T, CardCount>);

impl ForecastObject<Date> {
    /// Gets the "later" value
    pub fn later(&self) -> CardCount {
        self.0.values().nth(0).copied().unwrap_or_default()
    }

    /// Gets the "tomorrow" value
    pub fn tomorrow(&self) -> CardCount {
        self.0.values().nth(1).copied().unwrap_or_default()
    }

    /// Gets the "rest" value (all the values after later and tomorrow)
    pub fn rest(&self) -> impl ExactSizeIterator<Item = ForecastItem<Date>> {
        // skip later and tomorrow
        self.0.iter().skip(2).map(ForecastItem::from)
    }

    /// Gets all the values (including later and tomorrow)
    pub fn all(&self) -> impl ExactSizeIterator<Item = ForecastItem<Date>> {
        // skip later and tomorrow
        self.0.iter().map(ForecastItem::from)
    }

    /// Infinite iterator which projects into future based on available data.
    /// Note that projected future data is only as good as the last available data.
    /// Refresh periodically to make it accurate.
    /// Use take() to limit amount of data
    pub fn rest_infinite(&self) -> impl Iterator<Item = ForecastItem<Date>> {
        let iter = self.rest();

        RepeatLast {
            len: iter.len(),
            iter,
            last: None,
            f: |item| {
                item.key += 1.day();
                item.value.new = 0;
            },
            x: 0,
            z: PhantomData,
        }
    }

    /// Infinite iterator which projects into future based on available data.
    /// Note that projected future data is only as good as the last available data.
    /// Refresh periodically to make it accurate.
    /// Use take() to limit amount of data
    pub fn all_infinite(&self) -> impl Iterator<Item = ForecastItem<Date>> {
        let iter = self.all();

        RepeatLast {
            len: iter.len(),
            iter,
            last: None,
            f: |item| {
                item.key += 1.day();
                item.value.new = 0;
            },
            x: 0,
            z: PhantomData,
        }
    }
}

impl ForecastObject<Zoned> {
    pub fn rest(&self) -> impl ExactSizeIterator<Item = ForecastItem<Zoned>> {
        // rest is everything there is
        self.0.iter().map(ForecastItem::from)
    }

    /// Infinite iterator which projects into future based on available data.
    /// Note that projected future data is only as good as the last available data.
    /// Refresh periodically to make it accurate.
    // Use take() to limit amount of data
    pub fn rest_infinite(&self) -> impl Iterator<Item = ForecastItem<Zoned>> {
        let iter = self.rest();

        RepeatLast {
            len: iter.len(),
            iter,
            last: None,
            f: |item| {
                item.key += 1.hour();
                item.value.new = 0;
            },
            x: 0,
            z: PhantomData,
        }
    }
}

impl From<(BpCardCount, ForecastDailyObject)> for ForecastObject<Date> {
    fn from((due, items): (BpCardCount, ForecastDailyObject)) -> Self {
        let mut total = due;
        let mut map = items.rest;

        let Some(&date) = map.keys().next() else {
            return Self::default();
        };

        let today = date - 2.days();
        let tomorrow = date - 1.day();

        map.insert(today, items.later);
        map.insert(tomorrow, items.tomorrow);

        let map = map
            .into_iter()
            .map(|(date, new)| {
                total += new;

                let count = CardCount { new, total };

                (date, count)
            })
            .collect();

        Self(map)
    }
}

impl From<(BpCardCount, ForecastHourlyObject)> for ForecastObject<Zoned> {
    fn from((due, items): (BpCardCount, ForecastHourlyObject)) -> Self {
        let mut total = due;
        let map = items
            .rest
            .into_iter()
            .map(|(zone, new)| {
                total += new;

                let count = CardCount { new, total };

                (zone.0, count)
            })
            .collect();

        Self(map)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForecastItem<T> {
    pub key: T,
    pub value: CardCount,
}

impl<T: Copy> Copy for ForecastItem<T> {}

impl<T: Clone> From<(&T, &CardCount)> for ForecastItem<T> {
    fn from((key, &value): (&T, &CardCount)) -> Self {
        Self {
            key: key.clone(),
            value,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CardCount {
    pub new: BpCardCount,
    pub total: BpCardCount,
}
