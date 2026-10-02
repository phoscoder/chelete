//! Page arithmetic shared by the list screens.

pub const PER_PAGE_OPTIONS: [usize; 8] = [10, 20, 25, 30, 40, 60, 80, 100];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pager {
    pub page: usize,
    pub per_page: usize,
    pub total: usize,
}

impl Pager {
    pub fn new(page: usize, per_page: usize, total: usize) -> Self {
        let per_page = per_page.max(1);
        let pages = total.div_ceil(per_page).max(1);
        Self { page: page.clamp(1, pages), per_page, total }
    }

    pub fn total_pages(&self) -> usize {
        self.total.div_ceil(self.per_page).max(1)
    }

    pub fn has_prev(&self) -> bool {
        self.page > 1
    }

    pub fn has_next(&self) -> bool {
        self.page < self.total_pages()
    }

    /// Index range of the current page within the full list.
    pub fn range(&self) -> std::ops::Range<usize> {
        let start = ((self.page - 1) * self.per_page).min(self.total);
        start..(start + self.per_page).min(self.total)
    }

    /// e.g. `1–10 of 25`.
    pub fn label(&self) -> String {
        if self.total == 0 {
            return "0–0 of 0".into();
        }
        let r = self.range();
        format!("{}–{} of {}", r.start + 1, r.end, self.total)
    }
}

pub fn page_slice<'a, T>(items: &'a [T], pager: &Pager) -> &'a [T] {
    &items[pager.range()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_and_ranges() {
        let p = Pager::new(1, 10, 25);
        assert_eq!(p.label(), "1–10 of 25");
        assert_eq!(p.range(), 0..10);
        assert!(!p.has_prev() && p.has_next());
        let last = Pager::new(3, 10, 25);
        assert_eq!(last.label(), "21–25 of 25");
        assert_eq!(last.range(), 20..25);
        assert!(last.has_prev() && !last.has_next());
    }

    #[test]
    fn page_is_clamped_when_the_list_shrinks() {
        let p = Pager::new(9, 10, 12);
        assert_eq!(p.page, 2);
        assert_eq!(Pager::new(0, 10, 12).page, 1);
    }

    #[test]
    fn empty_list() {
        let p = Pager::new(1, 10, 0);
        assert_eq!(p.label(), "0–0 of 0");
        assert_eq!(p.total_pages(), 1);
        assert!(!p.has_next() && !p.has_prev());
        assert!(page_slice::<u8>(&[], &p).is_empty());
    }

    #[test]
    fn slices_the_current_page() {
        let items: Vec<u32> = (0..25).collect();
        assert_eq!(page_slice(&items, &Pager::new(3, 10, 25)), &[20, 21, 22, 23, 24]);
    }

    #[test]
    fn zero_per_page_does_not_panic() {
        assert_eq!(Pager::new(1, 0, 5).per_page, 1);
    }
}
