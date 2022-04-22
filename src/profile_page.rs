use crate::timeline_range::TimelineRange;
use crate::timeline_view::TimelineView;

pub fn new_profile_page() -> TimelineView {
    TimelineView::new(&TimelineRange::new(1649442376873, 1649442419233))
}
