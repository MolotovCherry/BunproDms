use std::{
    env,
    fmt::{Display, Write},
    sync::{Mutex, TryLockError},
};

use env_filter::FilteredLog;
use log::{Level, LevelFilter, Log, Metadata, Record, kv::VisitSource};
use owo_colors::{OwoColorize, Style};

use crate::{q_critical, q_debug, q_info, q_warning};

pub struct QtLogger {
    buf: Mutex<String>,
}

impl Log for QtLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        // we already got filtered due to FilteredLog<QtLogger>
        true
    }

    fn log(&self, record: &Record) {
        let mut _guard;

        let buf = match self.buf.try_lock() {
            Ok(b) => {
                _guard = b;
                &mut *_guard
            }

            Err(TryLockError::Poisoned(lock)) => {
                _guard = lock.into_inner();
                &mut *_guard
            }

            Err(TryLockError::WouldBlock) => &mut String::new(),
        };

        _ = write!(
            buf,
            "{level} · {target} ·",
            level = color_level(record.level()),
            target = record.target().bright_blue()
        );

        if let Some(file) = record.file() {
            _ = write!(buf, " {}", file.bright_blue());

            _ = match record.line() {
                Some(line) => write!(buf, "{}{} ·", ":".bright_blue(), line.bright_blue()),
                None => write!(buf, " ·"),
            };
        }

        _ = write!(buf, " {msg}", msg = record.args());

        struct KvVisitor<'a> {
            buf: &'a mut String,
        }

        impl<'kvs> VisitSource<'kvs> for KvVisitor<'kvs> {
            fn visit_pair(
                &mut self,
                key: log::kv::Key<'kvs>,
                value: log::kv::Value<'kvs>,
            ) -> Result<(), log::kv::Error> {
                write!(
                    self.buf,
                    " {key}={value}",
                    key = key.bright_cyan(),
                    value = value.bright_magenta()
                )
                .map_err(|_| log::kv::Error::msg("failed to write kv"))
            }
        }

        let mut visitor = KvVisitor { buf: &mut *buf };
        _ = record.key_values().visit(&mut visitor);

        match record.level() {
            Level::Error => q_critical!("{buf}"),
            Level::Warn => q_warning!("{buf}"),
            Level::Info => q_info!("{buf}"),
            Level::Debug | Level::Trace => q_debug!("{buf}"),
        }

        buf.clear();
    }

    fn flush(&self) {}
}

impl QtLogger {
    pub fn init() {
        let mut builder = env_filter::Builder::new();

        if let Ok(ref filter) = env::var("BUNPRO_LOG") {
            builder.parse(filter);
        } else {
            let level = if cfg!(debug_assertions) {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            };

            builder.filter_level(level);
        }

        let filter = builder.build();
        let level = filter.filter();

        let filter = FilteredLog::new(
            Self {
                buf: Mutex::new(String::new()),
            },
            filter,
        );
        log::set_max_level(level);
        // ignore Result because we can't afford to do panics
        if let Err(e) = log::set_boxed_logger(Box::new(filter)) {
            q_critical!(
                "[{}] [log:{}] failed to init logger: {e}",
                "ERROR".bright_red(),
                line!()
            );
        }
    }
}

fn color_level(level: Level) -> impl Display {
    let style = Style::new();

    match level {
        Level::Error => {
            let style = style.bright_red();
            "ERROR".style(style)
        }

        Level::Warn => {
            let style = style.bright_yellow();
            "WARN".style(style)
        }

        Level::Info => {
            let style = style.bright_green();
            "INFO".style(style)
        }

        Level::Debug => {
            let style = style.bright_blue();
            "DEBUG".style(style)
        }

        Level::Trace => {
            let style = style.bright_cyan();
            "TRACE".style(style)
        }
    }
}
