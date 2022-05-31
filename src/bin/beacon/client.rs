use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;

pub type ClientType = hyper::Client<HttpsConnector<HttpConnector>>;
pub fn make_client() -> ClientType {
    let https = HttpsConnector::new();
    hyper::Client::builder().build::<_, hyper::Body>(https)
}
