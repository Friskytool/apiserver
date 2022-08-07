pub mod guilds;
pub mod plugins;
pub mod tags;
use rocket::routes;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        guilds::get_guild,
        guilds::get_roles,
        guilds::get_channels,
        plugins::get_settings,
        plugins::edit_settings,
        plugins::get_plugins,
        plugins::edit_plugins,
        tags::get_tags,
        tags::create_tag,
        tags::update_tag,
        tags::delete_tag
    ]
}
