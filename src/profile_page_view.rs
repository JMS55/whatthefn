use crate::profile_page::new_profile_page;
use crate::profile_setup_page::ProfileSetupPage;
use adw::subclass::prelude::BinImpl;
use adw::traits::BinExt;
use adw::Bin;
use glib::subclass::prelude::{DerivedObjectProperties, ObjectImpl, ObjectImplExt, ObjectSubclass};
use glib::{
    object_subclass, Enum as GEnum, Object, ObjectExt, ParamSpec, Properties, StaticType, Value,
};
use gtk::subclass::prelude::WidgetImpl;
use gtk::traits::WidgetExt;
use gtk::{Accessible, Buildable, ConstraintTarget, Widget};
use std::cell::RefCell;
use std::path::Path;

glib::wrapper! {
    pub struct ProfilePageView(ObjectSubclass<ProfilePageViewPrivate>)
    @extends Bin, Widget,
    @implements Accessible, Buildable, ConstraintTarget;
}

impl ProfilePageView {
    pub fn new() -> Self {
        Object::new(&[]).unwrap()
    }

    pub fn switch_to_profile_page(&self, profile_path: &Path) {
        todo!()
    }

    pub fn set_data(&self, state: ProfilePageViewState, profile_name: &str) {
        self.set_state(state);
        self.set_profile_name(Some(profile_name.to_owned()));
    }
}

#[derive(GEnum, Clone, Copy, PartialEq, Eq)]
#[enum_type(name = "WtfProfilePageViewState")]
pub enum ProfilePageViewState {
    Setup,
    SetupCompilingProgram,
    SetupProfilingProgram,
    LoadingProfile,
    Profile,
}

impl Default for ProfilePageViewState {
    fn default() -> Self {
        Self::Setup
    }
}

// ------------------------------------------------------------------------------

#[derive(Properties, Default)]
pub struct ProfilePageViewPrivate {
    #[property(get, set, builder(ProfilePageViewState::static_type()))]
    state: RefCell<ProfilePageViewState>,
    #[property(get, set)]
    profile_name: RefCell<Option<String>>,
}

#[object_subclass]
impl ObjectSubclass for ProfilePageViewPrivate {
    const NAME: &'static str = "WtfProfilePageView";
    type Type = ProfilePageView;
    type ParentType = Bin;
}

impl ObjectImpl for ProfilePageViewPrivate {
    fn constructed(&self, this: &Self::Type) {
        self.parent_constructed(this);

        this.set_child(Some(&new_profile_page())); // TODO: ProfileSetupPage

        this.set_margin_top(18);
        this.set_margin_bottom(18);
        this.set_margin_start(18);
        this.set_margin_end(18);
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

impl WidgetImpl for ProfilePageViewPrivate {}
impl BinImpl for ProfilePageViewPrivate {}
