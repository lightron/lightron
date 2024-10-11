use std::io;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use std::fs::File;
use http::{StatusCode};
use log::{info,warn,error,trace,debug};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use std::io::prelude::*;
use std::collections::HashMap;
use linkcheck::validation::{resolve_link,Options};


#[tokio::main]
pub async fn handle_http1_1(access: &str,port_no : u16, resource_path : &'static str, is_virtually_shared : bool, domain_map : Option<HashMap<&'static str,[&'static str;3]>>) -> io::Result<()> {
    info!("Thread created for HTTP port no : {}",port_no);
    let addr: std::net::SocketAddr = if access == "Local" {
        std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),port_no)
    }
    else {
        std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),port_no)
    };
    let listener = TcpListener::bind(addr).await.unwrap();
    loop {
        let (stream, peer_addr) = listener.accept().await.unwrap();
        info!("{} HTTP/1.1 Hello : {}",port_no,peer_addr);
        let temp = domain_map.clone();
        let fut = async move {
            handle_connection(stream, resource_path, is_virtually_shared,temp,port_no).await
        };
        tokio::spawn(async move {
            if let Err(err) = fut.await {
                error!("{} {:?}",port_no,err);
            }
        });
    }
}

async fn handle_connection(mut stream: TcpStream,resource_path: &str,is_virtually_shared : bool, domain_map : Option<HashMap<&'static str,[&'static str;3]>>,port_no : u16) -> io::Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await.unwrap();
    trace!("{} REQUEST: {}", port_no ,String::from_utf8_lossy(&buffer));
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    req.parse(&buffer).unwrap().unwrap();
    let mut path = req.path.unwrap().to_string();
    if path == "/" {
        path = path + "index.html";
    }
    debug!("{} {}",port_no,path);
    let content_type = mime_guess::from_path(&path);
    let (status,contents) = if is_virtually_shared {
        let mut hostname : &str = "";
        for header in req.headers {
            if header.name == "Host" {
                hostname = std::str::from_utf8(header.value).unwrap();
            }
        }
        read_web_docs(
            validate_path(domain_map.unwrap()[hostname][0],&path), 
            content_type.first_or(mime_guess::mime::TEXT_HTML),
            port_no).await
    } 
    else {
        read_web_docs(
        validate_path(resource_path,&path),
        content_type.first_or(mime_guess::mime::TEXT_HTML),
        port_no).await
    };
    let response = format!("HTTP/1.1 {} {}\r\nContent-Type: {}\r\nServer: Lightron/0.1.0\r\n\r\n", status.as_str(), status.canonical_reason().unwrap(),content_type.first_or(mime_guess::mime::TEXT_HTML)).into_bytes();
    stream.write_all(&response).await.unwrap();
    stream.write_all(&contents).await.unwrap();
    stream.flush().await.unwrap();
    Ok(()) as io::Result<()>
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
            std::path::PathBuf::from("C:\\Program Files\\Common Files\\Lightron\\403.html")
        }
        else {
            std::path::PathBuf::from("/var/www/403.html")
        }
    )
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
            warn!("{} 404 Triggered",port_no);
            status = StatusCode::NOT_FOUND;
            if content_type == mime_guess::mime::TEXT_HTML {
                let mut file_404 = if cfg!(target_os = "windows") {
                        File::open("C:\\Program Files\\Common Files\\Lightron\\404.html").unwrap()
                    }
                    else {
                        File::open("/var/www/404.html").unwrap()
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