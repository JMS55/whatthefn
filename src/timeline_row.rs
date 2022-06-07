use crate::perf_data_parser::Sample;
use crate::timeline_view::{TimelineView, TimelineViewPrivatePropertiesExt};
use glib::once_cell::sync::OnceCell;
use glib::subclass::prelude::{DerivedObjectProperties, ObjectImpl, ObjectImplExt, ObjectSubclass};
use glib::{
    object_subclass, BoxedAnyObject, Cast, Object, ObjectExt, ParamSpec, Properties, StaticType,
    Value,
};
use gtk::graphene::Rect;
use gtk::subclass::prelude::{WidgetClassSubclassExt, WidgetImpl, WidgetImplExt};
use gtk::traits::{StyleContextExt, WidgetExt};
use gtk::{Accessible, Buildable, ConstraintTarget, Orientation, Snapshot, Widget};
use std::cell::Ref;

glib::wrapper! {
    pub struct TimelineRow(ObjectSubclass<TimelineRowPrivate>)
    @extends Widget,
    @implements Accessible, Buildable, ConstraintTarget;
}

impl TimelineRow {
    pub fn new(samples: Vec<Sample>) -> Self {
        Object::new(&[("samples", &BoxedAnyObject::new(samples))]).unwrap()
    }
}

// ------------------------------------------------------------------------------

#[derive(Properties, Default)]
pub struct TimelineRowPrivate {
    #[property(get, set, construct_only, builder(BoxedAnyObject::static_type()))]
    samples: OnceCell<BoxedAnyObject>,
}

#[object_subclass]
impl ObjectSubclass for TimelineRowPrivate {
    const NAME: &'static str = "WtfTimelineRow";
    type Type = TimelineRow;
    type ParentType = Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.set_css_name("timeline-row");
    }
}

impl ObjectImpl for TimelineRowPrivate {
    fn constructed(&self, this: &Self::Type) {
        self.parent_constructed(this);

        this.set_hexpand(true);
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

impl WidgetImpl for TimelineRowPrivate {
    fn measure(&self, _: &Self::Type, orientation: Orientation, _: i32) -> (i32, i32, i32, i32) {
        match orientation {
            Orientation::Horizontal => (0, 0, -1, -1),
            Orientation::Vertical => (30, 30, -1, -1),
            _ => unreachable!(),
        }
    }

    fn snapshot(&self, this: &Self::Type, snapshot: &Snapshot) {
        self.parent_snapshot(this, snapshot);

        let timeline_view = this.parent().unwrap().downcast::<TimelineView>().unwrap();

        let samples: Ref<Vec<Sample>> = self.samples.get().unwrap().borrow();
        let samples = samples.iter().filter(|sample| {
            timeline_view
                .display_time_range()
                .contains(sample.timestamp)
        });

        let mut color = this.style_context().lookup_color("blue_3").unwrap();
        color.set_alpha(0.7);

        for sample in samples {
            let x = timeline_view.time_to_widget_point(sample.timestamp);
            let height = (sample.callchain.len() as f32).min(this.height() as f32);
            snapshot.append_color(
                &color,
                &Rect::new(x as f32 - 0.5, this.height() as f32 - height, 2.0, height),
            );
        }
    }
}
