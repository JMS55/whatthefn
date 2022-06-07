use crate::perf_data_parser::convert_perf_json_to_wtf;
use crate::timeline_view::TimelineView;

pub fn new_profile_page() -> TimelineView {
    TimelineView::new(convert_perf_json_to_wtf("test.perf.json").unwrap())
}
