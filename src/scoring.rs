use crate::constants::*;
use dom_query::Document;

/// Returns a CSS selector string for the main content element.
pub fn find_main_content_selector(doc: &Document) -> String {
    for sel_str in ENTRY_POINT_SELECTORS {
        let sel = doc.select(sel_str);
        if sel.exists() {
            let words = sel.text().split_whitespace().count();
            if words > 20 || *sel_str == "body" {
                return sel_str.to_string();
            }
        }
    }
    "body".to_string()
}
