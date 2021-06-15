use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::rustls::{Certificate, NoClientAuth, PrivateKey, ServerConfig, ResolvesServerCertUsingSNI , sign::{CertifiedKey,RSASigningKey}};
use tokio_rustls::TlsAcceptor;
use rustls_pemfile;
use http::{Response,StatusCode,Version,Request};
use h2::server;
use bytes::Bytes;
use std::collections::HashMap;
use simplelog::*;
use log::{info,error,trace,debug};
use linkcheck::validation::{resolve_link,Options};

fn load_certs(filename: &str) -> Vec<Certificate> {
    let certfile = File::open(filename).unwrap();
    let mut reader = BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader).unwrap()
        .iter()
        .map(|v| Certificate(v.clone()))
        .collect()
}

fn load_private_key(filename: &str) -> PrivateKey {
    let keyfile = File::open(filename).unwrap();
    let mut reader = BufReader::new(keyfile);

    loop {
        match rustls_pemfile::read_one(&mut reader).unwrap() {
            Some(rustls_pemfile::Item::RSAKey(key)) => return PrivateKey(key),
            Some(rustls_pemfile::Item::PKCS8Key(key)) => return PrivateKey(key),
            None => break,
            _ => {}
        }
    }

    panic!("no keys found in {:?} (encrypted keys not supported)", filename);
}

async fn read_web_docs(file_name : std::path::PathBuf, content_type : mime_guess::Mime, port_no : u16) -> (StatusCode,Vec<u8>) {
    let src_file = File::open(file_name);
    let mut file_contents = Vec::new();
    let status:StatusCode;
    file_contents = match src_file {
        Ok(mut file) => {
            file.read_to_end(&mut file_contents).unwrap();
            status = StatusCode::OK;
            file_contents
        },
        Err(_) => { 
            log::warn!("{} 404 Triggered",port_no);
            status = StatusCode::NOT_FOUND;
            if content_type == mime_guess::mime::TEXT_HTML {
                let mut file_404 = if cfg!(target_os = "windows") {
                        File::open("assets\\404.html").unwrap()
                    }
                    else {
                        File::open("assets/404.html").unwrap()
                    };
                let mut file_404_string = Vec::new();
                file_404.read_to_end(&mut file_404_string).unwrap();
                file_404_string
            }
            else
            {
                vec![] 
            }
        },
    };
    (status,file_contents)
}

fn validate_path(parent_path : &str, file_path : &str) -> std::path::PathBuf {
    let (modified_parent_path,modified_file_path) = if cfg!(target_os = "windows") {
        (parent_path.replace("/", "\\"),file_path.replace("/", "\\"))
    }
    else {
        (parent_path.to_string(),file_path.to_string())
    };
    let linkcheck_options = Options::new().with_root_directory(modified_parent_path.clone()).unwrap().set_links_may_traverse_the_root_directory(false);
    resolve_link(std::path::Path::new(&modified_parent_path),std::path::Path::new(&modified_file_path),&linkcheck_options).unwrap_or(
        if cfg!(target_os = "windows") {
            std::path::PathBuf::from("assets\\403.html")
        }
        else {
            std::path::PathBuf::from("assets/403.html")
        }
    )
}



