use crate::profile_setup_page::new_profile_setup_page;
use adw::subclass::prelude::{BinImpl, ObjectImpl, ObjectSubclass};
use adw::traits::BinExt;
use adw::Bin;
use glib::{
    object_subclass, Enum as GEnum, Object, ObjectExt, ParamFlags, ParamSpec, ParamSpecEnum,
    ParamSpecString, StaticType, ToValue, Value,
};
use gtk::subclass::prelude::WidgetImpl;
use gtk::{Accessible, Buildable, ConstraintTarget, Widget};
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::path::Path;

glib::wrapper! {
    pub struct ProfileTab(ObjectSubclass<ProfileTabPrivate>)
    @extends Bin, Widget,
    @implements Accessible, Buildable, ConstraintTarget;
}

impl ProfileTab {
    pub fn new() -> Self {
        let profile_tab = Object::new(&[]).unwrap();
        let initial_page = new_profile_setup_page(&profile_tab);
        profile_tab.set_child(Some(&initial_page));
        profile_tab
    }

    pub fn switch_to_profile_page(&self, profile_path: &Path) {
        todo!()
    }

    pub fn set_data(&self, state: ProfileTabState, profile_name: &str) {
        self.set_profile_name(profile_name);
        self.set_state(state);
    }

    pub fn profile_name(&self) -> Option<String> {
        self.property("profile-name")
    }

    pub fn set_profile_name(&self, profile_name: &str) {
        self.set_property("profile-name", profile_name);
    }

    pub fn state(&self) -> ProfileTabState {
        self.property("state")
    }

    pub fn set_state(&self, state: ProfileTabState) {
        self.set_property("state", state);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, GEnum)]
#[enum_type(name = "WtfProfileTabState")]
pub enum ProfileTabState {
    Setup,
    SetupCompilingProgram,
    SetupProfilingProgram,
    LoadingProfile,
    Profile,
}

impl Default for ProfileTabState {
    fn default() -> Self {
        Self::Setup
    }
}

// ------------------------------------------------------------------------------

#[derive(Default)]
pub struct ProfileTabPrivate {
    state: RefCell<ProfileTabState>,
    profile_name: RefCell<Option<String>>,
}

#[object_subclass]
impl ObjectSubclass for ProfileTabPrivate {
    const NAME: &'static str = "WtfProfileTab";
    type Type = ProfileTab;
    type ParentType = Bin;
    type Interfaces = ();
}

impl ObjectImpl for ProfileTabPrivate {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<[ParamSpec; 2]> = Lazy::new(|| {
            [
                ParamSpecEnum::new(
                    "state",
                    "State",
                    "Which page the tab is on and what it is doing",
                    ProfileTabState::static_type(),
                    ProfileTabState::default() as i32,
                    ParamFlags::READWRITE,
                ),
                ParamSpecString::new(
                    "profile-name",
                    "ProfileName",
                    "Name of the profile the tab is operating on",
                    None,
                    ParamFlags::READWRITE,
                ),
            ]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "state" => *self.state.borrow_mut() = value.get().unwrap(),
            "profile-name" => *self.profile_name.borrow_mut() = value.get().unwrap(),
            _ => unreachable!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "state" => self.state.borrow().to_value(),
            "profile-name" => self.profile_name.borrow().to_value(),
            _ => unreachable!(),
        }
    }
}

impl WidgetImpl for ProfileTabPrivate {}
impl BinImpl for ProfileTabPrivate {}
