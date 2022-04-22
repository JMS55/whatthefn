use crate::timeline_range::TimelineRange;
use crate::timeline_view::TimelineView;
use glib::subclass::prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass};
use glib::{
    object_subclass, Cast, Object, ObjectExt, ParamFlags, ParamSpec, ParamSpecObject, StaticType,
    ToValue, Value,
};
use gtk::gdk::RGBA;
use gtk::graphene::Rect;
use gtk::subclass::prelude::{WidgetClassSubclassExt, WidgetImpl, WidgetImplExt};
use gtk::traits::WidgetExt;
use gtk::{
    Accessible, Buildable, ConstraintTarget, Orientation, Overflow, Scrollable, Snapshot, Widget,
};
use once_cell::sync::Lazy;
use std::cell::RefCell;

glib::wrapper! {
    pub struct TimelineTicker(ObjectSubclass<TimelineTickerPrivate>)
    @extends Widget,
    @implements Scrollable, Accessible, Buildable, ConstraintTarget;
}

impl TimelineTicker {
    pub fn new() -> Self {
        Object::new(&[]).unwrap()
    }

    pub fn time_range(&self) -> TimelineRange {
        self.property("time-range")
    }

    pub fn set_time_range(&self, time_range: &TimelineRange) {
        self.set_property("time-range", time_range);
    }
}

// ------------------------------------------------------------------------------

#[derive(Default)]
pub struct TimelineTickerPrivate {
    time_range: RefCell<TimelineRange>,
}

#[object_subclass]
impl ObjectSubclass for TimelineTickerPrivate {
    const NAME: &'static str = "WtfTimelineTicker";
    type Type = TimelineTicker;
    type ParentType = Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.set_css_name("timeline-ticker");
    }
}

impl ObjectImpl for TimelineTickerPrivate {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        obj.set_hexpand(true);
        obj.set_overflow(Overflow::Hidden);
        obj.set_css_classes(&["caption-heading", "monospace"]);
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<[ParamSpec; 1]> = Lazy::new(|| {
            [ParamSpecObject::new(
                "time-range",
                "TimeRange",
                "TODO",
                TimelineRange::static_type(),
                ParamFlags::READWRITE,
            )]
        });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "time-range" => self.time_range.borrow().to_value(),
            _ => unreachable!(),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "time-range" => *self.time_range.borrow_mut() = value.get().unwrap(),
            _ => unreachable!(),
        }
    }
}

impl WidgetImpl for TimelineTickerPrivate {
    fn measure(
        &self,
        _widget: &Self::Type,
        orientation: Orientation,
        _for_size: i32,
    ) -> (i32, i32, i32, i32) {
        match orientation {
            Orientation::Horizontal => (0, 0, -1, -1),
            Orientation::Vertical => (40, 40, -1, -1),
            _ => unreachable!(),
        }
    }

    fn snapshot(&self, widget: &Self::Type, snapshot: &Snapshot) {
        self.parent_snapshot(widget, snapshot);

        let timeline_view = widget.parent().unwrap().downcast::<TimelineView>().unwrap();
        let time_range = widget.time_range();

        let mut p = 5;
        while p < (widget.width() - 5) {
            let is_longer_tick = p % 55 == 0;

            snapshot.append_color(
                &RGBA::BLACK,
                &Rect::new(
                    p as f32,
                    20.0,
                    1.0,
                    if is_longer_tick { 20.0 } else { 10.0 },
                ),
            );

            if is_longer_tick {
                let timestamp = timeline_view.widget_to_time_point(p as f64)
                    - timeline_view.profile_time_range().start();
                let timestamp = match time_range.end() - time_range.start() {
                    0..=999 => format!("{timestamp}ms"),
                    1000..=59999 => format!("{:.2}s", timestamp as f64 / 1000.0),
                    60000..=3599999 => format!("{:.2}m", timestamp as f64 / 60000.0),
                    _ => format!("{:.2}h", timestamp as f64 / 3600000.0),
                };
                let timestamp = widget.create_pango_layout(Some(&timestamp));
                snapshot.render_layout(
                    &widget.style_context(),
                    p as f64 - (timestamp.pixel_size().0 as f64 / 2.0)
                        + if timestamp.pixel_size().0 % 2 == 0 {
                            0.0
                        } else {
                            1.0
                        },
                    2.0,
                    &timestamp,
                );
            }

            p += 10;
        }
    }
}
