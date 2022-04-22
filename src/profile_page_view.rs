use crate::profile_setup_page::ProfileSetupPage;
use adw::subclass::prelude::BinImpl;
use adw::traits::BinExt;
use adw::Bin;
use glib::subclass::prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass};
use glib::{
    object_subclass, Enum as GEnum, Object, ObjectExt, ParamFlags, ParamSpec, ParamSpecEnum,
    ParamSpecString, StaticType, ToValue, Value,
};
use gtk::subclass::prelude::WidgetImpl;
use gtk::traits::WidgetExt;
use gtk::{Accessible, Buildable, ConstraintTarget, Widget};
use once_cell::sync::Lazy;
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
        self.set_profile_name(profile_name);
        self.set_state(state);
    }

    pub fn profile_name(&self) -> Option<String> {
        self.property("profile-name")
    }

    pub fn set_profile_name(&self, profile_name: &str) {
        self.set_property("profile-name", profile_name);
    }

    pub fn state(&self) -> ProfilePageViewState {
        self.property("state")
    }

    pub fn set_state(&self, state: ProfilePageViewState) {
        self.set_property("state", state);
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

#[derive(Default)]
pub struct ProfilePageViewPrivate {
    state: RefCell<ProfilePageViewState>,
    profile_name: RefCell<Option<String>>,
}

#[object_subclass]
impl ObjectSubclass for ProfilePageViewPrivate {
    const NAME: &'static str = "WtfProfilePageView";
    type Type = ProfilePageView;
    type ParentType = Bin;
}

impl ObjectImpl for ProfilePageViewPrivate {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        obj.set_child(Some(&ProfileSetupPage::new()));

        obj.set_margin_top(18);
        obj.set_margin_bottom(18);
        obj.set_margin_start(18);
        obj.set_margin_end(18);
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<[ParamSpec; 2]> = Lazy::new(|| {
            [
                ParamSpecEnum::new(
                    "state",
                    "State",
                    "Which page the view is showing and what it is doing",
                    ProfilePageViewState::static_type(),
                    ProfilePageViewState::default() as i32,
                    ParamFlags::READWRITE,
                ),
                ParamSpecString::new(
                    "profile-name",
                    "ProfileName",
                    "Name of the profile the view is showing",
                    None,
                    ParamFlags::READWRITE,
                ),
            ]
        });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "state" => self.state.borrow().to_value(),
            "profile-name" => self.profile_name.borrow().to_value(),
            _ => unreachable!(),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "state" => *self.state.borrow_mut() = value.get().unwrap(),
            "profile-name" => *self.profile_name.borrow_mut() = value.get().unwrap(),
            _ => unreachable!(),
        }
    }
}

impl WidgetImpl for ProfilePageViewPrivate {}
impl BinImpl for ProfilePageViewPrivate {}
