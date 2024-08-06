use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::extract::Request;
use axum::response::Response;
use tokio::time::Instant;
use tower::{Layer, Service};

/*
    write middle using tower Service
*/

const SERVER_TIME_HEADER: &str = "X-Server-Time";

#[derive(Clone)]
pub struct ServerTimeLayer;

impl<S> Layer<S> for ServerTimeLayer {
    type Service = ServerTimeMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ServerTimeMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct ServerTimeMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for ServerTimeMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let start = Instant::now();
        let future = self.inner.call(request);
        Box::pin(async move {
            let mut res: Response = future.await?;
            let elapsed = start.elapsed().as_millis().to_string();
            res.headers_mut()
                .insert(SERVER_TIME_HEADER, elapsed.parse().unwrap());
            Ok(res)
        })
    }
}
