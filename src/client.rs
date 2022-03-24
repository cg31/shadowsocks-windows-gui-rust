
use std::thread;
use std::sync::Arc;

use log::error;

use futures::future::{self, Either};

use tokio::{self, runtime::Builder};
use tokio::sync::Notify;

use shadowsocks_service as ss;

use native_windows_gui as nwg;

use anyhow::Result;

use crate::config;


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

    pub fn connect(&mut self, index: usize) -> Result<()> {
        if self.config.servers.len() == 0 {
            return Err(anyhow::format_err!("Empty servers"))
        }

        self.notify.notify_one();

        if let Some(th) = self.th.take() {
            let _ = th.join();
        }

        self.config.select = index;

        self.notify = Arc::new(Notify::new());
        let notify = self.notify.clone();

        let server = &self.config.servers[index];

        let mut ssconfig = ss::config::Config::new(ss::config::ConfigType::Local);

        let svr_addr = match server.server.parse::<ss::shadowsocks::config::ServerAddr>() {
            Ok(addr) => addr,
            Err(_) => anyhow::bail!("Wrong server address")
        };

        let cipher = match server.method.parse() {
            Ok(cipher) => cipher,
            Err(_) => anyhow::bail!("Wrong method")
        };

        let sc = ss::shadowsocks::ServerConfig::new(svr_addr, server.password.clone(), cipher);

        ssconfig.server.push(sc);

        let laddr = match self.config.local_addr.parse::<ss::shadowsocks::config::ServerAddr>() {
            Ok(cipher) => cipher,
            Err(_e) => anyhow::bail!("Wrong local address")
        };
        let lc = ss::config::LocalConfig::new_with_addr(laddr, ss::config::ProtocolType::Socks);

        ssconfig.local.push(lc);

        self.th = Some(thread::spawn(move || {
            if let Err(_e) = serve(ssconfig, notify) {
                //error!("{}", e);
            }
        }));

        Ok(())
    }
}

fn serve(ssconfig: ss::config::Config, notify: Arc<Notify>) -> Result<()> {
    let mut builder = Builder::new_multi_thread();

    if let Ok(runtime) = builder.enable_all().build() {
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
    }

    Ok(())
}

async fn wait_notify(notify: Arc<Notify>) -> Result<()> {
    notify.notified().await;
    Ok(())
}

