pub async fn create_token() -> hyper::Response<hyper::Body> {
    hyper::Response::builder()
        .body(hyper::Body::empty())
        .unwrap()
}
