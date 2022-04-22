use crate::timeline_range::TimelineRange;
use crate::timeline_ticker::TimelineTicker;
use glib::subclass::prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassIsExt};
use glib::{
    object_subclass, Cast, Object, ObjectExt, ParamFlags, ParamSpec, ParamSpecObject, StaticType,
    ToValue, Value,
};
use gtk::gdk::RGBA;
use gtk::graphene::Rect;
use gtk::subclass::prelude::{WidgetClassSubclassExt, WidgetImpl, WidgetImplExt};
use gtk::traits::{EventControllerExt, GestureDragExt, OrientableExt, WidgetExt};
use gtk::{
    Accessible, BoxLayout, Buildable, ConstraintTarget, GestureDrag, Orientation, Scrollable,
    Snapshot, Widget,
};
use once_cell::sync::Lazy;
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

    pub fn profile_time_range(&self) -> TimelineRange {
        self.property("profile-time-range")
    }

    pub fn display_time_range(&self) -> TimelineRange {
        self.property("display-time-range")
    }

    pub fn set_display_time_range(&self, display_time_range: &TimelineRange) {
        self.set_property("display-time-range", display_time_range);
    }

    pub fn selected_time_range(&self) -> Option<TimelineRange> {
        self.property("selected-time-range")
    }

    pub fn set_selected_time_range(&self, selected_time_range: Option<&TimelineRange>) {
        self.set_property("selected-time-range", selected_time_range);
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

#[derive(Default)]
pub struct TimelineViewPrivate {
    profile_time_range: RefCell<TimelineRange>,
    display_time_range: RefCell<TimelineRange>,
    selected_time_range: RefCell<Option<TimelineRange>>,
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

    timeline.set_selected_time_range(Some(&TimelineRange::new(
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
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        let layout_manager = obj
            .layout_manager()
            .unwrap()
            .downcast::<gtk::BoxLayout>()
            .unwrap();
        layout_manager.set_orientation(Orientation::Vertical);

        obj.set_hexpand(true);
        obj.set_vexpand(true);

        let timeline_ticker = TimelineTicker::new();
        timeline_ticker.set_parent(obj);
        obj.bind_property("display-time-range", &timeline_ticker, "time-range")
            .build();

        // TODO: Add TimelineRow children, bind display-time-range

        let selection_controller = GestureDrag::new();
        selection_controller.connect_drag_begin(clear_selection);
        selection_controller.connect_drag_update(update_selection);
        obj.add_controller(&selection_controller);
    }

    fn dispose(&self, obj: &Self::Type) {
        while let Some(child) = obj.first_child() {
            child.unparent();
        }
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<[ParamSpec; 3]> = Lazy::new(|| {
            [
                ParamSpecObject::new(
                    "profile-time-range",
                    "ProfileTimeRange",
                    "Time range for the entire profile",
                    TimelineRange::static_type(),
                    ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                ),
                ParamSpecObject::new(
                    "display-time-range",
                    "DisplayTimeRange",
                    "Currently displayed time range",
                    TimelineRange::static_type(),
                    ParamFlags::READWRITE,
                ),
                ParamSpecObject::new(
                    "selected-time-range",
                    "DisplayTimeRange",
                    "Currently selected time range",
                    TimelineRange::static_type(),
                    ParamFlags::READWRITE,
                ),
            ]
        });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "profile-time-range" => self.profile_time_range.borrow().to_value(),
            "display-time-range" => self.display_time_range.borrow().to_value(),
            "selected-time-range" => self.selected_time_range.borrow().to_value(),
            _ => unreachable!(),
        }
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "profile-time-range" => *self.profile_time_range.borrow_mut() = value.get().unwrap(),
            "display-time-range" => {
                *self.display_time_range.borrow_mut() = value.get().unwrap();
                *self.selected_time_range.borrow_mut() = None;
            }
            "selected-time-range" => *self.selected_time_range.borrow_mut() = value.get().unwrap(),
            _ => unreachable!(),
        }
    }
}

impl WidgetImpl for TimelineViewPrivate {
    fn snapshot(&self, widget: &Self::Type, snapshot: &Snapshot) {
        self.parent_snapshot(widget, snapshot);

        if let Some(selected_time_range) = widget.selected_time_range() {
            let selection_start = widget.time_to_widget_point(selected_time_range.start()) as f32;
            let selection_end = widget.time_to_widget_point(selected_time_range.end()) as f32;

            let selection_color = RGBA::new(0.0, 0.0, 0.0, 0.1);
            snapshot.append_color(
                &selection_color,
                &Rect::new(0.0, 0.0, selection_start, widget.height() as f32),
            );
            snapshot.append_color(
                &selection_color,
                &Rect::new(
                    selection_end,
                    0.0,
                    (widget.width() as f32) - selection_end,
                    widget.height() as f32,
                ),
            );
        }
    }
}
