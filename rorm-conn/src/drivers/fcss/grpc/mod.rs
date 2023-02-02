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
                value: value.to_string()
            }
        );

        Ok(())
    }

    pub async fn del<K: ToString + Display>(&self, key: K) -> Result<()> {
        log::trace!("Fcss grpc del `{}``", key);

        grpc_call!(
            self,
            del,
            api::TblLsrp {
                key: key.to_string(),
                value: String::new(),
            }
        );

        Ok(())
    }

    pub async fn get<K: ToString + Display>(&self, key: K) -> Result<String> {
        log::trace!("Fcss grpc get `{}``", key);

        let res = grpc_call!(
            self,
            get,
            api::TblLsrp {
                key: key.to_string(),
                value: String::new(),
            }
        );

        Ok(res
            .entry
            .ok_or(rorm_error::runtime!("Fcss grpc get return is none"))?
            .value)
    }

    pub async fn list(&self) -> Result<Vec<(String, String)>> {
        log::trace!("Fcss grpc list");

        let res = grpc_call!(self, getall, api::DefRange { start: 0, end: -1 });

        Ok(res
            .entries
            .into_iter()
            .map(|entry| (entry.key, entry.value))
            .collect())
    }
}
