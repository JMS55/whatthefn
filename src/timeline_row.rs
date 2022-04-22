use glib::subclass::prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass};
use glib::{
    object_subclass, Object, ObjectExt, ParamFlags, ParamSpec, ParamSpecUInt64, ToValue, Value,
};
use gtk::graphene::Rect;
use gtk::subclass::prelude::{WidgetImpl, WidgetImplExt};
use gtk::traits::{StyleContextExt, WidgetExt};
use gtk::{Accessible, Buildable, ConstraintTarget, Orientation, Snapshot, Widget};
use once_cell::sync::Lazy;
use std::cell::Cell;

glib::wrapper! {
    pub struct TimelineRow(ObjectSubclass<TimelineRowPrivate>)
    @extends Widget,
    @implements Accessible, Buildable, ConstraintTarget;
}

impl TimelineRow {
    pub fn new(timeline_length: u64) -> Self {
        Object::new(&[("timeline-length", &timeline_length)]).unwrap()
    }

    pub fn timeline_length(&self) -> u64 {
        self.property("timeline-length")
    }
}

// ------------------------------------------------------------------------------

#[derive(Default)]
pub struct TimelineRowPrivate {
    timeline_length: Cell<u64>,
}

#[object_subclass]
impl ObjectSubclass for TimelineRowPrivate {
    const NAME: &'static str = "WtfTimelineRow";
    type Type = TimelineRow;
    type ParentType = Widget;
}

impl ObjectImpl for TimelineRowPrivate {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        obj.set_hexpand(true);
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<[ParamSpec; 1]> = Lazy::new(|| {
            [ParamSpecUInt64::new(
                "timeline-length",
                "TimelineLength",
                "Length of the timeline in milliseconds",
                0,
                u64::MAX,
                u64::default(),
                ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
            )]
        });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "timeline-length" => self.timeline_length.get().to_value(),
            _ => unreachable!(),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "timeline-length" => self.timeline_length.set(value.get().unwrap()),
            _ => unreachable!(),
        }
    }
}

impl WidgetImpl for TimelineRowPrivate {
    fn measure(
        &self,
        _widget: &Self::Type,
        orientation: Orientation,
        _for_size: i32,
    ) -> (i32, i32, i32, i32) {
        match orientation {
            Orientation::Horizontal => (20, 20, -1, -1),
            Orientation::Vertical => (30, 30, -1, -1),
            _ => unreachable!(),
        }
    }

