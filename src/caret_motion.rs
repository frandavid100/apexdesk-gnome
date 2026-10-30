//! Visual-only caret interpolation. Engine selection and hit testing stay exact.
use crate::selection::Rect;

const DURATION_US: i64 = 100_000;

#[derive(Default)]
pub struct CaretMotion {
    displayed: Option<Rect>,
    target: Option<Rect>,
    from: Option<Rect>,
    start: i64,
}

impl CaretMotion {
    pub fn position(&self) -> Option<&Rect> { self.displayed.as_ref() }
    pub fn is_moving(&self) -> bool { self.from.is_some() }
    pub fn finish(&mut self) { self.displayed = self.target.clone(); self.from = None; }

    pub fn retarget(&mut self, target: Option<Rect>, now: i64, animate: bool) {
        self.advance(now);
        if self.target == target {
            if !animate { self.finish(); }
            return;
        }
        // New documents, hidden/selected carets and page transitions snap.
        // Never draw a caret flying through page margins or unrelated pages.
        let same_page = matches!((&self.displayed, &target), (Some(a), Some(b)) if a.page == b.page);
        self.from = if animate && same_page { self.displayed.clone() } else { None };
        self.target = target;
        self.start = now;
        if !self.is_moving() { self.finish(); }
    }

    pub fn advance(&mut self, now: i64) {
        let (Some(from), Some(to)) = (&self.from, &self.target) else { return };
        let t = ((now - self.start) as f64 / DURATION_US as f64).clamp(0.0, 1.0);
        if t >= 1.0 { self.finish(); return; }
        let t = 1.0 - (1.0 - t).powi(3);
        let lerp = |a, b| a + (b - a) * t;
        self.displayed = Some(Rect { page: to.page,
            x: lerp(from.x, to.x), y: lerp(from.y, to.y),
            width: lerp(from.width, to.width), height: lerp(from.height, to.height) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rect(page: usize, x: f64) -> Option<Rect> {
        Some(Rect { page, x, y: 20.0, width: 1.0, height: 16.0 })
    }
    #[test]
    fn rapid_typing_retargets_from_current_visual_position_and_finishes() {
        let mut m = CaretMotion::default();
        m.retarget(rect(0, 10.0), 0, true);
        assert!(!m.is_moving());
        m.retarget(rect(0, 30.0), 10, true);
        m.advance(50_010);
        let x = m.position().unwrap().x;
        assert!(x > 10.0 && x < 30.0);
        m.retarget(rect(0, 40.0), 50_010, true);
        assert_eq!(m.position().unwrap().x, x);
        // Repeated layout refreshes must not restart the animation.
        m.retarget(rect(0, 40.0), 70_010, true);
        m.advance(150_010);
        assert_eq!(m.position(), rect(0, 40.0).as_ref());
        assert!(!m.is_moving());
    }
    #[test]
    fn reduced_motion_hidden_caret_and_page_changes_snap() {
        let mut m = CaretMotion::default();
        m.retarget(rect(0, 10.0), 0, true);
        m.retarget(rect(0, 30.0), 1, true);
        m.retarget(rect(0, 30.0), 2, false);
        assert_eq!(m.position(), rect(0, 30.0).as_ref());
        assert!(!m.is_moving());
        m.retarget(rect(1, 40.0), 3, true);
        assert!(!m.is_moving());
        assert_eq!(m.position(), rect(1, 40.0).as_ref());
        m.retarget(None, 4, true);
        assert!(m.position().is_none());
        assert!(!m.is_moving());
    }
}
