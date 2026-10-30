//! Identical workload and pixel checksum for development/production comparison.
use std::time::Instant;
use wordcraft_gnome::{sample_session, render_page};
fn main() {
    let start = Instant::now();
    let mut session = sample_session();
    let pages = session.layout().pages.len();
    println!("layout_ms={:.3}", start.elapsed().as_secs_f64()*1000.0);
    let start = Instant::now();
    let mut checksum = 0u64;
    let mut bytes = 0usize;
    for _ in 0..5 {
        for page in 0..pages {
            for scale in [1.0, 0.28] {
                let image = render_page(&mut session, page, scale).unwrap();
                bytes += image.rgba.len();
                for pixel in &image.rgba { checksum = checksum.wrapping_mul(31).wrapping_add(u64::from(*pixel)); }
                std::hint::black_box(&image);
            }
        }
    }
    println!("render_ms={:.3}", start.elapsed().as_secs_f64()*1000.0);
    println!("CHECK pages={pages} bytes={bytes} pixels={checksum:016x}");
}
