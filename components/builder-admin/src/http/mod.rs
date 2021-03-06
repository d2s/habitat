// Copyright (c) 2016 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A module containing the HTTP server and handlers for servicing client requests

pub mod handlers;

use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};

use hab_net::privilege;
use hab_net::http::middleware::*;
use hab_net::oauth::github::GitHubClient;
use iron::prelude::*;
use iron::Protocol;
use mount::Mount;
use persistent;
use staticfile::Static;

use config::Config;
use error::Result;
use self::handlers::*;

// Iron defaults to a threadpool of size `8 * num_cpus`.
// See: http://172.16.2.131:9633/iron/prelude/struct.Iron.html#method.http
const HTTP_THREAD_COUNT: usize = 128;

/// Create a new `iron::Chain` containing a Router and it's required middleware
pub fn router(config: Arc<Config>) -> Result<Chain> {
    let admin = Authenticated::new(&*config).require(privilege::ADMIN);
    let router = router!(
        get "/status" => status,
        post "/search" => XHandler::new(search).before(admin.clone()),
        get "/accounts/:id" => XHandler::new(account_show).before(admin.clone()),
        get "/features" => XHandler::new(features_list).before(admin.clone()),
        get "/features/:id/teams" => XHandler::new(feature_grants_list).before(admin.clone()),
        post "/features/:id/teams" => XHandler::new(feature_grant).before(admin.clone()),
        delete "/features/:feature/teams/:id" => {
            XHandler::new(feature_revoke).before(admin.clone())
        }
    );
    let mut chain = Chain::new(router);
    chain.link(persistent::Read::<GitHubCli>::both(GitHubClient::new(&*config)));
    chain.link_before(RouteBroker);
    chain.link_after(Cors);
    Ok(chain)
}

/// Create a new HTTP listener and run it in a separate thread. This function will block the calling
/// thread until the new listener has successfully started.
///
/// # Errors
///
/// * Couldn't create Router or it's middleware
///
/// # Panics
///
/// * Listener crashed during startup
pub fn run(config: Arc<Config>) -> Result<JoinHandle<()>> {
    let (tx, rx) = mpsc::sync_channel(1);
    let addr = config.http_addr.clone();
    let mut mount = Mount::new();
    if let Some(ref path) = config.ui_root {
        debug!("Mounting UI at filepath {}", path);
        mount.mount("/", Static::new(path));
    }
    let chain = try!(router(config));
    mount.mount("/v1", chain);

    let handle = thread::Builder::new()
        .name("http-srv".to_string())
        .spawn(move || {
            let _server = Iron::new(mount)
                .listen_with(addr, HTTP_THREAD_COUNT, Protocol::Http, None)
                .unwrap();
            tx.send(()).unwrap();
        })
        .unwrap();
    match rx.recv() {
        Ok(()) => Ok(handle),
        Err(e) => panic!("http-srv thread startup error, err={}", e),
    }
}
