use std::env::var;
use twilight_http::client::{Client, InteractionClient};

lazy_static! {
    pub static ref HTTPCLIENT: Client =
        Client::new(format!("Bot {}", var("DISCORD_TOKEN").unwrap()));
    pub static ref ICLIENT: InteractionClient<'static> =
        HTTPCLIENT.interaction(var("CLIENT_ID").unwrap().parse().unwrap());
}
