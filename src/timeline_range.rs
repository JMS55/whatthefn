use glib::subclass::prelude::{DerivedObjectProperties, ObjectImpl, ObjectSubclass};
use glib::{object_subclass, Object, ObjectExt, ParamSpec, Properties, Value};
use std::cell::Cell;

glib::wrapper! {
    pub struct TimelineRange(ObjectSubclass<TimelineRangePrivate>);
}

impl TimelineRange {
    pub fn new(start: u64, end: u64) -> Self {
        Object::new(&[("start", &start), ("end", &end)]).unwrap()
    }

    pub fn duration(&self) -> u64 {
        self.end() - self.start()
    }

    pub fn contains(&self, timestamp: u64) -> bool {
        (self.start()..=self.end()).contains(&timestamp)
    }
}

impl Default for TimelineRange {
    fn default() -> Self {
        Object::new(&[]).unwrap()
    }
}

// ------------------------------------------------------------------------------

#[derive(Properties, Default)]
pub struct TimelineRangePrivate {
    #[property(get, set, construct_only)]
    start: Cell<u64>,
    #[property(get, set, construct_only)]
    end: Cell<u64>,
}

#[object_subclass]
impl ObjectSubclass for TimelineRangePrivate {
    const NAME: &'static str = "WtfTimelineRange";
    type Type = TimelineRange;
    type ParentType = Object;
}

impl ObjectImpl for TimelineRangePrivate {
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
