use crate::profile_page_view::{
    ProfilePageView, ProfilePageViewPrivatePropertiesExt, ProfilePageViewState,
};
use adw::subclass::prelude::AdwApplicationWindowImpl;
use adw::{TabBar, TabPage, TabView, WindowTitle};
use gio::{ActionGroup, ActionMap};
use glib::subclass::prelude::{
    DerivedObjectProperties, ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt,
    ObjectSubclassIsExt,
};
use glib::subclass::InitializingObject;
use glib::{clone, object_subclass, IsA, Object, ObjectExt, ParamSpec, Properties, Value};
use gtk::prelude::{GObjectPropertyExpressionExt, InitializingWidgetExt};
use gtk::subclass::prelude::{
    ApplicationWindowImpl, CompositeTemplateCallbacksClass, CompositeTemplateClass, TemplateChild,
    WidgetClassSubclassExt, WidgetImpl, WindowImpl,
};
use gtk::traits::{GtkWindowExt, WidgetExt};
use gtk::{
    template_callbacks, Accessible, Application, Buildable, Button, CompositeTemplate,
    ConstraintTarget, Native, Root, ShortcutManager, Stack, Widget, Window,
};
use std::cell::Cell;

glib::wrapper! {
    pub struct ApplicationWindow(ObjectSubclass<ApplicationWindowPrivate>)
    @extends adw::ApplicationWindow, gtk::ApplicationWindow, Window, Widget,
    @implements ActionGroup, ActionMap, Accessible, Buildable, ConstraintTarget, Native, Root, ShortcutManager;
}

impl ApplicationWindow {
    pub fn new<A: IsA<Application>>(application: &A, create_initial_tab: bool) -> Self {
        Object::new(&[
            ("application", application),
            ("create-initial-tab", &create_initial_tab),
        ])
        .unwrap()
    }
}

// ------------------------------------------------------------------------------

#[derive(CompositeTemplate, Properties, Default)]
#[template(resource = "/com/github/jms55/WhatTheFn/ui/application_window.ui")]
pub struct ApplicationWindowPrivate {
    #[template_child]
    header_stack: TemplateChild<Stack>,
    #[template_child]
    title: TemplateChild<WindowTitle>,
    #[template_child]
    tab_view: TemplateChild<TabView>,
    #[template_child]
    new_tab_button: TemplateChild<Button>,

    #[property(get, set, construct_only)]
    create_initial_tab: Cell<bool>,
}

#[template_callbacks]
impl ApplicationWindowPrivate {
    #[template_callback]
    fn add_new_tab(&self) {
        // Add new tab to tab view
        let page_view = ProfilePageView::new();
        let tab = self.tab_view.append(&page_view);
        set_tab_properties(&tab, &page_view);
        self.tab_view.set_selected_page(&tab);

        // Update tab properties whenever its ProfilePageView changes
        page_view.connect_notify_local(
            None,
            clone!(@weak tab => move |page_view, _| set_tab_properties(&tab, page_view)),
        );

        // Remove needs-attention from a tab once clicked
        tab.connect_selected_notify(|tab| tab.set_needs_attention(false));
    }

    // Show tabs in headerbar only when there is more than 1 tab
    #[template_callback]
    fn swap_header_widgets(&self, _: &ParamSpec, tab_bar: &TabBar) {
        if tab_bar.is_tabs_revealed() {
            self.new_tab_button.hide();
            self.header_stack.set_visible_child_name("tabs");
        } else {
            self.new_tab_button.show();
            self.header_stack.set_visible_child_name("title");
        }
    }

    #[template_callback]
    fn create_window_for_detached_tab(&self) -> TabView {
        let application = self.instance().application().unwrap();
        let window = ApplicationWindow::new(&application, false);
        window.imp().tab_view.get()
    }
}

// Set tab properties based on the page its ProfilePageView is displaying
fn set_tab_properties(tab: &TabPage, page_view: &ProfilePageView) {
    match (page_view.state(), page_view.profile_name()) {
        (ProfilePageViewState::Setup, None) => {
            tab.set_title("Profile Setup");
        }
        (ProfilePageViewState::Setup, Some(profile_name)) => {
            tab.set_title(&format!("Profile Setup - {profile_name}"));
        }
        (ProfilePageViewState::SetupCompilingProgram, Some(profile_name)) => {
            tab.set_title(&format!("Compiling - {profile_name}"));
            tab.set_loading(true);
        }
        (ProfilePageViewState::SetupProfilingProgram, Some(profile_name)) => {
            tab.set_title(&format!("Profiling - {profile_name}"));
            tab.set_loading(true);
        }
        (ProfilePageViewState::LoadingProfile, Some(profile_name)) => {
            tab.set_title(&profile_name);
            tab.set_loading(true);
        }
        (ProfilePageViewState::Profile, Some(profile_name)) => {
            tab.set_title(&profile_name);
            tab.set_loading(true);
            tab.set_needs_attention(!tab.is_selected());
        }
        _ => unreachable!(),
    }
}

#[object_subclass]
impl ObjectSubclass for ApplicationWindowPrivate {
    const NAME: &'static str = "WtfApplicationWindow";
    type Type = ApplicationWindow;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(this: &InitializingObject<Self>) {
        this.init_template();
    }
}

impl ObjectImpl for ApplicationWindowPrivate {
    fn constructed(&self, this: &Self::Type) {
        self.parent_constructed(this);

        // Sync the window subtitle with the first tab's title
        // TODO: Remove this once blueprints gets expression support
        self.tab_view
            .property_expression("selected-page")
            .chain_property::<TabPage>("title")
            .bind(&self.title.get(), "subtitle", Widget::NONE);

        if this.create_initial_tab() {
            self.add_new_tab();
        }

        this.present();
    }

    fn properties() -> &'static [ParamSpec] {
        Self::derived_properties()
    }
    fn set_property(&self, this: &Self::Type, id: usize, value: &Value, pspec: &ParamSpec) {
        Self::derived_set_property(self, this, id, value, pspec).unwrap();
    }
    fn property(&self, this: &Self::Type, id: usize, pspec: &ParamSpec) -> Value {
        Self::derived_property(self, this, id, pspec).unwrap()
    }
}

impl WidgetImpl for ApplicationWindowPrivate {}
impl WindowImpl for ApplicationWindowPrivate {}
impl ApplicationWindowImpl for ApplicationWindowPrivate {}
impl AdwApplicationWindowImpl for ApplicationWindowPrivate {}
