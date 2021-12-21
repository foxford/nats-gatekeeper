pub async fn read_options() -> hyper::Response<hyper::Body> {
    hyper::Response::builder()
        .body(hyper::Body::empty())
        .unwrap()
}

pub mod nats;
