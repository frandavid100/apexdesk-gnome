//! Geometría del visor; no depende ni de GTK ni de egui.
pub const PAGE_GAP: f64 = 22.0;
pub const MIN_ZOOM: f32 = 0.5;
pub const MAX_ZOOM: f32 = 2.0;
pub const THUMB_SCALE: f32 = 0.28;

pub fn zoom_clamped(value: f32) -> f32 {
    if value.is_finite() { value.clamp(MIN_ZOOM, MAX_ZOOM) } else { 1.0 }
}

/// Coordenada Y superior de cada página, incluyendo un margen entre hojas.
pub fn page_tops(dimensions: &[(f32, f32)], zoom: f32) -> Vec<f64> {
    let z = zoom_clamped(zoom) as f64;
    let mut top = PAGE_GAP;
    dimensions.iter().map(|&(_, height)| {
        let current = top;
        top += f64::from(height) * z + PAGE_GAP;
        current
    }).collect()
}

pub fn page_at(tops: &[f64], heights: &[(f32, f32)], zoom: f32, position: f64) -> usize {
    if tops.is_empty() { return 0; }
    let z = f64::from(zoom_clamped(zoom));
    tops.iter().enumerate()
        .min_by(|(a, ya), (b, yb)| {
            let da = distance_to_page(position, **ya, f64::from(heights[*a].1) * z);
            let db = distance_to_page(position, **yb, f64::from(heights[*b].1) * z);
            da.total_cmp(&db)
        })
        .map(|(i, _)| i).unwrap_or(0)
}

fn distance_to_page(position: f64, top: f64, height: f64) -> f64 {
    if position < top { top - position }
    else if position > top + height { position - (top + height) }
    else { 0.0 }
}

pub fn visible_indices(tops: &[f64], heights: &[(f32, f32)], zoom: f32, first: f64, last: f64) -> Vec<usize> {
    let margin = 260.0; // Predibujar las páginas inmediatamente próximas.
    let z = f64::from(zoom_clamped(zoom));
    tops.iter().enumerate()
        .filter(|(i, y)| **y <= last + margin && **y + f64::from(heights[*i].1) * z >= first - margin)
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pages_are_stacked_with_a_gap() {
        let sizes = [(612.0, 792.0), (612.0, 792.0), (612.0, 792.0)];
        let offsets = page_tops(&sizes, 1.0);
        assert_eq!(offsets, vec![22.0, 836.0, 1650.0]);
        assert_eq!(page_at(&offsets, &sizes, 1.0, 912.0), 1);
        assert_eq!(visible_indices(&offsets, &sizes, 1.0, 1200.0, 1500.0), vec![1,2]);
    }
    #[test]
    fn zoom_is_bounded_and_monotonic() {
        assert_eq!(zoom_clamped(f32::NAN), 1.0);
        assert_eq!(zoom_clamped(12.0), 2.0);
        assert_eq!(zoom_clamped(0.1), 0.5);
        let sizes = [(612.0,792.0);2];
        assert!(page_tops(&sizes, 1.5)[1] > page_tops(&sizes, 1.0)[1]);
    }
    #[test]
    fn empty_document_does_not_crash() {
        assert!(page_tops(&[], 1.0).is_empty());
        assert_eq!(page_at(&[], &[], 1.0, 10.0), 0);
        assert!(visible_indices(&[], &[], 1.0, 0.0, 100.0).is_empty());
    }
}
