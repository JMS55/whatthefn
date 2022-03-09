mod application_window;
mod perf_data_parser;
mod profile_page;
mod profile_page_view;
mod profile_setup_page;

use crate::application_window::new_application_window;
use adw::Application;
use gtk::prelude::{ApplicationExt, ApplicationExtManual};

fn main() {
    gio::resources_register_include!("com.github.jms55.WhatTheFn.gresource").unwrap();

    let app = Application::builder()
        .application_id("com.github.jms55.WhatTheFn")
        .build();

    app.connect_activate(|app| {
        new_application_window(app, true);
    });

    app.run();
}
