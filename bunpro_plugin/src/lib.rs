mod log;
mod panic;
mod qtlogging;
mod setup;

use std::{pin::Pin, sync::Arc, time::Duration};

use ::log::{trace, warn};
use bunpro_client::{
    BunproClient,
    bunpro_api::config::Token,
    objects::{CardCount, Forecast},
};
use cxx_qt::{Constructor, CxxQtType, Threading};
use cxx_qt_lib::{QMap, QMapPair as _, QMapPair_QString_QVariant, QString, QVariant};
use jiff::{ToSpan, Zoned};
use tokio::{
    runtime::{Builder, Runtime},
    select,
    sync::Mutex,
    time,
};
use tokio_util::sync::CancellationToken;

use crate::setup::setup;

#[cxx_qt::bridge]
pub mod qobject {

    impl cxx_qt::Threading for Bunpro {}
    impl cxx_qt::Constructor<()> for Bunpro {}

    extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qmap.h");
        type QMap_QString_QVariant = cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant>;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, api_key, READ, WRITE = set_api_key, NOTIFY, REQUIRED, FINAL)]
        #[qproperty(bool, dangerously_authenticate_using_api_key, READ, WRITE = set_dangerously_authenticate_using_api_key, NOTIFY, FINAL)]
        #[qproperty(QMap_QString_QVariant, current, READ = current, NOTIFY, FINAL)]
        #[qproperty(u16, update_interval, READ, WRITE = set_update_interval, NOTIFY, FINAL)]
        type Bunpro = super::BunproRust;

        fn current(&self) -> QMap_QString_QVariant;
        fn set_api_key(self: Pin<&mut Self>, api_key: &QString);
        fn set_dangerously_authenticate_using_api_key(self: Pin<&mut Self>, state: bool);
        fn set_update_interval(self: Pin<&mut Self>, mins: u16);

        #[qinvokable]
        fn refresh_forecast(self: Pin<&mut Self>);

        #[qsignal]
        fn error(self: Pin<&mut Self>, msg: QString);
    }
}

impl Constructor<()> for qobject::Bunpro {
    type NewArguments = ();

    type BaseArguments = ();

    type InitializeArguments = ();

    fn route_arguments(
        args: (),
    ) -> (
        Self::NewArguments,
        Self::BaseArguments,
        Self::InitializeArguments,
    ) {
        (args, (), ())
    }

    fn new(_: Self::NewArguments) -> <Self as CxxQtType>::Rust {
        setup();
        <Self as CxxQtType>::Rust::default()
    }

    fn initialize(mut self: Pin<&mut Self>, _: Self::InitializeArguments) {
        self.as_mut().spawn_current_update();
        self.as_mut().spawn_forecast_update();
    }
}

pub struct BunproRust {
    // config
    api_key: QString,
    dangerously_authenticate_using_api_key: bool,
    update_interval: u16,

    forecast: Forecast,

    client: Arc<Mutex<BunproClient>>,
    runtime: Runtime,
    cancel_update_forecast: Option<CancellationToken>,
}

impl Default for BunproRust {
    fn default() -> Self {
        let api_key = QString::default();
        let dangerously_authenticate_using_api_key = false;

        let runtime = Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime");

        let client = BunproClient::builder()
            .api_token("")
            .dangerously_authenticate_using_api_token(dangerously_authenticate_using_api_key)
            .build()
            .expect("Failed building BunproClient");

        Self {
            api_key,
            dangerously_authenticate_using_api_key,
            update_interval: 15,
            forecast: Forecast::default(),
            client: Arc::new(Mutex::new(client)),
            runtime,
            cancel_update_forecast: None,
        }
    }
}

// Invokables
impl qobject::Bunpro {
    fn refresh_forecast(self: Pin<&mut Self>) {
        let qthread = self.qt_thread();
        let client = self.client.clone();
        self.runtime.spawn(async move {
            {
                let mut client = client.lock().await;
                if let Err(e) = client.refresh_forecast().await {
                    _ = qthread.queue(move |this| {
                        this.error(format!("Api request failed: {e}").into());
                    });

                    return;
                }
            }

            let res = qthread.queue(move |mut this| {
                let lock = client.blocking_lock();

                if this.forecast != lock.forecast {
                    this.as_mut().rust_mut().forecast = lock.forecast.clone();
                    this.current_changed();
                }
            });

            if let Err(e) = res {
                warn!("QThread failed: {e}");
            }
        });
    }
}

