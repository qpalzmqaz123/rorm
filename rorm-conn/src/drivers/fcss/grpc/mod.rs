mod api;

use std::fmt::Display;

use http::Uri;
use tonic::transport::{Channel, Endpoint};

use crate::Result;

macro_rules! grpc_call {
    ($self:ident, $method:ident, $arg:expr) => {{
        // Grpc 调用
        let mut client = api::lsrp_client::LsrpClient::new($self.channel.clone());
        let res = client
            .$method($arg)
            .await
            .map_err(|e| {
                rorm_error::runtime!("Fcss grpc `{}` call failed: {}", stringify!($method), e)
            })?
            .into_inner();

        // 检查返回值
        if res.return_code != 0 {
            return Err(rorm_error::runtime!(
                "Fcss grpc {} return error: {}",
                stringify!($method),
                res.return_string
            ));
        }

        res
    }};
}

pub struct FcssGrpcClient {
    channel: Channel,
}

impl FcssGrpcClient {
    pub async fn connect(uri_str: &str) -> Result<Self> {
        log::trace!("Fcss grpc connect '{}'", uri_str);

        let uri = uri_str
            .parse::<Uri>()
            .map_err(|e| rorm_error::connection!("Fcss parse uri `{}` error: {}", uri_str, e))?;
        let channel = Endpoint::new(uri)
            .map_err(|e| rorm_error::connection!("Fcss endpoint '{}' error: {}", uri_str, e))?
            .connect()
            .await
            .map_err(|e| rorm_error::connection!("Fcss connect to '{}' error: {}", uri_str, e))?;

        Ok(Self { channel })
    }

    pub async fn set<K: ToString + Display, V: ToString + Display>(
        &self,
        key: K,
        value: V,
    ) -> Result<()> {
        log::trace!("Fcss grpc set `{}` -> `{}`", key, value);

        grpc_call!(
            self,
            add,
            api::TblLsrp {
                key: key.to_string(),
                value: maybe_compress(value.to_string())?,
            }
        );

        Ok(())
    }

    pub async fn del<K: ToString + Display>(&self, key: K) -> Result<()> {
        log::trace!("Fcss grpc del `{}`", key);

        grpc_call!(
            self,
            del,
            api::TblLsrp {
                key: key.to_string(),
                // FIXME: 设备问题，get 和 del 需要 value 不为空
                value: " ".to_owned(),
            }
        );

        Ok(())
    }

    pub async fn get<K: ToString + Display>(&self, key: K) -> Result<Option<String>> {
        log::trace!("Fcss grpc get `{}`", key);

        let res = grpc_call!(
            self,
            get,
            api::TblLsrp {
                key: key.to_string(),
                // FIXME: 设备问题，get 和 del 需要 value 不为空
                value: " ".to_owned(),
            }
        );
        let value = res
            .entry
            .ok_or(rorm_error::runtime!("Fcss grpc get return is none"))?
            .value;

        Ok(if res.flag {
            Some(maybe_decompress(value)?)
        } else {
            None
        })
    }

    pub async fn list(&self) -> Result<Vec<(String, String)>> {
        log::trace!("Fcss grpc list");

        let res = grpc_call!(self, getall, api::DefRange { start: 0, end: -1 });

        res.entries
            .into_iter()
            .map(|entry| Ok((entry.key, maybe_decompress(entry.value)?)))
            .collect::<Result<_>>()
    }
}

#[cfg(not(feature = "fcss-compress"))]
fn maybe_compress(s: String) -> Result<String> {
    Ok(s)
}

#[cfg(feature = "fcss-compress")]
fn maybe_compress(s: String) -> Result<String> {
    use std::io::Write;

    let mut encoder = snap::write::FrameEncoder::new(vec![]);
    encoder.write_all(s.as_bytes()).map_err(|e| {
        rorm_error::runtime!("Fcss grpc compress into_inner error: {}, data: `{}`", e, s)
    })?;
    let buf = encoder.into_inner().map_err(|e| {
        rorm_error::runtime!("Fcss grpc compress into_inner error: {}, data: `{}`", e, s)
    })?;
    log::trace!("Fcss grpc compressed {} -> {}", s.len(), buf.len());

    let b64 = base64::encode(buf);
    log::trace!("Fcss grpc compress to base64: `{}`", b64);

    Ok(b64)
}

#[cfg(not(feature = "fcss-compress"))]
fn maybe_decompress(s: String) -> Result<String> {
    Ok(s)
}

#[cfg(feature = "fcss-compress")]
fn maybe_decompress(b64: String) -> Result<String> {
    use std::io::copy;

    log::trace!("Fcss grpc decompress base64: `{}`", b64);
    let compressed = base64::decode(&b64).map_err(|e| {
        rorm_error::runtime!("Fcss grpc decompress base64 error: {}, data: `{}`", e, b64)
    })?;

    let mut decoder = snap::read::FrameDecoder::new(compressed.as_slice());
    let mut decompressed = vec![];
    copy(&mut decoder, &mut decompressed).map_err(|e| {
        rorm_error::runtime!(
            "Fcss grpc decompress copy error: {}, base64 data: `{}`",
            e,
            b64
        )
    })?;
    log::trace!(
        "Fcss grpc decompressed {} -> {}",
        compressed.len(),
        decompressed.len()
    );

    String::from_utf8(decompressed)
        .map_err(|e| rorm_error::runtime!("Fcss grpc convert to string error: {}", e))
}
