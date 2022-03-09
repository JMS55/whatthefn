use crate::profile_tab::{ProfileTab, ProfileTabState};
use adw::traits::AdwApplicationWindowExt;
use adw::{Application, ApplicationWindow, HeaderBar, TabBar, TabPage, TabView, WindowTitle};
use glib::{clone, ObjectExt};
use gtk::prelude::GObjectPropertyExpressionExt;
use gtk::traits::{BoxExt, ButtonExt, GtkWindowExt, WidgetExt};
use gtk::{Box as BoxWidget, Button, Orientation, Stack, Widget};

pub fn new_application_window(app: &Application, create_initial_tab: bool) -> TabView {
    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(685)
        .default_height(385)
        .build();
    let window_content = BoxWidget::new(Orientation::Vertical, 0);
    let tab_view = TabView::new();
    let header_bar = HeaderBar::new();
    let header_stack = Stack::builder().hexpand(true).build();
    let window_title = WindowTitle::new("WhatTheFn", "");
    let new_tab_button_large = Button::from_icon_name("plus-large-symbolic");
    let new_tab_button_small = Button::from_icon_name("plus-symbolic");
    let tab_bar = TabBar::builder()
        .view(&tab_view)
        .end_action_widget(&new_tab_button_small)
        .css_classes(vec!["inline".to_owned()])
        .build();
    header_stack.add_named(&window_title, Some("title"));
    header_stack.add_named(&tab_bar, Some("tabs"));
    header_bar.pack_start(&new_tab_button_large);
    header_bar.set_title_widget(Some(&header_stack));
    window_content.append(&header_bar);
    window_content.append(&tab_view);
    window.set_content(Some(&window_content));

    // Create an initial tab if this window was not launched by dragging a tab out of an existing window
    if create_initial_tab {
        add_profile_tab(&tab_view);
    }

    // Sync the window subtitle with the first tab's title
    tab_view
        .property_expression("selected-page")
        .chain_property::<TabPage>("title")
        .bind(&window_title, "subtitle", Widget::NONE);

    // Show tabs in headerbar only when there is more than 1 tab
    tab_bar.connect_tabs_revealed_notify(
        clone!(@weak header_stack, @weak new_tab_button_large => move |tab_bar| {
            if tab_bar.is_tabs_revealed() {
                new_tab_button_large.hide();
                header_stack.set_visible_child_name("tabs");
            } else {
                new_tab_button_large.show();
                header_stack.set_visible_child_name("title");
            }
        }),
    );

    // Create a new window for detached tabs
    tab_view.connect_create_window(clone!(@strong app => move |_|
        Some(new_application_window(&app, false))
    ));

    // Add a new tab when either button is pressed
    new_tab_button_small.connect_clicked(clone!(@weak tab_view => move |_|
        add_profile_tab(&tab_view);
    ));
    new_tab_button_large.connect_clicked(clone!(@weak tab_view => move |_|
        add_profile_tab(&tab_view);
    ));

    window.present();
    tab_view
}

fn add_profile_tab(tab_view: &TabView) {
    // Add new tab to tab view
    let profile_tab = ProfileTab::new();
    let tab = tab_view.append(&profile_tab);
    tab.set_title("Profile Setup");
    tab_view.set_selected_page(&tab);

    // Set tab props based on the page it's displaying
    profile_tab.connect_notify_local(
        None,
        clone!(@weak tab => move |profile_tab, _| {
            match (profile_tab.state(), profile_tab.profile_name()) {
                (ProfileTabState::Setup, None) => {
                    tab.set_title("Profile Setup");
                },
                (ProfileTabState::Setup, Some(profile_name)) => {
                    tab.set_title(&format!("Profile Setup - {profile_name}"));
                },
                (ProfileTabState::SetupCompilingProgram, Some(profile_name)) => {
                    tab.set_title(&format!("Compiling - {profile_name}"));
                    tab.set_loading(true);
                },
                (ProfileTabState::SetupProfilingProgram, Some(profile_name)) => {
                    tab.set_title(&format!("Profiling - {profile_name}"));
                    tab.set_loading(true);
                },
                (ProfileTabState::LoadingProfile, Some(profile_name)) => {
                    tab.set_title(&profile_name);
                    tab.set_loading(true);
                },
                (ProfileTabState::Profile, Some(profile_name)) => {
                    tab.set_title(&profile_name);
                    tab.set_loading(true);
                    tab.set_needs_attention(!tab.is_selected());
                },
                _ => unreachable!(),
            }
        }),
    );

    // Remove needs-attention from a tab once clicked
    tab.connect_selected_notify(|tab| tab.set_needs_attention(false));
}