    fn snapshot(&self, widget: &Self::Type, snapshot: &Snapshot) {
        self.parent_snapshot(widget, snapshot);

        let data = [
            13, 9, 8, 11, 14, 14, 12, 9, 8, 1, 6, 2, 5, 7, 11, 4, 5, 11, 9, 3, 11, 4, 7, 1, 7, 7,
            1, 8, 12, 3, 8, 1, 5, 5, 4, 5, 3, 1, 3, 1, 6, 4, 6, 1, 5, 4, 11, 1, 15, 6, 5, 8, 3, 6,
            1, 6, 7, 1, 8, 5, 1, 7, 4, 11, 1, 6, 9, 14, 6, 3, 8, 2, 1, 11, 6, 2, 11, 8, 1, 4, 1, 1,
            8, 11, 7, 4, 2, 1, 8, 3, 7, 11, 1, 3, 1, 1, 1, 1, 3, 11, 1, 5, 8, 1, 8, 11, 1, 1, 7, 1,
            14, 9, 7, 5, 1, 1, 11, 2, 4, 6, 8, 2, 11, 1, 1, 5, 3, 11, 1, 7, 9, 4, 8, 11, 1, 1, 8,
            2, 8, 13, 5, 2, 3, 1, 1, 1, 11, 11, 6, 6, 5, 11, 16, 1, 12, 12, 6, 7, 3, 2, 8, 12, 4,
            3, 1, 2, 11, 1, 3, 5, 5, 6, 6, 1, 2, 1, 3, 6, 6, 6, 7, 1, 13, 4, 1, 1, 8, 4, 7, 6, 5,
            2, 12, 6, 1, 7, 1, 2, 11, 1, 6, 11, 4, 1, 9, 3, 8, 7, 11, 11, 5, 1, 3, 7, 8, 6, 12, 1,
            5, 4, 5, 3, 6, 7, 11, 4, 2, 5, 21, 9, 4, 13, 3, 1, 1, 5, 1, 12, 1, 8, 5, 6, 7, 5, 8, 1,
            7, 1, 4, 11, 9, 14, 9, 8, 8, 1, 1, 6, 2, 4, 3, 1, 2, 7, 12, 1, 6, 4, 1, 1, 11, 1, 6,
            11, 3, 15, 1, 15, 11, 2, 6, 7, 6, 11, 1, 1, 2, 1, 2, 4, 9, 6, 7, 1, 1, 5, 14, 1, 8, 6,
            1, 3, 1, 8, 6, 2, 1, 3, 1, 9, 3, 3, 7, 11, 1, 4, 11, 3, 1, 2, 3, 5, 7, 1, 11, 11, 3, 3,
            1, 4, 11, 7, 8, 1, 9, 1, 11, 1, 7, 6, 12, 4, 1, 14, 13, 1, 1, 17, 1, 8, 3, 1, 8, 7, 7,
            3, 6, 7, 3, 13, 2, 4, 5, 1, 9, 3, 11, 7, 1, 5, 8, 8, 7, 9, 6, 6, 11, 1, 1, 1, 5, 9, 11,
            1, 9, 1, 7, 1, 7, 4, 1, 5, 1, 7, 14, 1, 1, 11, 7, 11, 1, 11, 14, 2, 2, 6, 11, 1, 6, 17,
            8, 5, 3, 1, 9, 3, 1, 1, 4, 1, 1, 12, 1, 4, 6, 7, 7, 7, 8, 1, 1, 1, 1, 7, 2, 6, 1, 9, 5,
            3, 4, 5, 1, 4, 5, 2, 23, 1, 1, 4, 15, 1, 8, 1, 11, 7, 7, 6, 11, 11, 1, 1, 5, 3, 6, 1,
            7, 2, 6, 2, 13, 6, 3, 5, 8, 7, 5, 12, 11, 1, 9, 3, 9, 1, 6, 9, 2, 1, 9, 3, 2, 11, 1, 1,
            1, 5, 1, 2, 1, 3, 11, 6, 14, 1, 3, 4, 8, 1, 8, 5, 1, 4, 11, 6, 3, 8, 1, 3, 2, 3, 6, 6,
            6, 8, 7, 5, 11, 4, 8, 5, 8, 4, 11, 5, 1, 4, 8, 1, 2, 3, 2, 6, 11, 4, 3, 7, 11, 3, 6,
            17, 1, 5, 4, 5, 11, 5, 4, 8, 4, 12, 11, 12, 7, 6, 2, 3, 1, 9, 1, 6, 7, 1, 1, 4, 3, 12,
            2, 1, 3, 1, 1, 4, 11, 1, 1, 2, 1, 1, 5, 9, 11, 11, 1, 9, 3, 3, 9, 1, 1, 1, 2, 3, 5, 1,
            7, 12, 7, 9, 7, 7, 7, 11, 2, 11, 5, 1, 3, 9, 7, 2, 8, 4, 9, 2, 1, 7, 7, 8, 15, 6, 8, 6,
            8, 1, 13, 3, 4, 9, 5, 11, 4, 11, 9, 2, 1, 3, 8, 1, 8, 1, 1, 19, 14, 4, 6, 5, 4, 13, 7,
            6, 9, 9, 13, 11, 1, 1, 1, 9, 13, 1, 11, 1, 5, 2, 7, 2, 4, 1, 11, 1, 2, 2, 13, 11, 16,
            1, 17, 1, 3, 9, 7, 6, 3, 12, 4, 17, 5, 7, 2, 9, 9, 4, 7, 11, 8, 5, 8, 12, 6, 13, 2, 5,
            4, 18, 5, 6, 1, 1, 6, 2, 1, 4, 4, 1, 11, 7, 13, 1, 8, 3, 2, 4, 13, 1, 7, 6, 6, 3, 4, 1,
            9, 1, 9, 3, 5, 4, 4, 8, 8, 4, 1, 11, 4, 4, 4, 2, 1, 18, 12, 6, 2, 1, 7, 4, 14, 1, 5, 1,
            1, 12, 9, 1, 6, 8, 9, 6, 6, 5, 12, 9, 6, 11, 3, 1, 1, 1, 11, 1, 4, 2, 7, 2, 3, 7, 1, 9,
            4, 8, 6, 9, 4, 2, 8, 12, 1, 6, 1, 12, 1, 11, 8, 7, 1, 9, 1, 11, 4, 1, 1, 5, 11, 11, 4,
            7, 4, 1, 11, 1, 11, 7, 3, 6, 8, 8, 6, 6, 1, 5, 11, 5, 5, 4, 6, 12, 1, 4, 1, 1, 4, 4,
            11, 3, 3, 9, 11, 3, 1, 4, 3, 5, 9, 6, 1, 8, 6, 4, 1, 1, 9, 3, 11, 1, 1, 4, 1, 13, 1, 1,
            13, 7, 1, 1, 11, 2, 7, 11, 11, 11, 7, 1, 1, 1, 9, 1, 5, 1, 5, 1, 1, 8, 8, 11, 5, 12,
            12, 2, 4, 3, 7, 7, 4, 9, 3, 1, 1, 7, 5, 1, 2, 1, 5, 8, 3, 8, 3, 1, 7, 11, 1, 1, 1, 1,
            5, 13, 3, 6, 8, 9, 15, 2, 11, 2, 1, 3, 5, 3, 11, 7, 1, 7, 13, 4, 1, 9, 1, 2, 11, 13, 9,
            8, 11, 7, 8, 2, 1, 6, 1, 4, 1, 1, 7, 2, 6, 4, 1, 1, 5, 1, 11, 1, 1, 8, 7, 4, 13, 1, 2,
            9, 1, 1, 7, 9, 3, 1, 1, 3, 1, 4, 11, 1,
        ];

        let color = widget.style_context().lookup_color("blue_3").unwrap();
        let widget_height = widget.height() as f32;
        const CROP_CALLSTACKS_TALLER_THAN: f32 = 20.0;

        let count = (widget.width() / 2 + 1).min(data.len() as i32);

        for i in 0..count {
            let datum = data[i as usize];
            if datum > 0 {
                let height = ((datum as f32 / CROP_CALLSTACKS_TALLER_THAN) * widget_height)
                    .min(widget_height);
                snapshot.append_color(
                    &color,
                    &Rect::new((i * 2) as f32, widget_height - height, 2.0, height),
                );
            }
        }
    }
}
