
use tokio::sync::RwLock;
use std::sync::Arc;
use futures::future::BoxFuture;

use crate::http::request::Request;
use crate::http::response::Response;

pub struct Handler {
    pub func: Arc<RwLock<dyn Fn(Request) -> BoxFuture<'static, (Request, Response)> + Send + Sync + 'static>>,
}

impl Handler {
    pub fn new<F>(f: F) -> Handler
    where
        F: Fn(Request) -> BoxFuture<'static, (Request, Response)> + Send + Sync + 'static,
    {
        Handler {
            func: Arc::new(RwLock::new(f)),
        }
    }
}