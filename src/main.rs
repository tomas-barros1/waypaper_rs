use adw::prelude::*;
use std::env;

mod app;
mod services;

rust_i18n::i18n!("src/locale", fallback = "en");

fn main() {
    set_system_locale();
    let args: Vec<String> = env::args().skip(1).collect();
    let cache = services::cache::Cache::load_default();
    if args.iter().any(|arg| arg == "--restore") {
        if let Err(error) = services::wallpaper_service::WallpaperService::new(cache).restore() {
            eprintln!("waypaper-rs: {error}");
            std::process::exit(1);
        }
        return;
    }
    if let Some(folder) = args
        .iter()
        .position(|arg| arg == "--folder")
        .and_then(|i| args.get(i + 1))
    {
        if let Err(error) =
            services::wallpaper_service::WallpaperService::new(cache).set_folder(folder.into())
        {
            eprintln!("waypaper-rs: {error}");
            std::process::exit(1);
        }
        return;
    }
    let application = adw::Application::builder()
        .application_id("io.github.waypaper_rs")
        .build();
    application.connect_activate(|application| app::build_ui(application));
    application.run();
}

fn set_system_locale() {
    // Keep the usual precedence, while accepting common desktop locale forms
    // such as pt_BR.UTF-8 and pt-BR. Locale files use rust-i18n's pt_br stem.
    let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|name| env::var(name).ok())
        .find(|value| !value.is_empty() && *value != "C" && *value != "C.UTF-8");
    if let Some(locale) = locale {
        let locale = locale
            .split('.')
            .next()
            .unwrap_or(&locale)
            .replace('-', "_");
        let locale = locale.to_ascii_lowercase();
        rust_i18n::set_locale(&locale);
    }
}
