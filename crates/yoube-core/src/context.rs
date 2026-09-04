use crate::error::AppResult;

#[derive(Default)]
pub struct AppContext {
    // Fields are added in later tasks: youtube, downloader, filter, dnsblock,
    // account, media, settings. They are Arc<T: ServiceTrait> so commands can
    // resolve them cheaply.
}

impl AppContext {
    pub fn new() -> Self { Self::default() }
    pub fn ping(&self) -> AppResult<&'static str> { Ok("pong") }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_context_pings() {
        let c = AppContext::new();
        assert_eq!(c.ping().unwrap(), "pong");
    }
}
