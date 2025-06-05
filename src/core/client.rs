use crate::{macros::usize_to_str_bytes, FlakeExtension, structs::*};

use tokio_rustls::{rustls::{ClientConfig, RootCertStore}, TlsConnector};
use tokio::{io::{copy, AsyncWriteExt, BufReader}, net::TcpStream};
use aahc::{receive_headers, send_headers, Header};
use rustls_native_certs::load_native_certs;
use std::{error::Error, pin::Pin};
use async_compat::CompatExt as _;
use kroos::Flake;

pub struct Client {
    pub authorization: Flake<str>  ,
    pub user_agent   : Flake<str>  ,
    pub connector    : TlsConnector,
}

impl Client {
    #[inline(always)]
    pub fn new(authorization: Flake<str>, user_agent: Flake<str>) -> Result<Self, Box<dyn Error>> {
        let mut roots = RootCertStore::empty();
        for cert in load_native_certs().certs {
            roots.add(cert)?;
        }

        let config = ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();

        Ok(Self {
            connector: TlsConnector::from(std::sync::Arc::new(config)),
            authorization, user_agent,
        })
    }

    pub async fn send(&self, request: &Request) -> Result<Response, Box<dyn Error>> {
        let tcp_stream = TcpStream::connect("discord.com:443").await?;
        let tls_stream = self.connector.connect("discord.com".try_into()?, tcp_stream).await?;

        let (half_read, half_write) = tokio::io::split(tls_stream);
        let mut reader  = BufReader::new(half_read).compat();
        let mut writer  = half_write.compat();

        let mut headers = vec![
            Header { name: "Authorization", value: self.authorization.as_bytes() },
            Header { name: "User-Agent"   , value: self.user_agent   .as_bytes() },
            Header { name: "Host",          value: b"discord.com"                },
        ];

        if request.reason.len() != 0 {
            headers.push(Header { name: "X-Audit-Log-Reason", value: request.reason.as_bytes() });
        }
        
        let send = if !request.files.is_empty() {
            headers.push(Header { name: "Content-Type", value: b"multipart/form-data; boundary=boundary" });
            
            let mut raw_bytes = Vec::with_capacity(4096);

            raw_bytes.extend_from_slice(b"--boundary\r\nContent-Disposition: form-data; name=\"payload_json\"\r\nContent-Type: application/json\r\n\r\n");
            raw_bytes.extend_from_slice(request.body.as_bytes());
            raw_bytes.extend_from_slice(b"\r\n");
            
            for att in &request.files {
                raw_bytes.extend_from_slice(b"--boundary\r\n");
                raw_bytes.extend_from_slice(att.header.as_bytes());
                raw_bytes.extend_from_slice(att.data  .as_ref()  );
                raw_bytes.extend_from_slice(b"\r\n");
            }
            
            raw_bytes.extend_from_slice(b"--boundary--\r\n");
            
            let raw_len = raw_bytes.len().to_string();
            headers.push(Header { name: "Content-Length", value: raw_len.as_bytes() });

            let mut send = send_headers(request.method.into(), &*request.route, &headers, Pin::new(&mut writer)).await?;
            send.hint_length(raw_bytes.len() as u64);

            Pin::new(&mut send).compat().write_all(&raw_bytes).await?;

            send
        } else if request.body != Flake::empty() {
            let raw_bytes = request.body.as_bytes();

            headers.extend_from_slice(&[
                Header { name: "Content-Length", value: usize_to_str_bytes(raw_bytes.len()) },
                Header { name: "Content-Type"  , value: b"application/json"                 }
            ]);

            let mut send = send_headers(request.method.into(), &*request.route, &headers, Pin::new(&mut writer)).await?;
            send.hint_length(raw_bytes.len() as u64);
            
            Pin::new(&mut send).compat().write_all(raw_bytes).await?;
            
            send
        } else {
            send_headers(request.method.into(), &*request.route, &headers, Pin::new(&mut writer)).await?
        };

        
        let metadata = send.finish().await?;
        
        let mut hdr_store = [Header { name: "", value: b"" }; 128];
        let mut hdr_buf   = [0u8; 8192];
        
        let (resp_meta, resp_body) = receive_headers(Pin::new(&mut reader), &mut hdr_buf, &mut hdr_store, metadata).await?;
        
        let headers = resp_meta.headers.iter()
            .take_while(|hdr| !hdr.name.is_empty())
            .map(|hdr| unsafe { (
                Flake::new(hdr.name), 
                Flake::new(std::mem::transmute(hdr.value))
            )})
            .collect();

        
        let mut body = Vec::new();
        copy(&mut resp_body.compat(), &mut body).await?;
        
        Ok(unsafe {
            Response::new(
                resp_meta.status, headers, 
                Flake::from_raw(body.leak() as *const [u8] as *const str)
            )
        })
    }
}