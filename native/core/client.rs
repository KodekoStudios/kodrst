use async_compat::CompatExt;
use tokio::io::AsyncWriteExt;

use std::mem::transmute;
use std::sync::Arc;
use std::pin::Pin;
use std::alloc::*;
use std::ptr::*;

use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;
use tokio::net::TcpStream;
use tokio::io::BufReader;

use crate::*;

#[repr(C)]
pub(crate) struct Client {
    pub authorization: &'static str,
    pub user_agent   : &'static str,
        connector    : TlsConnector
}

impl Client {
    #[inline(always)]
    pub fn new(authorization: &'static str, user_agent: &'static str) -> Client {
        let mut roots = RootCertStore::empty();
        for cert in rustls_native_certs::load_native_certs().certs {
            roots.add(cert).expect("Failed to add a cert!");
        }

        let connector = TlsConnector::from(
            Arc::new(
                ClientConfig::builder()
                    .with_root_certificates(roots)
                    .with_no_client_auth()
            )
        );

        Client { 
            authorization, 
            user_agent,
            connector,
        }
    }

    #[inline(always)]
    pub async unsafe fn send(&self, request: &Request) -> Result<Ptr<Response>, String> {
        let tcp_stream = map_err!(TcpStream::connect("discord.com:443").await)?;
        let tls_stream = map_err!(
            self.connector.connect(
                "discord.com".try_into().expect("Should never faild."),
                tcp_stream
            ).await
        )?;

        let (half_read, half_write) = tokio::io::split(tls_stream);
        let mut reader  = BufReader::new(half_read).compat();
        let mut writer  = half_write.compat();

        let mut headers = vec![
            aahc::Header { name: "Authorization", value: self.authorization.as_bytes() },
            aahc::Header { name: "User-Agent"   , value: self.user_agent   .as_bytes() },
            aahc::Header { name: "Host"         , value: b"discord.com"                },
        ];

        if !request.reason.is_null() {
            headers.push(
                aahc::Header { 
                    name : "X-Audit-Log-Reason", 
                    value: transmute(cstr(request.reason))
                }
            );
        }

        let len; // fuck

        let send = if request.files.is_empty() {
            if request.body.is_null() {
                map_err!(
                    aahc::send_headers(
                        request.method, 
                        cstr(request.route), 
                        &headers, 
                        Pin::new(&mut writer)
                    ).await
                )?
            } else {
                let bytes = cstr(request.body).as_bytes();
                let len   = bytes.len();

                headers.extend_from_slice(&[
                    aahc::Header { name: "Content-Length", value: usize_to_bytes(len) },
                    aahc::Header { name: "Content-Type"  , value: b"application/json" }
                ]);

                let mut send = map_err!(
                    aahc::send_headers(
                        request.method,
                        cstr(request.route),
                        &headers,
                        Pin::new(&mut writer)).await
                )?;
                send.hint_length(len as u64);

                map_err!(Pin::new(&mut send).compat().write_all(bytes).await)?;
                
                send
            }
        } else {
            headers.push(aahc::Header { name: "Content-Type"  , value: b"multipart/form-data; boundary=boundary" });

            let mut bytes = Vec::with_capacity(4096);

            bytes.extend_from_slice(b"--boundary\r\nContent-Disposition: form-data; name=\"payload_json\"\r\nContent-Type: application/json\r\n\r\n");
            bytes.extend_from_slice(transmute(cstr(request.body)));
            bytes.extend_from_slice(b"\r\n");

            for att in request.files {
                bytes.extend_from_slice(b"--boundary\r\n");
                bytes.extend_from_slice(att.header.as_bytes());
                bytes.extend_from_slice(att.buffer           );
                bytes.extend_from_slice(b"\r\n");
            }

            bytes.extend_from_slice(b"--boundary--\r\n");

            len = bytes.len().to_string();
            headers.push(aahc::Header { name: "Content-Length", value: len.as_bytes() });

            let mut send = map_err!(
                aahc::send_headers(request.method, cstr(request.route), &headers, Pin::new(&mut writer)).await
            )?;
            
            send.hint_length(bytes.len() as u64);

            map_err!(Pin::new(&mut send).compat().write_all(&bytes).await)?;

            send
        };

        let metadata = map_err!(send.finish().await)?;

        let mut hdr_store = [aahc::Header { name: "", value: b"" }; 128];
        let mut hdr_buf   = [0u8; 8192];
        
        let (resp_meta, resp_body) = map_err!(aahc::receive_headers(Pin::new(&mut reader), &mut hdr_buf, &mut hdr_store, metadata).await)?;

        let mut bytes = Vec::new();
        map_err!(tokio::io::copy(&mut resp_body.compat(), &mut bytes).await)?;

        let body = leak_as_cstr(String::from_utf8_unchecked(bytes).as_str());
        
        let raw_headers = alloc(Layout::array::<Header>(headers.len()).unwrap()).cast::<Header>();
        for (i, aahc::Header { name, value }) in headers.iter().enumerate() {
            *raw_headers.add(i) = Header {
                value: (value.len() as u32, leak_as_cstr(transmute(*value)).as_ptr()),
                name : (name .len() as u8 , leak_as_cstr(           name  ).as_ptr()),
            };
        }

        let raw_response = alloc(Layout::new::<Response>()).cast::<Response>();

        write(
            raw_response,
            Response {
                headers: (headers.len() as u8, raw_headers),
                body   : (body.len() as u32, body.as_ptr()),
                status : resp_meta.status,
                _pad   : [0; 6]
            }
        );

        Ok(Ptr(raw_response))
    }
}
