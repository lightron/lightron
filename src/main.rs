mod http2;
mod http1_1;
use serde_derive::{Deserialize,Serialize};
use std::fs::OpenOptions;
use std::io::prelude::*;
use crossbeam_utils::thread;
use std::collections::HashMap;
use toml::from_str;
use http2::handle_http2;
use http1_1::handle_http1_1;
use simplelog::*;
use std::str::FromStr;



pub fn read_conf() -> String {
    let mut file = OpenOptions::new().read(true).open("lightron.conf").unwrap();
    let mut con=String::new();
    file.read_to_string(&mut con).expect("Unable to read to string.");
    con
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Website {
    name: String,
    class: String,
    access: String,
    resource: String,
    certificate: String,
    private_key: String,
    port_no: u16,
    push_protocol_files: Vec<String>,
    log_level: String
}


#[cfg(windows)]
fn main() -> windows_service::Result<()> {
    lightron_service::run()
}


#[cfg(not(windows))]
fn main() {
    let config: HashMap<String, Vec<Website>> = from_str(&read_conf()).unwrap();
    let mut virtual_hosted_website : HashMap<u16,HashMap<&'static str,[&'static str;3]>> = HashMap::new(); // domain_name : <resourcepath,certificate,privkey>
    let mut websites = config["websites"].clone();
    for website in &websites {
        let mut v_websites : HashMap<&'static str,[&'static str;3]> = HashMap::new();
        if !virtual_hosted_website.contains_key(&website.port_no) {
            for webs in websites.clone() {
                if website.port_no == webs.port_no {
                    v_websites.insert(Box::leak(webs.name.into_boxed_str()),[Box::leak(webs.resource.into_boxed_str()),Box::leak(webs.certificate.into_boxed_str()),Box::leak(webs.private_key.into_boxed_str())]);
                }
            }
            virtual_hosted_website.insert(website.port_no,v_websites.clone());
        }
    }
    for port in virtual_hosted_website.keys() {
        let mut websites_to_be_removed : Vec<u8> = Vec::new();
        let mut i = 0;
        for website in &websites {
            if website.port_no == *port {
                websites_to_be_removed.push(i);
            }
            i+=1;
        }
        websites_to_be_removed.remove(0);
        for j in websites_to_be_removed {
            websites.remove(j.into());
        }
    }
    let mut log_config = ConfigBuilder::new();
    log_config.set_time_to_local(true);
    WriteLogger::init(LevelFilter::from_str(&websites[0].log_level).unwrap(), log_config.build(), std::fs::File::create(format!("lightron.log")).unwrap()).unwrap();
    thread::scope(|s| {
        for website in websites.clone() {
            let (is_virtually_shared,domain_map) = if virtual_hosted_website[&website.port_no].len() == 1 {
                (false,None)
            }
            else {
                (true,Some(virtual_hosted_website[&website.port_no].clone()))
            };
            if website.class == "HTTPS" {
                s.builder().name(website.port_no.to_string()).spawn(move |_| {
                    handle_http2(&website.access,website.port_no,&website.certificate,&website.private_key,Box::leak(website.resource.into_boxed_str()),website.push_protocol_files,is_virtually_shared,domain_map).unwrap();
                }).unwrap();
            }
            else {
                s.builder().name(website.port_no.to_string()).spawn(move |_| {
                    handle_http1_1(&website.access,website.port_no,Box::leak(website.resource.into_boxed_str()),is_virtually_shared,domain_map).unwrap();
                }).unwrap();
            }
        }
    }).unwrap();
}


#[cfg(windows)]
mod lightron_service {
    use std::{
        ffi::OsString,
        sync::mpsc,
        time::Duration,
    };
    use windows_service::{
        define_windows_service,
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher, Result,
    };
    use crate::*;

    const SERVICE_NAME: &str = "Lightron";
    const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

    pub fn run() -> Result<()> {
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)
    }
    define_windows_service!(ffi_service_main, my_service_main);
    pub fn my_service_main(_arguments: Vec<OsString>) {
        if let Err(_e) = run_service() {
            // Handle the error, by logging or something.
        }
    }
    pub fn run_service() -> Result<()> {
        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                ServiceControl::Stop => {
                    shutdown_tx.send(()).unwrap();
                    ServiceControlHandlerResult::NoError
                }
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };
        let status_handle = service_control_handler::register(SERVICE_NAME, event_handler).unwrap();

        
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;
        let config: HashMap<String, Vec<Website>> = from_str(&read_conf()).unwrap();
        let mut virtual_hosted_website : HashMap<u16,HashMap<&'static str,[&'static str;3]>> = HashMap::new(); // domain_name : <resourcepath>
        let mut websites = config["websites"].clone();
        for website in &websites {
            let mut v_websites : HashMap<&'static str,[&'static str;3]> = HashMap::new();
            if !virtual_hosted_website.contains_key(&website.port_no) {
                for webs in websites.clone() {
                    if website.port_no == webs.port_no {
                        v_websites.insert(Box::leak(webs.name.into_boxed_str()),[Box::leak(webs.resource.into_boxed_str()),Box::leak(webs.certificate.into_boxed_str()),Box::leak(webs.private_key.into_boxed_str())]);
                    }
                }
                virtual_hosted_website.insert(website.port_no,v_websites.clone());
            }
        }
        for port in virtual_hosted_website.keys() {
            let mut websites_to_be_removed : Vec<u8> = Vec::new();
            let mut i = 0;
            for website in &websites {
                if website.port_no == *port {
                    websites_to_be_removed.push(i);
                }
                i+=1;
            }
            websites_to_be_removed.remove(0);
            for j in websites_to_be_removed {
                websites.remove(j.into());
            }
        }
        let mut log_config = ConfigBuilder::new();
        log_config.set_time_to_local(true);
        WriteLogger::init(LevelFilter::from_str(&websites[0].log_level).unwrap(), log_config.build(), std::fs::File::create(format!("lightron.log")).unwrap()).unwrap();
        thread::scope(|s| {
            for website in websites.clone() {
                let (is_virtually_shared,domain_map) = if virtual_hosted_website[&website.port_no].len() == 1 {
                    (false,None)
                }
                else {
                    (true,Some(virtual_hosted_website[&website.port_no].clone()))
                };
                if website.class == "HTTPS" {
                    s.builder().name(website.port_no.to_string()).spawn(move |_| {
                        handle_http2(&website.access,website.port_no,&website.certificate,&website.private_key,Box::leak(website.resource.into_boxed_str()),website.push_protocol_files,is_virtually_shared,domain_map).unwrap();
                    }).unwrap();
                }
                else {
                    s.builder().name(website.port_no.to_string()).spawn(move |_| {
                        handle_http1_1(&website.access,website.port_no,Box::leak(website.resource.into_boxed_str()),is_virtually_shared,domain_map).unwrap();
                    }).unwrap();
                }
            }
            loop {
                match shutdown_rx.recv_timeout(Duration::from_secs(1)) {
                    // Break the loop either upon stop or channel disconnect
                    Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                        status_handle.set_service_status(ServiceStatus {
                            service_type: SERVICE_TYPE,
                            current_state: ServiceState::Stopped,
                            controls_accepted: ServiceControlAccept::empty(),
                            exit_code: ServiceExitCode::Win32(0),
                            checkpoint: 0,
                            wait_hint: Duration::default(),
                            process_id: None
                        }).unwrap();
                        break
                    },
                    // Continue work if no events were received within the timeout
                    Err(mpsc::RecvTimeoutError::Timeout) => (),
                };
            }            
        }).unwrap();

        Ok(())
    }
}
