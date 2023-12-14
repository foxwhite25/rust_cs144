use std::{
    fmt::Write as fmtWrite,
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    path::Path,
};

use clap::Parser;
use dns_lookup::lookup_host;
use log::{debug, warn};
use url::Url;

fn get_addr(addr: &SocketAddr, host: &str, path: &str) -> String {
    let mut tcp = TcpStream::connect(addr).unwrap();
    debug!(
        "Sending TCP/HTTP requests to {} with host {}, path: {}",
        addr, host, path
    );
    write!(
        tcp,
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();

    let mut buf = String::new();
    tcp.read_to_string(&mut buf).unwrap();
    buf
}

fn lookup_domain(host: &str) -> Option<SocketAddr> {
    debug!("Looking up host {}", host);
    Some(SocketAddr::new(*lookup_host(host).ok()?.get(0)?, 80))
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The path to get
    #[arg(short, long)]
    path: String,
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    let args = Args::parse();
    let url = Url::parse(args.path.as_str()).unwrap();
    if url.scheme() != "http" {
        warn!(
            "Scheme {} is not http, still goint to try http but server might not support http only.",
            url.scheme()
        )
    }

    let addr = match url.host().unwrap() {
        url::Host::Domain(domain) => lookup_domain(domain).unwrap(),
        url::Host::Ipv4(ip) => SocketAddr::new(IpAddr::V4(ip), 80),
        url::Host::Ipv6(ip) => SocketAddr::new(IpAddr::V6(ip), 80),
    };
    debug!("Got addr: {}", addr);
    println!(
        "{}",
        get_addr(&addr, url.host().unwrap().to_string().as_str(), url.path())
    )
}
