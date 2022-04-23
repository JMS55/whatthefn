use glib::subclass::prelude::{ObjectImpl, ObjectSubclass};
use glib::{object_subclass, Object, ObjectExt, Properties};
use std::cell::Cell;

glib::wrapper! {
    pub struct TimelineRange(ObjectSubclass<TimelineRangePrivate>);
}

impl TimelineRange {
    pub fn new(start: u64, end: u64) -> Self {
        Object::new(&[("start", &start), ("end", &end)]).unwrap()
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
    #[property(get, construct_only)]
    start: Cell<u64>,
    #[property(get, construct_only)]
    end: Cell<u64>,
}

#[object_subclass]
impl ObjectSubclass for TimelineRangePrivate {
    const NAME: &'static str = "WtfTimelineRange";
    type Type = TimelineRange;
    type ParentType = Object;
}

impl ObjectImpl for TimelineRangePrivate {}
