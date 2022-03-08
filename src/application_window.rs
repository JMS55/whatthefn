use crate::profile_tab::add_new_tab;
use adw::traits::AdwApplicationWindowExt;
use adw::{Application, ApplicationWindow, HeaderBar, TabBar, TabPage, TabView, WindowTitle};
use glib::clone;
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

    if create_initial_tab {
        add_new_tab(&tab_view);
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
        add_new_tab(&tab_view);
    ));
    new_tab_button_large.connect_clicked(clone!(@weak tab_view => move |_|
        add_new_tab(&tab_view);
    ));

    window.present();
    tab_view
}
