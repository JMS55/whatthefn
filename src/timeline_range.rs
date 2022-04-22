use glib::subclass::prelude::{ObjectImpl, ObjectSubclass};
use glib::{
    object_subclass, Object, ObjectExt, ParamFlags, ParamSpec, ParamSpecUInt64, ToValue, Value,
};
use once_cell::sync::Lazy;
use std::cell::Cell;

glib::wrapper! {
    pub struct TimelineRange(ObjectSubclass<TimelineRangePrivate>);
}

impl Default for TimelineRange {
    fn default() -> Self {
        Object::new(&[]).unwrap()
    }
}

impl TimelineRange {
    pub fn new(start: u64, end: u64) -> Self {
        Object::new(&[("start", &start), ("end", &end)]).unwrap()
    }

    pub fn start(&self) -> u64 {
        self.property("start")
    }

    pub fn end(&self) -> u64 {
        self.property("end")
    }
}

// ------------------------------------------------------------------------------

#[derive(Default)]
pub struct TimelineRangePrivate {
    start: Cell<u64>,
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
        static PROPERTIES: Lazy<[ParamSpec; 2]> = Lazy::new(|| {
            [
                ParamSpecUInt64::new(
                    "start",
                    "Start",
                    "TODO",
                    0,
                    u64::MAX,
                    u64::default(),
                    ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                ),
                ParamSpecUInt64::new(
                    "end",
                    "End",
                    "TODO",
                    0,
                    u64::MAX,
                    u64::default(),
                    ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                ),
            ]
        });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "start" => self.start.get().to_value(),
            "end" => self.end.get().to_value(),
            _ => unreachable!(),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "start" => self.start.set(value.get().unwrap()),
            "end" => self.end.set(value.get().unwrap()),
            _ => unreachable!(),
        }
    }
}
