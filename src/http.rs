mod http;

pub use self::http::{Http, HttpTrait};

#[cfg(test)]
pub use self::http::MockHttpTrait;