#[tokio::main]
pub async fn handle_http2(access: &str,port_no:u16, cert_path:&str, priv_path:&str, resource_path:&'static str, push_files:Vec<String>, is_virtually_shared : bool, domain_map : Option<HashMap<&'static str,[&'static str;3]>>) -> io::Result<()> {
    let mut log_config = ConfigBuilder::new();
    log_config.set_time_to_local(true);
    info!("Thread created for HTTPS port no : {}",port_no);
    let addr: std::net::SocketAddr = if access == "Local" {
        std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),port_no)
    }
    else {
        std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),port_no)
    };
    let mut config = ServerConfig::new(NoClientAuth::new());
    if is_virtually_shared {
        let mut resolver = ResolvesServerCertUsingSNI::new();
        for (domain_name,others) in domain_map.clone().unwrap() {
            let cred : CertifiedKey = CertifiedKey::new(load_certs(others[1]),Arc::new(Box::new(RSASigningKey::new(&load_private_key(others[2])).unwrap())));
            resolver.add(domain_name, cred).unwrap();
        } 
        config.cert_resolver = Arc::new(resolver);
    }
    else {
        let certs = load_certs(cert_path);
        let keys = load_private_key(&priv_path);
        config.set_single_cert(certs, keys).map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err)).unwrap();
    }
    config.set_protocols(&[b"h2".to_vec(), b"http/1.1".to_vec()]);
    
    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind(&addr).await.unwrap();   
    loop {
        let (stream, peer_addr) = listener.accept().await.unwrap();
        let acceptor = acceptor.clone();
        let push_files = push_files.clone();
        let domain_map_clone = domain_map.clone();
        let fut = async move {
            let tls_stream = acceptor.accept(stream).await.unwrap();
            let mut connection = server::handshake(tls_stream).await.unwrap();
            info!("{} HTTP/2 Hello: {}", port_no ,peer_addr);
            while let Some(result) = connection.accept().await {
                let (request, mut respond) = result.unwrap();
                trace!("{} REQUEST : {:?}", port_no ,request);
                let mut path = request.uri().path().to_string();
                if request.uri().path() == "/" {
                    path = path + "index.html";
                    let pushed_uri_auth: &str = &(request.uri().scheme_str().unwrap().to_string() + "://" + &request.uri().authority().unwrap().to_string());
                    debug!("{} pushed_path : {}",port_no,pushed_uri_auth);
                    for file in &push_files {
                        let pushed_req = Request::builder()
                            .uri(pushed_uri_auth.to_string() + "/" + file)
                            .body(())
                            .unwrap();
                        let content_type = mime_guess::from_path(&file);
                        let pushed_rsp = http::Response::builder().status(200).header("Content-Type", format!("{}",content_type.first_or(mime_guess::mime::TEXT_HTML))).body(()).unwrap();
                        let mut send_pushed = respond
                            .push_request(pushed_req)
                            .unwrap()
                            .send_response(pushed_rsp, false)
                            .unwrap();
                        if is_virtually_shared {
                            let mut push_file = if cfg!(target_os = "windows") {
                                File::open(domain_map_clone.clone().unwrap()[request.uri().authority().unwrap().as_str()][0].to_string() + "\\" + &file.replace('/',"\\")).unwrap()
                            }
                            else {
                                File::open(domain_map_clone.clone().unwrap()[request.uri().authority().unwrap().as_str()][0].to_string() + "/" + file).unwrap()
                            };
                            let mut push_contents = Vec::new();
                            push_file.read_to_end(&mut push_contents).unwrap();
                            send_pushed.send_data(Bytes::from(push_contents), true).unwrap();
                        }
                        else {
                            let mut push_file = if cfg!(target_os = "windows") {
                                File::open(resource_path.to_string() + "\\" + &file.replace('/',"\\")).unwrap()
                            }
                            else {
                                File::open(resource_path.to_string() + "/" + file).unwrap()
                            };
                            let mut push_contents = Vec::new();
                            push_file.read_to_end(&mut push_contents).unwrap();
                            send_pushed.send_data(Bytes::from(push_contents), true).unwrap();
                        }
                    }
                }
                let content_type = mime_guess::from_path(&path);
                debug!("{} path : {}",port_no,path);
                let (status,contents) = if is_virtually_shared {
                    read_web_docs(validate_path(domain_map_clone.clone().unwrap()[request.uri().authority().unwrap().as_str()][0],&path),content_type.first_or(mime_guess::mime::TEXT_HTML),port_no).await
                }
                else {
                    read_web_docs(validate_path(resource_path,&path),content_type.first_or(mime_guess::mime::TEXT_HTML),port_no).await
                };
                let response = Response::builder().version(Version::HTTP_2).status(status).header("Content-Type", format!("{}",content_type.first_or(mime_guess::mime::TEXT_HTML))).header("Server", "Lightron/0.1.0").body(()).unwrap();                
                let mut send = respond.send_response(response, false).unwrap();
                send.send_data(Bytes::from(contents),true).unwrap();
            }
            Ok(()) as io::Result<()>
        };

        tokio::spawn(async move {
            if let Err(err) = fut.await {
                error!("{} {:?}", port_no,err);
            }
        });
    }
}