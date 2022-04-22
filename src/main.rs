mod application_window;
mod perf_data_parser;
mod profile_page;
mod profile_page_view;
mod profile_setup_page;
mod timeline_range;
mod timeline_row;
mod timeline_ticker;
mod timeline_view;

use crate::application_window::ApplicationWindow;
use adw::Application;
use gtk::prelude::{ApplicationExt, ApplicationExtManual};

fn main() {
    gio::resources_register_include!("com.github.jms55.WhatTheFn.gresource").unwrap();

    let app = Application::builder()
        .application_id("com.github.jms55.WhatTheFn")
        .build();

    app.connect_activate(|app| {
        ApplicationWindow::new(app, true);
    });

    app.run();
}
