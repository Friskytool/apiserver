use crate::models::User;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use rocket::http::{Cookie, CookieJar};
use rocket::response::{Flash, Redirect};
use rocket::routes;
use std::env::var;
//use url::Url;

lazy_static!(
    static ref CLIENT: BasicClient = BasicClient::new(
        ClientId::new(var("CLIENT_ID").unwrap()),
        Some(ClientSecret::new(var("CLIENT_SECRET").unwrap())),
        AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string()).unwrap(),
        Some(TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap()),
    )
    // Set the URL the user will be redirected to after the authorization process.
    .set_redirect_uri(RedirectUrl::new(var("REDIRECT_URI").unwrap()).unwrap());
);

#[get("/login")]
async fn login() -> Redirect {
    let (auth_url, _) = CLIENT
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        .add_scope(Scope::new("guilds".to_string()))
        .add_scope(Scope::new("identify".to_string()))
        .url();

    Redirect::to(auth_url.to_string())
}

// ?error=access_denied&error_description=The resource owner or authorization server denied the request&state=9onkpNMtwTDxBoSYWFSxFc0kJVUbsS
// ?code=t8SYH4qhmHLqGamoEQr1gJucZCdF44&state=9onkpNMtwTDxBoSYWFSxFc0kJVUbsS

#[get("/callback?<code>&<_state>", rank = 1)]
async fn callback_success(
    code: String,
    _state: Option<String>,
    jar: &CookieJar<'_>,
) -> Flash<Redirect> {
    println!("1");
    let result = CLIENT
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await
        .map(Option::Some)
        .unwrap_or_else(|e| {
            println!("1.5");
            dbg!(e);
            None
        });
    println!("2");
    if let Some(response) = result {
        let user = User {
            access_token: response.access_token().clone(),
            token_type: response.token_type().clone(),
            expires_in: response.expires_in().unwrap().clone(),
            refresh_token: response.refresh_token().unwrap().clone(),
            scopes: response.scopes().unwrap().clone(),
        };
        println!("3");
        jar.add_private(Cookie::build("user", serde_json::to_string(&user).unwrap()).finish());
        println!("4");
        Flash::success(Redirect::to("/static/callback.html"), "Logged in")
    } else {
        println!("5");
        Flash::error(Redirect::to("/static/callback.html"), "Failed to log in")
    }
}

#[get("/callback?<error>&<error_description>&<state>", rank = 2)]
#[allow(unused_variables)]
async fn callback_error(
    error: String,
    error_description: String,
    state: String,
) -> Flash<Redirect> {
    Flash::error(Redirect::to("/static/callback.html"), "Error Logging In")
}

#[get("/logout")]
async fn logout(cookies: &CookieJar<'_>) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named("user"));
    Flash::success(Redirect::to("/"), "Logged out")
}

#[get("/invite?<guild_id>")]
async fn invite(guild_id: u64) -> Redirect {
    Redirect::to(format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&scope=bot%20applications.commands&permissions=1611000937&redirect_uri=https%3A%2F%2Fdashboard.squid.pink%2Fapi%2Foauth%2Fcallback&guild_id={}&response_type=code",
        var("CLIENT_ID").unwrap(),
        guild_id
    ))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![login, callback_success, callback_error, logout, invite]
}
