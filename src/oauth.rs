use crate::models::User;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use rocket::http::{Cookie, CookieJar};
use rocket::response::{Flash, Redirect};
use std::env::var;
//use url::Url;

fn get_client() -> BasicClient {
    BasicClient::new(
        ClientId::new(var("CLIENT_ID").unwrap()),
        Some(ClientSecret::new(var("CLIENT_SECRET").unwrap())),
        AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string()).unwrap(),
        Some(TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap()),
    )
    // Set the URL the user will be redirected to after the authorization process.
    .set_redirect_uri(RedirectUrl::new("http://localhost:8000/oauth/callback".to_string()).unwrap())
}

#[get("/login")]
async fn login() -> Redirect {
    let client = get_client();
    let (auth_url, _) = client
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
    let oauth_client = get_client();

    let result = oauth_client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await
        .ok();

    if let Some(response) = result {
        let user = User {
            access_token: response.access_token().clone(),
            token_type: response.token_type().clone(),
            expires_in: response.expires_in().unwrap().clone(),
            refresh_token: response.refresh_token().unwrap().clone(),
            scopes: response.scopes().unwrap().clone(),
        };

        jar.add_private(Cookie::build("user", serde_json::to_string(&user).unwrap()).finish());

        Flash::success(Redirect::to("/callback"), "Logged in")
    } else {
        Flash::error(Redirect::to("/callback"), "Failed to log in")
    }
}

#[get("/callback?<error>&<error_description>&<state>", rank = 2)]
#[allow(unused_variables)]
async fn callback_error(
    error: String,
    error_description: String,
    state: String,
) -> Flash<Redirect> {
    Flash::error(Redirect::to("/callback"), "Error Logging In")
}

#[get("/logout")]
async fn logout(cookies: &CookieJar<'_>) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named("user"));
    Flash::success(Redirect::to("/"), "Logged out")
}

pub fn routes() -> Vec<rocket::Route> {
    routes![login, callback_success, callback_error, logout]
}
