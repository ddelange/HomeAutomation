extern crate dataserver;
extern crate actix_web;
#[macro_use]
extern crate log;

use crate::actix_web::actix::Arbiter;
use crate::actix_web::{server,App,http::Method};
use crate::actix_web::middleware::identity::{CookieIdentityPolicy, IdentityService};

use std::sync::mpsc;
use std::sync::atomic::{AtomicUsize};
use std::thread;

use dataserver::{certificate_manager, httpserver};
use dataserver::helper;
use dataserver::httpserver::{secure_database::PasswordDatabase, timeseries_interface, ServerHandle, DataRouterHandle, CheckLogin};
use dataserver::httpserver::{ws_index, index, logout, newdata, plot_data, list_data, login_get_and_check, login_page, serve_file};
use dataserver::httpserver::{InnerState, DataServerState};

use std::sync::{Arc, RwLock};
use std::io::stdin;
use std::collections::HashMap;

mod controller;
use controller::Event;

mod input;
use input::web_api;

pub struct ServerState {
	controller_addr: mpsc::Sender<Event>,
	dataserver_state: DataServerState,
}

impl InnerState for ServerState {
	fn inner_state(&self) -> &DataServerState {
		&self.dataserver_state
	}
}

const FORCE_CERT_REGEN: bool =	false;

pub fn start(signed_cert: &str, private_key: &str,
	data: Arc<RwLock<timeseries_interface::Data>>,
	passw_db: Arc<RwLock<PasswordDatabase>>,
	sessions: Arc<RwLock<HashMap<u16, dataserver::httpserver::Session>>>,
	controller_tx: mpsc::Sender<Event>) -> (DataRouterHandle, ServerHandle) {

	let tls_config = httpserver::make_tls_config(signed_cert, private_key);
	let cookie_key = httpserver::make_random_cookie_key();

  let free_session_ids = Arc::new(AtomicUsize::new(0));
	let free_ws_session_ids = Arc::new(AtomicUsize::new(0));

	let (tx, rx) = mpsc::channel();
	thread::spawn(move || {
		// Start data server actor in separate thread
		let sys = actix::System::new("http-server");
		let data_server = Arbiter::start(|_| httpserver::websocket_data_router::DataServer::default());
		let data_server_clone = data_server.clone();

		let web_server = server::new(move || {
			// data the webservers functions have access to
			let dataserver_state = DataServerState {
			  passw_db: passw_db.clone(),
			  websocket_addr: data_server_clone.clone(),
			  data: data.clone(),
			  sessions: sessions.clone(),
			  free_session_ids: free_session_ids.clone(),
			  free_ws_session_ids: free_ws_session_ids.clone(),
		  };
			let state = ServerState {
			  controller_addr: controller_tx.clone(),
				dataserver_state,
		  };

			App::with_state(state)
		    .middleware(IdentityService::new(
		      CookieIdentityPolicy::new(&cookie_key[..])
		      .domain("deviousd.duckdns.org")
		      .name("auth-cookie")
		      .path("/")
		      .secure(true),
		    ))
				.middleware(CheckLogin{
					public_roots: vec!(String::from("/commands")),
					..CheckLogin::default()
				})
				// homeautomation actions activated by https calls
				.resource(r"/commands/lamps/toggle", |r| r.method(Method::GET).f(web_api::toggle))
				.resource(r"/commands/lamps/evening", |r| r.method(Method::GET).f(web_api::evening))
				.resource(r"/commands/lamps/night", |r| r.method(Method::GET).f(web_api::night))
				.resource(r"/commands/lamps/day", |r| r.method(Method::GET).f(web_api::normal))
				.resource(r"/commands/lamps/dimmest", |r| r.method(Method::GET).f(web_api::dimmest))
				.resource(r"/commands/lamps/dim", |r| r.method(Method::GET).f(web_api::dim))
				// websocket route
				// note some browsers need already existing http connection to
				// this server for the upgrade to wss to work
				.resource("/", |r| r.f(index))
				.resource("/ws/", |r| r.method(Method::GET).f(ws_index))
				.resource("/logout", |r| r.f(logout))
				.resource(r"/newdata", |r| r.method(Method::POST).f(newdata))
				.resource("/plot", |r| r.f(plot_data))
				.resource(r"/list_data.html", |r| r.method(Method::GET).f(list_data))

				//login route, every uri starting "/login" will be forwarded to the adress
				//after "/login" once the client has been authenticated
				.resource(r"/login/{tail:.*}", |r| {
					r.method(Method::POST).with(login_get_and_check);
					r.method(Method::GET).f(login_page);
				})
				//for all other urls we try to resolve to static files in the "web" dir
				.resource(r"/{tail:.*}", |r| r.f(serve_file))
    })
    .bind_rustls("0.0.0.0:8080", tls_config).unwrap()
    //.bind("0.0.0.0:8080").unwrap() //without tcp use with debugging (note: https -> http, wss -> ws)
    .shutdown_timeout(5)    // shut down 5 seconds after getting the signal to shut down
    .start();

		let _ = tx.send((data_server, web_server));
		let _ = sys.run();
	});

	let (data_handle, web_handle) = rx.recv().unwrap();
	(data_handle, web_handle)
}

fn main() {
	//https://www.deviousd.duckdns.org:8080/index.html
	//only do if certs need update
	if FORCE_CERT_REGEN {
		//generate_and_sign_keys
		if let Err(error) = certificate_manager::generate_and_sign_keys(
			"deviousd.duckdns.org",
			"keys/cert.key",
			"keys/cert.cert",
			"keys/user.key",
		) {
			println!("could not auto generate certificate, error: {:?}", error)
		}
	}

	helper::setup_logging(2).expect("could not set up debugging");

	let passw_db = Arc::new(RwLock::new(PasswordDatabase::load("").unwrap()));
	let dataset_handle = Arc::new(RwLock::new(timeseries_interface::init("data").unwrap()));
	let sessions = Arc::new(RwLock::new(HashMap::new()));
	let (controller_tx, controller_rx) = mpsc::channel();

	let (data_router_handle, web_handle) = //starts webserver
	start("keys/cert.key", "keys/cert.cert", dataset_handle.clone(), passw_db.clone(), sessions.clone(), controller_tx.clone());

	controller::start(controller_rx).unwrap();
	input::attached_sensors::start_monitoring(controller_tx.clone(), data_router_handle, dataset_handle.clone());

	println!("press: t to send test data, n: to add a new user, q to quit, a to add new dataset");
	loop {
		let mut input = String::new();
		stdin().read_line(&mut input).unwrap();
		match input.as_str() {
			"t\n" => helper::send_test_data(dataset_handle.clone()),
			//"x\n" => httpserver::signal_newdata(data_handle.clone(),0),
			"n\n" => helper::add_user(& passw_db),
			"a\n" => helper::add_dataset(&passw_db, &dataset_handle),
			"q\n" => break,
			_ => println!("unhandled"),
		};
	}
	println!("shutting down");
	httpserver::stop(web_handle);
	controller_tx.send(Event::Stop);
}