// Properties
impl qobject::Bunpro {
    fn set_api_key(mut self: Pin<&mut Self>, api: &QString) {
        self.as_mut().rust_mut().api_key = api.clone();
        self.client.blocking_lock().config_mut(|config| {
            config.api_token = Token::new(&api.to_string());
        });
        self.as_mut().api_key_changed();
    }

    fn set_dangerously_authenticate_using_api_key(mut self: Pin<&mut Self>, state: bool) {
        self.as_mut()
            .rust_mut()
            .dangerously_authenticate_using_api_key = state;
        self.client.blocking_lock().config_mut(|config| {
            config.dangerously_authenticate_using_api_token = state;
        });
        self.dangerously_authenticate_using_api_key_changed();
    }

    fn set_update_interval(mut self: Pin<&mut Self>, mins: u16) {
        if let Some(key) = self.as_mut().rust_mut().cancel_update_forecast.take() {
            key.cancel();
        }

        self.as_mut().rust_mut().update_interval = mins;
        self.as_mut().update_interval_changed();

        self.spawn_forecast_update();
    }

    fn current(&self) -> QMap<QMapPair_QString_QVariant> {
        let (grammar, vocab) = self.get_current();

        let mut grammar_map = QMapPair_QString_QVariant::default();
        grammar_map.insert("new".into(), QVariant::from(&grammar.new));
        grammar_map.insert("total".into(), QVariant::from(&grammar.total));

        let mut vocab_map = QMapPair_QString_QVariant::default();
        vocab_map.insert("new".into(), QVariant::from(&vocab.new));
        vocab_map.insert("total".into(), QVariant::from(&vocab.total));

        let mut map = QMapPair_QString_QVariant::default();
        map.insert("grammar".into(), QVariant::from(&grammar_map));
        map.insert("vocab".into(), QVariant::from(&vocab_map));

        map
    }
}

// Normal functions
impl qobject::Bunpro {
    /// Returns (grammar, vocab)
    fn get_current(&self) -> (CardCount, CardCount) {
        let now = Zoned::now();

        macro_rules! get_count {
            ($name:ident) => {{
                let mut iter = self.forecast.hourly.$name.rest_infinite().peekable();
                loop {
                    // infinite iterators always have a next value
                    let Some(item) = iter.next() else {
                        break Default::default();
                    };
                    let Some(peek) = iter.peek() else {
                        break Default::default();
                    };

                    if peek.key > now {
                        break item.value;
                    }
                }
            }};
        }

        let grammar = get_count!(grammar);
        let vocab = get_count!(vocab);

        (grammar, vocab)
    }

    /// issues current changed signal every hour to update the displayed stats
    fn spawn_current_update(&self) {
        let qthread = self.qt_thread();

        self.runtime.spawn(async move {
            loop {
                let now = Zoned::now();
                let next_hour = now
                    .with()
                    .minute(0)
                    .second(0)
                    .millisecond(0)
                    .microsecond(0)
                    .nanosecond(0)
                    .build()
                    .unwrap()
                    + 1.hour();

                trace!("spawn_current_update: next hour at {next_hour:?}");

                let left = now.duration_until(&next_hour).as_secs() + 1;

                trace!("spawn_current_update: sleeping for {left}secs");

                time::sleep(Duration::from_secs(left as _)).await;

                let res = qthread.queue(|this| {
                    this.current_changed();
                });

                if res.is_err() {
                    break;
                }
            }
        });
    }

    fn spawn_forecast_update(mut self: Pin<&mut Self>) {
        let qthread = self.qt_thread();
        let cancel_key = CancellationToken::new();

        let interval = self.update_interval;
        self.as_mut().rust_mut().cancel_update_forecast = Some(cancel_key.clone());

        self.runtime.spawn(async move {
            loop {
                let now = Zoned::now();

                let next = ((now.minute() as u16 / interval) + 1) * interval;

                let next_time = now
                    .with()
                    .minute(0)
                    .second(0)
                    .millisecond(0)
                    .microsecond(0)
                    .nanosecond(0)
                    .build()
                    .unwrap()
                    + (next as i16).minutes();

                trace!("spawn_forecast_update: next wakeup time is at {next_time:?}");

                let seconds = now.duration_until(&next_time).as_secs() + 1;

                trace!("spawn_forecast_update: sleeping for {seconds}secs");

                select! {
                    _ = cancel_key.cancelled() => {
                        trace!("spawn_forecast_update: cancelled");
                        break
                    },

                    _ = time::sleep(Duration::from_secs(seconds as _)) => ()
                }

                let res = qthread.queue(|this| {
                    this.refresh_forecast();
                });

                if res.is_err() {
                    trace!("spawn_forecast_update: qthread error, breaking");
                    break;
                }
            }
        });
    }
}
