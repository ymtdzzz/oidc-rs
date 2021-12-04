use rocket::{
    http::{hyper::header::LOCATION, Cookie, Header, Status},
    response::Responder,
    Response,
};
use time::{Duration, OffsetDateTime};

pub struct RedirectWithCookie {
    pub key: String,
    pub value: String,
    pub next: String,
}

impl<'r> Responder<'r, 'static> for RedirectWithCookie {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        // TODO: set cookie expiration
        let _exp = OffsetDateTime::now_utc()
            .checked_add(Duration::hours(12))
            .expect("failed to calculate cookie expiration");

        let cookie = Cookie::build(self.key, self.value)
            // .expires(exp)
            .finish();
        Response::build()
            .status(Status::Found)
            .header(cookie)
            .header(Header::new(LOCATION.as_str(), self.next))
            .ok()
    }
}
