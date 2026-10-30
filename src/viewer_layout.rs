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
    // Ordered non-overlapping pages: inspect only the neighbours of the pointer.
    let next = tops.partition_point(|&top| top <= position);
    if next == 0 { return 0; }
    let previous = next - 1;
    if next == tops.len() { return previous; }
    let z = f64::from(zoom_clamped(zoom));
    let before = distance_to_page(position, tops[previous], f64::from(heights[previous].1)*z);
    let after = distance_to_page(position, tops[next], f64::from(heights[next].1)*z);
    if before <= after { previous } else { next }
}

fn distance_to_page(position: f64, top: f64, height: f64) -> f64 {
    if position < top { top - position }
    else if position > top + height { position - (top + height) }
    else { 0.0 }
}

pub fn visible_indices(tops: &[f64], heights: &[(f32, f32)], zoom: f32, first: f64, last: f64) -> Vec<usize> {
    let margin = 260.0; // Predibujar las páginas inmediatamente próximas.
    let z = f64::from(zoom_clamped(zoom));
    // Both page starts and ends are ordered; avoid scanning the whole document
    // on every scroll event. Boundary comparisons match the original viewport.
    let start = tops.partition_point(|&y| y < first - margin);
    let start = start.saturating_sub(1);
    let end = tops.partition_point(|&y| y <= last + margin);
    (start..end).filter(|&i| tops[i] + f64::from(heights[i].1)*z >= first-margin).collect()
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

#[cfg(test)]
mod lookup_regression {
    use super::*;
    #[test]
    fn indexed_lookup_matches_full_scan_on_mixed_pages_and_gaps() {
        let sizes: Vec<_> = (0..1000).map(|i| (612.0, 300.0+(i%7) as f32*113.0)).collect();
        for zoom in [0.5, 1.0, 2.0] {
            let tops=page_tops(&sizes,zoom);
            let z=f64::from(zoom);
            for i in (0..1000).step_by(13) {
                for position in [tops[i]-500.0, tops[i]-11.0, tops[i], tops[i]+300.0, tops[i]+900.0] {
                    let expected=(0..tops.len()).min_by(|&a,&b| distance_to_page(position,tops[a],f64::from(sizes[a].1)*z).total_cmp(&distance_to_page(position,tops[b],f64::from(sizes[b].1)*z))).unwrap();
                    assert_eq!(page_at(&tops,&sizes,zoom,position),expected);
                    let last=position+800.0;
                    let expected: Vec<_>=(0..tops.len()).filter(|&j| tops[j]<=last+260.0 && tops[j]+f64::from(sizes[j].1)*z>=position-260.0).collect();
                    assert_eq!(visible_indices(&tops,&sizes,zoom,position,last),expected);
                }
            }
        }
    }
}
