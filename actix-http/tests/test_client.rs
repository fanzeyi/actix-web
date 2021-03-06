use actix_service::NewService;
use bytes::{Bytes, BytesMut};
use futures::future::{self, ok};
use futures::{Future, Stream};

use actix_http::{
    error::PayloadError, http, HttpMessage, HttpService, Request, Response,
};
use actix_http_test::TestServer;

const STR: &str = "Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World";

fn load_body<S>(stream: S) -> impl Future<Item = BytesMut, Error = PayloadError>
where
    S: Stream<Item = Bytes, Error = PayloadError>,
{
    stream.fold(BytesMut::new(), move |mut body, chunk| {
        body.extend_from_slice(&chunk);
        Ok::<_, PayloadError>(body)
    })
}

#[test]
fn test_h1_v2() {
    env_logger::init();
    let mut srv = TestServer::new(move || {
        HttpService::build()
            .finish(|_| future::ok::<_, ()>(Response::Ok().body(STR)))
            .map(|_| ())
    });
    let response = srv.block_on(srv.get().send()).unwrap();
    assert!(response.status().is_success());

    let request = srv.get().header("x-test", "111").send();
    let mut response = srv.block_on(request).unwrap();
    assert!(response.status().is_success());

    // read response
    let bytes = srv.block_on(load_body(response.take_payload())).unwrap();
    assert_eq!(bytes, Bytes::from_static(STR.as_ref()));

    let mut response = srv.block_on(srv.post().send()).unwrap();
    assert!(response.status().is_success());

    // read response
    let bytes = srv.block_on(load_body(response.take_payload())).unwrap();
    assert_eq!(bytes, Bytes::from_static(STR.as_ref()));
}

#[test]
fn test_connection_close() {
    let mut srv = TestServer::new(move || {
        HttpService::build()
            .finish(|_| ok::<_, ()>(Response::Ok().body(STR)))
            .map(|_| ())
    });
    let response = srv.block_on(srv.get().close_connection().send()).unwrap();
    assert!(response.status().is_success());
}

#[test]
fn test_with_query_parameter() {
    let mut srv = TestServer::new(move || {
        HttpService::build()
            .finish(|req: Request| {
                if req.uri().query().unwrap().contains("qp=") {
                    ok::<_, ()>(Response::Ok().finish())
                } else {
                    ok::<_, ()>(Response::BadRequest().finish())
                }
            })
            .map(|_| ())
    });

    let request = srv.request(http::Method::GET, srv.url("/?qp=5")).send();
    let response = srv.block_on(request).unwrap();
    assert!(response.status().is_success());
}
