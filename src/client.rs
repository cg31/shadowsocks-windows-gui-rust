
use std::io;

use std::thread;
use std::sync::Arc;

use log::error;

use futures::future::{self, Either};

use tokio::{self, runtime::Builder};
use tokio::sync::Notify;

use shadowsocks_service as ss;

use native_windows_gui as nwg;

use crate::config;
use crate::utils;


#[derive(Default)]
pub struct Client {
    pub config: config::Config,
    pub notify: Arc<Notify>,
    pub th: Option<thread::JoinHandle<()>>,
    pub handle: nwg::ControlHandle,
}

impl Client {
    pub fn new() -> Client {
        let cfg = match config::Config::load() {
            Ok(cfg) => cfg,
            _ => {
                error!("Can't find config file");
                config::Config::default()
            }
        };

        let data = Client {
            config: cfg,
            notify: Arc::new(Notify::new()),
            th: None,
            handle: nwg::ControlHandle::NoHandle,
        };

        data
    }

    pub fn connect(&mut self, index: usize) {
        if self.config.servers.len() == 0 {
            return;
        }

        self.notify.notify_one();

        if let Some(th) = self.th.take() {
            th.join().unwrap();
        }

        self.config.select = index;

        self.notify = Arc::new(Notify::new());
        let notify = self.notify.clone();

        let server = &self.config.servers[index];

        let server_addr = utils::string_to_str(server.server.clone());
        let passwd = utils::string_to_str(server.password.clone());
        let method = utils::string_to_str(server.method.clone());
        let local_addr = utils::string_to_str(self.config.local_addr.clone());

        let mut ssconfig = ss::config::Config::new(ss::config::ConfigType::Local);

        let svr_addr = server_addr.parse::<ss::shadowsocks::config::ServerAddr>().expect("server-addr");

        let cipher = method.parse().expect("server-addr");
        let sc = ss::shadowsocks::ServerConfig::new(svr_addr, passwd.to_owned(), cipher);

        ssconfig.server.push(sc);
        ssconfig.local_addr = Some(local_addr.parse().unwrap());

        self.th = Some(thread::spawn(move || {
            if let Err(_e) = serve(ssconfig, notify) {
                //error!("{}", e);
            }
        }));
    }
}

fn serve(ssconfig: ss::config::Config, notify: Arc<Notify>) -> io::Result<()> {
    let mut builder = Builder::new_multi_thread();

    let runtime = builder.enable_all().build().expect("create tokio Runtime");
    runtime.block_on(async move {
        let abort_signal = wait_notify(notify);
        let server = ss::run_local(ssconfig);

        tokio::pin!(abort_signal);
        tokio::pin!(server);

        match future::select(server, abort_signal).await {
            // Server future resolved without an error. This should never happen.
            Either::Left((Ok(..), ..)) => panic!("server exited unexpectly"),
            // Server future resolved with error, which are listener errors in most cases
            Either::Left((Err(err), ..)) => panic!("aborted with {}", err),
            // The abort signal future resolved. Means we should just exit.
            Either::Right(_) => (),
        }
    });

    Ok(())
}

async fn wait_notify(notify: Arc<Notify>) -> io::Result<()> {
    notify.notified().await;
    Ok(())
}

