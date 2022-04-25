use crate::timeline_range::{TimelineRange, TimelineRangePrivatePropertiesExt};
use crate::timeline_row::TimelineRow;
use crate::timeline_ticker::TimelineTicker;
use glib::subclass::prelude::{
    DerivedObjectProperties, ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassIsExt,
};
use glib::{object_subclass, Cast, Object, ObjectExt, ParamSpec, Properties, StaticType, Value};
use gtk::gdk::RGBA;
use gtk::graphene::Rect;
use gtk::subclass::prelude::{WidgetClassSubclassExt, WidgetImpl, WidgetImplExt};
use gtk::traits::{EventControllerExt, GestureDragExt, OrientableExt, WidgetExt};
use gtk::{
    Accessible, BoxLayout, Buildable, ConstraintTarget, GestureDrag, Orientation, Scrollable,
    Snapshot, Widget,
};
use std::cell::RefCell;

glib::wrapper! {
    pub struct TimelineView(ObjectSubclass<TimelineViewPrivate>)
    @extends Widget,
    @implements Scrollable, Accessible, Buildable, ConstraintTarget;
}

impl TimelineView {
    pub fn new(profile_time_range: &TimelineRange) -> Self {
        Object::new(&[
            ("profile-time-range", profile_time_range),
            ("display-time-range", profile_time_range),
        ])
        .unwrap()
    }

    pub fn time_to_widget_point(&self, time_point: u64) -> f64 {
        let timeline_width = self.width() as f64;
        let display_time_range = self.imp().display_time_range.borrow();
        let display_start = display_time_range.start() as f64;
        let display_end = display_time_range.end() as f64;
        (((time_point as f64 - display_start) * timeline_width) / (display_end - display_start))
            .clamp(0.0, timeline_width)
    }

    pub fn widget_to_time_point(&self, widget_point: f64) -> u64 {
        let timeline_width = self.width() as f64;
        let display_time_range = self.imp().display_time_range.borrow();
        let display_start = display_time_range.start() as f64;
        let display_end = display_time_range.end() as f64;
        ((((widget_point * (display_end - display_start)) / timeline_width) + display_start).round()
            as u64)
            .clamp(display_time_range.start(), display_time_range.end())
    }
}

// ------------------------------------------------------------------------------

#[derive(Properties, Default)]
pub struct TimelineViewPrivate {
    #[property(get, set, construct_only, builder(TimelineRange::static_type()))]
    profile_time_range: RefCell<TimelineRange>,
    #[property(get, set = Self::set_display_time_range, builder(TimelineRange::static_type()))]
    display_time_range: RefCell<TimelineRange>,
    #[property(get, set, builder(TimelineRange::static_type()))]
    selected_time_range: RefCell<Option<TimelineRange>>,
}

impl TimelineViewPrivate {
    pub fn set_display_time_range(&self, value: TimelineRange) {
        *self.display_time_range.borrow_mut() = value;
        *self.selected_time_range.borrow_mut() = None;
    }
}

fn clear_selection(gesture: &GestureDrag, _: f64, _: f64) {
    let timeline = gesture.widget().downcast::<TimelineView>().unwrap();
    timeline.set_selected_time_range(None);

    timeline.queue_draw();
}

fn update_selection(gesture: &GestureDrag, _: f64, _: f64) {
    let drag_offset = gesture.offset().unwrap().0;
    let drag_point = gesture.start_point().unwrap().0;
    let (drag_start, drag_end) = if drag_offset < 0.0 {
        (drag_point + drag_offset, drag_point)
    } else {
        (drag_point, drag_point + drag_offset)
    };

    let timeline = gesture.widget().downcast::<TimelineView>().unwrap();

    timeline.set_selected_time_range(Some(TimelineRange::new(
        timeline.widget_to_time_point(drag_start),
        timeline.widget_to_time_point(drag_end),
    )));

    timeline.queue_draw();
}

#[object_subclass]
impl ObjectSubclass for TimelineViewPrivate {
    const NAME: &'static str = "WtfTimelineView";
    type Type = TimelineView;
    type ParentType = Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.set_layout_manager_type::<BoxLayout>();
    }
}

impl ObjectImpl for TimelineViewPrivate {
    fn constructed(&self, this: &Self::Type) {
        self.parent_constructed(this);

        let layout_manager = this
            .layout_manager()
            .unwrap()
            .downcast::<gtk::BoxLayout>()
            .unwrap();
        layout_manager.set_orientation(Orientation::Vertical);
        layout_manager.set_spacing(6);

        this.set_hexpand(true);
        this.set_vexpand(true);

        let selection_controller = GestureDrag::new();
        selection_controller.connect_drag_begin(clear_selection);
        selection_controller.connect_drag_update(update_selection);
        this.add_controller(&selection_controller);

        let timeline_ticker = TimelineTicker::new();
        timeline_ticker.set_parent(this);
        this.bind_property("display-time-range", &timeline_ticker, "time-range")
            .build();

        for _ in 0..10 {
            let timeline_row = TimelineRow::new();
            timeline_row.set_parent(this);
            this.bind_property("display-time-range", &timeline_row, "time-range")
                .build();
        }
    }

    fn dispose(&self, this: &Self::Type) {
        while let Some(child) = this.first_child() {
            child.unparent();
        }
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

impl WidgetImpl for TimelineViewPrivate {
    fn snapshot(&self, this: &Self::Type, snapshot: &Snapshot) {
        self.parent_snapshot(this, snapshot);

        if let Some(selected_time_range) = this.selected_time_range() {
            let selection_start = this.time_to_widget_point(selected_time_range.start()) as f32;
            let selection_end = this.time_to_widget_point(selected_time_range.end()) as f32;

            let selection_color = RGBA::new(0.0, 0.0, 0.0, 0.1);
            snapshot.append_color(
                &selection_color,
                &Rect::new(0.0, 0.0, selection_start, this.height() as f32),
            );
            snapshot.append_color(
                &selection_color,
                &Rect::new(
                    selection_end,
                    0.0,
                    (this.width() as f32) - selection_end,
                    this.height() as f32,
                ),
            );
        }
    }
}
