use axum::http::{HeaderName, HeaderValue};
use axum_extra::headers::Header;

pub struct SessionToken(pub String);

impl Header for SessionToken {
    fn name() -> &'static axum::http::HeaderName {
        static NAME: HeaderName = HeaderName::from_static("x-session-token");
        &NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum_extra::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i axum::http::HeaderValue>,
    {
        let value = values
            .next()
            .ok_or_else(axum_extra::headers::Error::invalid)?;

        let token = value
            .to_str()
            .map_err(|_| axum_extra::headers::Error::invalid())?;

        Ok(SessionToken(token.to_string()))
    }

    fn encode<E: Extend<axum::http::HeaderValue>>(&self, values: &mut E) {
        let value =
            HeaderValue::try_from(&self.0).expect("SessionToken contains an invalid header value");

        values.extend(std::iter::once(value));
    }
}
