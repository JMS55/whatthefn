use crate::profile_page::new_profile_page;
use crate::profile_setup_page::new_profile_setup_page;
use adw::traits::BinExt;
use adw::{Bin, TabView};
use glib::clone;
use gtk::traits::WidgetExt;
use std::path::Path;

// TODO: Are we actually going to use profile_loading?

pub fn add_new_tab(tab_view: &TabView) {
    // Create tab with initial page
    let profile_tab = Bin::new();
    let profile_setup_page = new_profile_setup_page(&profile_tab);
    profile_tab.set_child(Some(&profile_setup_page));

    // Add tab to tab view
    let tab = tab_view.append(&profile_tab);
    tab_view.set_selected_page(&tab);
    tab.set_title("Profile Setup");

    // Set tab props based on the page it's displaying
    profile_tab.connect_name_notify(clone!(@weak tab => move |profile_tab| {
        match profile_tab.widget_name().split_once(",") {
            None if profile_tab.widget_name() == "setup" => {
                tab.set_title("Profile Setup");
            }
            Some(("profiling", program_name)) => {
                tab.set_title(&format!("Profiling - {program_name}"));
            }
            Some(("profile_loading", profile_name)) => {
                tab.set_title(profile_name);
                tab.set_loading(true);
            }
            Some(("profile", profile_name)) => {
                tab.set_title(profile_name);
                tab.set_loading(false);
                tab.set_needs_attention(!tab.is_selected());
            }
            _ => unreachable!()
        }
    }));

    // Remove needs-attention from a tab once clicked
    tab.connect_selected_notify(|tab| tab.set_needs_attention(false));
}

pub fn update_tab_to_setup(profile_tab: &Bin) {
    profile_tab.set_widget_name("setup");
}

pub fn update_tab_to_profiling(profile_tab: &Bin, profile_name: &str) {
    profile_tab.set_widget_name(&format!("profiling,{profile_name}"));
}

pub fn switch_tab_to_profile_page(profile_tab: &Bin, profile_path: &Path) {
    profile_tab.set_child(Some(&new_profile_page()));
    // TODO: Should be loading profile initially?
    profile_tab.set_widget_name("profile,SampleProfileTODO");
}
