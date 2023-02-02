#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DefRange {
    #[prost(int32, tag = "1")]
    pub start: i32,
    #[prost(int32, tag = "2")]
    pub end: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseStatus {
    /// 0 表示成功，!0 表示失败
    #[prost(int32, tag = "1")]
    pub return_code: i32,
    #[prost(string, tag = "2")]
    pub return_string: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TblLsrp {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetallLsrp {
    /// 0 表示成功，!0 表示失败
    #[prost(int32, tag = "1")]
    pub return_code: i32,
    #[prost(string, tag = "2")]
    pub return_string: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "6")]
    pub entries: ::prost::alloc::vec::Vec<TblLsrp>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseLsrp {
    /// 0 表示成功，!0 表示失败
    #[prost(int32, tag = "3")]
    pub return_code: i32,
    #[prost(string, tag = "4")]
    pub return_string: ::prost::alloc::string::String,
    /// true 表示 key 存在，false 表示不存在
    #[prost(bool, tag = "1")]
    pub flag: bool,
    #[prost(message, optional, tag = "2")]
    pub entry: ::core::option::Option<TblLsrp>,
}
/// Generated client implementations.
pub mod lsrp_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct LsrpClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl LsrpClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> LsrpClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> LsrpClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            LsrpClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        pub async fn add(
            &mut self,
            request: impl tonic::IntoRequest<super::TblLsrp>,
        ) -> Result<tonic::Response<super::ResponseStatus>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/api.lsrp/add");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn del(
            &mut self,
            request: impl tonic::IntoRequest<super::TblLsrp>,
        ) -> Result<tonic::Response<super::ResponseStatus>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/api.lsrp/del");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get(
            &mut self,
            request: impl tonic::IntoRequest<super::TblLsrp>,
        ) -> Result<tonic::Response<super::ResponseLsrp>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/api.lsrp/get");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn getall(
            &mut self,
            request: impl tonic::IntoRequest<super::DefRange>,
        ) -> Result<tonic::Response<super::GetallLsrp>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/api.lsrp/getall");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod lsrp_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with LsrpServer.
    #[async_trait]
    pub trait Lsrp: Send + Sync + 'static {
        async fn add(
            &self,
            request: tonic::Request<super::TblLsrp>,
        ) -> Result<tonic::Response<super::ResponseStatus>, tonic::Status>;
        async fn del(
            &self,
            request: tonic::Request<super::TblLsrp>,
        ) -> Result<tonic::Response<super::ResponseStatus>, tonic::Status>;
        async fn get(
            &self,
            request: tonic::Request<super::TblLsrp>,
        ) -> Result<tonic::Response<super::ResponseLsrp>, tonic::Status>;
        async fn getall(
            &self,
            request: tonic::Request<super::DefRange>,
        ) -> Result<tonic::Response<super::GetallLsrp>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct LsrpServer<T: Lsrp> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Lsrp> LsrpServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for LsrpServer<T>
    where
        T: Lsrp,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/api.lsrp/add" => {
                    #[allow(non_camel_case_types)]
                    struct addSvc<T: Lsrp>(pub Arc<T>);
                    impl<T: Lsrp> tonic::server::UnaryService<super::TblLsrp> for addSvc<T> {
                        type Response = super::ResponseStatus;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TblLsrp>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).add(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = addSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/api.lsrp/del" => {
                    #[allow(non_camel_case_types)]
                    struct delSvc<T: Lsrp>(pub Arc<T>);
                    impl<T: Lsrp> tonic::server::UnaryService<super::TblLsrp> for delSvc<T> {
                        type Response = super::ResponseStatus;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TblLsrp>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).del(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = delSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/api.lsrp/get" => {
                    #[allow(non_camel_case_types)]
                    struct getSvc<T: Lsrp>(pub Arc<T>);
                    impl<T: Lsrp> tonic::server::UnaryService<super::TblLsrp> for getSvc<T> {
                        type Response = super::ResponseLsrp;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TblLsrp>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = getSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/api.lsrp/getall" => {
                    #[allow(non_camel_case_types)]
                    struct getallSvc<T: Lsrp>(pub Arc<T>);
                    impl<T: Lsrp> tonic::server::UnaryService<super::DefRange> for getallSvc<T> {
                        type Response = super::GetallLsrp;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DefRange>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).getall(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = getallSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: Lsrp> Clone for LsrpServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Lsrp> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Lsrp> tonic::transport::NamedService for LsrpServer<T> {
        const NAME: &'static str = "api.lsrp";
    }
}
