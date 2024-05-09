use std::{collections::HashMap, sync::Arc};
use std::io::Error;

use tokio::sync::Mutex;


use crate::http::middleware::{Middlewares, MiddlewareMutex, MiddlewareGroup};
use crate::http::handler::Handler;
use crate::http::socket::connect_socket;

use super::middleware;


pub type RouteHandler = (Handler, Middlewares, Middlewares);

pub type Routes = HashMap<&'static str, Arc<Mutex<RouteHandler>>>;

pub struct Router {
    pub routes: Routes,
}

impl Router {
    pub fn new() -> Router {
        Router {
            routes: HashMap::new(),
        }
    }
    pub fn add(self: &mut Router, route: Route) -> &mut Router {
        let handler: RouteHandler = (route.handler, route.middlewares, route.outerwares);
        let handler_mutex = Arc::new(Mutex::new(handler));
        self.routes.insert(route.path, handler_mutex);
        return self;
    }
    pub async fn serve(self: Router, addr: &str) -> Option<Error> {
        let listener = tokio::net::TcpListener::bind(&addr).await;
        let router: Arc<Router> = Arc::new(self);
        match listener {
            Ok(ref listener) => {
                loop {
                    let router: Arc<Router> = Arc::clone(&router); // TODO: is cloning the router bad?
                    connect_socket(listener, router).await; 
                }
            },
            Err(e) => {
                return Some(e);
            },
        }
    }
}

pub struct Route {
    pub path: &'static str,
    pub handler: Handler,
    pub middlewares: Middlewares,
    pub outerwares: Middlewares,
}

impl Route {
    pub fn new(path: &'static str, handler: Handler) -> Route {
        let route = Route{
            path: path,
            handler: handler,
            middlewares: vec![],
            outerwares: vec![],
        };
        return route;
    }
    pub fn middleware(mut self: Route, middleware: MiddlewareMutex) -> Self {
        self.middlewares.push(middleware);
        return self;
    }
    pub fn outerware(mut self: Route, outerware: MiddlewareMutex) -> Self {
        self.outerwares.push(outerware);
        return self;
    }
    pub fn group(mut self: Route, middleware_group: MiddlewareGroup) -> Self {
        for middleware in middleware_group.middlewares {
            self.middlewares.push(middleware);
        }
        for outerware in middleware_group.outerwares {
            self.outerwares.push(outerware);
        }
        return self;
    }
    pub fn clone(self: &Route) -> Route {
        let route = Route{
            path: self.path,
            handler: self.handler.clone(),
            middlewares: self.middlewares.clone(),
            outerwares: self.outerwares.clone(),
        };
        return route;
    }
}






