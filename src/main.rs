


use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use zeke::http::{
    context::{get_context, set_context, Contextable}, 
    handler::Handler, 
    middleware::{Middleware, MiddlewareGroup}, 
    response::{new_response, set_header}, 
    router::{Route, Router},
};


#[tokio::main]
async fn main() {


    //================================================================
    // creating a router
    //================================================================

	let mut r = Router::new();

    //================================================================
    // creating a handler
    //================================================================

    pub fn handle_home() -> Handler {
        return Handler::new(|request| {
            let response = new_response(200, "<h1>Home</h1>");
            let response = set_header(response, "Content-Type", "text/html");
            return (request, response);
        });
    }

    pub fn handle_about() -> Handler {
        return Handler::new(|request| {
            let response = new_response(200, "<h1>About</h1>");
            let response = set_header(response, "Content-Type", "text/html");
            return (request, response);
        });
    }

    //================================================================
    // creating a type to represent shared state
    //================================================================

    pub enum AppContext {
        Trace,
    }

    impl Contextable for AppContext {
        fn key(&self) -> &'static str {
            match self {
                AppContext::Trace => {"TRACE"},
            }
        }
    }

    //================================================================
    // creating a middleware to track when our request starts
    //================================================================

    pub fn mw_trace() -> Middleware {
        return Middleware::new(|request| {
            let trace = HttpTrace{
                time_stamp: chrono::Utc::now().to_rfc3339(),
            };
            let trace_encoded = serde_json::to_string(&trace);
            match trace_encoded {
                Ok(trace_encoded) => {
                    set_context(request, AppContext::Trace, trace_encoded);
                    return None;
                },
                Err(_) => {
                    return Some(new_response(500, "failed to encode trace"));
                }
            }
        });
    }

    //================================================================
    // creating a middleware to log our request processing time
    //================================================================

    pub fn mw_trace_log() -> Middleware {
        return Middleware::new(|request| {
            let trace = get_context(&request.context, AppContext::Trace);
            if trace == "" {
                return Some(new_response(500, "trace not found"));
            }
            let trace: HttpTrace = serde_json::from_str(&trace).unwrap();
            let elapsed_time = trace.get_time_elapsed();
            let log_message = format!("[{}][{}][{}]", request.method, request.path, elapsed_time);
            println!("{}", log_message);
            return None;
        });
    }

    //================================================================
    // grouping middleware to reusability
    //================================================================

    pub fn mw_group_trace() -> MiddlewareGroup {
        return MiddlewareGroup::new(vec![mw_trace()], vec![mw_trace_log()]);
    }

    //================================================================
    // creating a type to track our request processing time
    //================================================================

    #[derive(Debug, Serialize, Deserialize)]
    pub struct HttpTrace {
        pub time_stamp: String,
    }

    impl HttpTrace {
        /// Prints the time elapsed since the `time_stamp` was set.
        pub fn get_time_elapsed(&self) -> String {
            if let Ok(time_set) = DateTime::parse_from_rfc3339(&self.time_stamp) {
                let time_set = time_set.with_timezone(&Utc);
                let now = Utc::now();
                let duration = now.signed_duration_since(time_set);
                let micros = duration.num_microseconds();
                match micros {
                    Some(micros) => {
                        if micros < 1000 {
                            return format!("{}µ", micros);
                        }
                    },
                    None => {

                    }
                }
                let millis = duration.num_milliseconds();
                return format!("{}ms", millis);
            } else {
                return "failed to parse time_stamp".to_string();
            }
        }
    }

    //================================================================
    // mounting handlers with middleware/outerware
    //================================================================

    // mount a handler with middleware/outerware
    r.add(Route::new("GET /", handle_home())
        .middleware(mw_trace())
        .outerware(mw_trace_log())
    );

    // mount a handler with a middleware group
    r.add(Route::new("GET /about", handle_about())
        .group(mw_group_trace())
    );

    //================================================================
    // starting the server
    //================================================================

    let err = r.serve("127.0.0.1:8080").await;
    match err {
        Some(e) => {
            println!("Error: {:?}", e);
        },
        None => {
            println!("Server closed");
        },
    }

}









