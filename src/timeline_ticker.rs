use crate::timeline_range::{TimelineRange, TimelineRangePrivatePropertiesExt};
use crate::timeline_view::{TimelineView, TimelineViewPrivatePropertiesExt};
use glib::subclass::prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass};
use glib::{object_subclass, Cast, Object, ObjectExt, Properties, StaticType};
use gtk::gdk::RGBA;
use gtk::graphene::Rect;
use gtk::subclass::prelude::{WidgetClassSubclassExt, WidgetImpl, WidgetImplExt};
use gtk::traits::WidgetExt;
use gtk::{
    Accessible, Buildable, ConstraintTarget, Orientation, Overflow, Scrollable, Snapshot, Widget,
};
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
}

// ------------------------------------------------------------------------------

#[derive(Properties, Default)]
pub struct TimelineTickerPrivate {
    #[property(get, set, builder(TimelineRange::static_type()))]
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
    fn constructed(&self, this: &Self::Type) {
        self.parent_constructed(this);

        this.set_hexpand(true);
        this.set_overflow(Overflow::Hidden);
        this.set_css_classes(&["caption-heading", "monospace"]);
    }
}

impl WidgetImpl for TimelineTickerPrivate {
    fn measure(&self, _: &Self::Type, orientation: Orientation, _: i32) -> (i32, i32, i32, i32) {
        match orientation {
            Orientation::Horizontal => (0, 0, -1, -1),
            Orientation::Vertical => (40, 40, -1, -1),
            _ => unreachable!(),
        }
    }

    fn snapshot(&self, this: &Self::Type, snapshot: &Snapshot) {
        self.parent_snapshot(this, snapshot);

        let timeline_view = this.parent().unwrap().downcast::<TimelineView>().unwrap();
        let time_range = this.time_range();

        let mut p = 5;
        while p < (this.width() - 5) {
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
                    1000..=59999 => format!("{:.0}s", (timestamp as f64 / 1000.0).round()),
                    60000..=3599999 => format!("{:.0}m", (timestamp as f64 / 60000.0).round()),
                    _ => format!("{:.0}h", (timestamp as f64 / 3600000.0).round()),
                };
                let timestamp = this.create_pango_layout(Some(&timestamp));
                snapshot.render_layout(
                    &this.style_context(),
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
