use std::collections::HashMap;
use std::sync::Mutex;

use eframe::egui::{self, Color32, Mesh, Pos2, Rect, Stroke, Vec2};

use crate::theme::{self, pal};
use crate::tokens;

// parts here---

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ink {
    Chrome,
    Signal,
    Text,
}

#[derive(Clone, Copy, Debug)]
pub enum Part {
    Capsule { from: (f32, f32), to: (f32, f32), width: f32, ink: Ink },
    Polygon { points: &'static [(f32, f32)], ink: Ink },
}

#[derive(Clone, Copy, Debug)]
pub struct AppMark {
    pub name: &'static str,
    pub parts: &'static [Part],
}

pub const NATIVECOMPUTE: AppMark = AppMark {
    name: "NativeCompute",
    parts: &[
        Part::Polygon { points: crate::brand::SLABS[0], ink: Ink::Signal },
        Part::Polygon { points: crate::brand::SLABS[1], ink: Ink::Signal },
        Part::Polygon { points: crate::brand::SLABS[2], ink: Ink::Signal },
        Part::Polygon { points: crate::brand::SLABS[3], ink: Ink::Signal },
    ],
};

pub const SLAB: f32 = 0.21;
pub const LINE: f32 = 0.075;

pub const fn slab(from: (f32, f32), to: (f32, f32)) -> Part {
    Part::Capsule { from, to, width: SLAB, ink: Ink::Chrome }
}

pub const fn line(from: (f32, f32), to: (f32, f32)) -> Part {
    Part::Capsule { from, to, width: LINE, ink: Ink::Signal }
}

// geometry here---

fn capsule(a: Pos2, b: Pos2, w: f32) -> Vec<Pos2> {
    let along = (b - a).normalized();
    let side = Vec2::new(-along.y, along.x);
    let r = w * 0.5;
    let steps = 10;
    let mut pts = Vec::with_capacity(2 * steps + 2);
    let base = side.y.atan2(side.x);
    for (c, from) in [(b, base + std::f32::consts::PI), (a, base)] {
        for k in 0..=steps {
            let t = from + std::f32::consts::PI * k as f32 / steps as f32;
            pts.push(Pos2::new(c.x + t.cos() * r, c.y + t.sin() * r));
        }
    }
    pts
}

impl AppMark {
    pub fn outlines(&self, rect: Rect) -> Vec<(Vec<Pos2>, Ink, bool)> {
        let side = rect.width().min(rect.height());
        let origin = rect.center() - Vec2::splat(side * 0.5);
        let at = |(x, y): (f32, f32)| origin + Vec2::new(x * side, y * side);
        self.parts
            .iter()
            .map(|part| match *part {
                Part::Capsule { from, to, width, ink } => (capsule(at(from), at(to), width * side), ink, true),
                Part::Polygon { points, ink } => (points.iter().map(|q| at(*q)).collect(), ink, false),
            })
            .collect()
    }

    pub fn paint(&self, painter: &egui::Painter, rect: Rect) {
        let p = pal();
        if rect.width().min(rect.height()) < 4.0 {
            return;
        }
        for (k, (pts, ink, convex)) in self.outlines(rect).into_iter().enumerate() {
            match ink {
                Ink::Chrome => {
                    let (lit, deep) = if p.dark { (p.chrome_hi, p.chrome_shade) } else { (p.chrome, p.chrome_shade) };
                    if convex {
                        theme::fill_grad_poly(painter, &pts, lit, deep, Vec2::new(0.35, 1.0));
                    } else {
                        fill_concave(painter, &pts, lit, self.key(k));
                    }
                }
                Ink::Signal | Ink::Text => {
                    let col = if ink == Ink::Signal { p.signal } else { p.text };
                    if convex {
                        painter.add(egui::Shape::convex_polygon(pts, col, Stroke::NONE));
                    } else {
                        fill_concave(painter, &pts, col, self.key(k));
                    }
                }
            }
        }
    }

    fn key(&self, part: usize) -> (usize, usize) {
        (self.parts.as_ptr() as usize, part)
    }

    pub fn svg(&self) -> String {
        let poly = |pts: &[Pos2]| -> String { pts.iter().map(|q| format!("{:.2},{:.2}", q.x, q.y)).collect::<Vec<_>>().join(" ") };
        let mut s = String::new();
        s.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 512 512\" width=\"512\" height=\"512\">\n");
        s.push_str(&format!("  <title>{}</title>\n", self.name));
        s.push_str("  <rect width=\"512\" height=\"512\" rx=\"116\" fill=\"#000000\"/>\n");
        for (pts, ink, _) in self.outlines(Rect::from_min_size(Pos2::new(56.0, 56.0), Vec2::splat(400.0))) {
            let fill = match ink {
                Ink::Signal => hex(tokens::DARK.signal),
                Ink::Chrome | Ink::Text => hex(tokens::DARK.chrome_hi),
            };
            s.push_str(&format!("  <polygon points=\"{}\" fill=\"{}\"/>\n", poly(&pts), fill));
        }
        s.push_str("</svg>\n");
        s
    }

    pub fn rgba(&self, size: u32) -> Vec<u8> {
        let n = size as f32;
        let scale = n / 512.0;
        let parts: Vec<(Vec<Pos2>, [u8; 3])> = self
            .outlines(Rect::from_min_size(Pos2::new(56.0 * scale, 56.0 * scale), Vec2::splat(400.0 * scale)))
            .into_iter()
            .map(|(pts, ink, _)| {
                let c = match ink {
                    Ink::Signal => tokens::DARK.signal,
                    Ink::Chrome | Ink::Text => tokens::DARK.chrome_hi,
                };
                (pts, [c.r(), c.g(), c.b()])
            })
            .collect();
        let radius = 116.0 * scale;
        let inside_tile = |x: f32, y: f32| {
            let cx = x.clamp(radius, n - radius);
            let cy = y.clamp(radius, n - radius);
            (x - cx).powi(2) + (y - cy).powi(2) <= radius * radius
        };
        const SS: usize = 4;
        let mut out = vec![0u8; (size * size * 4) as usize];
        for py in 0..size {
            for px in 0..size {
                let mut acc = [0.0f32; 4];
                for sy in 0..SS {
                    for sx in 0..SS {
                        let x = px as f32 + (sx as f32 + 0.5) / SS as f32;
                        let y = py as f32 + (sy as f32 + 0.5) / SS as f32;
                        if !inside_tile(x, y) {
                            continue;
                        }
                        let mut c = [0u8, 0, 0];
                        for (pts, col) in &parts {
                            if inside(pts, x, y) {
                                c = *col;
                            }
                        }
                        acc[0] += c[0] as f32;
                        acc[1] += c[1] as f32;
                        acc[2] += c[2] as f32;
                        acc[3] += 255.0;
                    }
                }
                let at = ((py * size + px) * 4) as usize;
                if acc[3] > 0.0 {
                    let cover = acc[3] / 255.0;
                    out[at] = (acc[0] / cover).round() as u8;
                    out[at + 1] = (acc[1] / cover).round() as u8;
                    out[at + 2] = (acc[2] / cover).round() as u8;
                    out[at + 3] = (acc[3] / (SS * SS) as f32).round() as u8;
                }
            }
        }
        out
    }

    pub fn icon_data(&self, size: u32) -> egui::IconData {
        egui::IconData { rgba: self.rgba(size), width: size, height: size }
    }

    pub fn ico(&self) -> Vec<u8> {
        let sizes = [16u32, 24, 32, 48, 64, 256];
        let images: Vec<Vec<u8>> = sizes.iter().map(|&s| self.bmp_entry(s)).collect();
        let mut out = Vec::new();
        out.extend_from_slice(&[0, 0, 1, 0]);
        out.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
        let mut offset = 6 + 16 * sizes.len() as u32;
        for (s, img) in sizes.iter().zip(&images) {
            let dim = if *s >= 256 { 0 } else { *s as u8 };
            out.extend_from_slice(&[dim, dim, 0, 0]);
            out.extend_from_slice(&1u16.to_le_bytes());
            out.extend_from_slice(&32u16.to_le_bytes());
            out.extend_from_slice(&(img.len() as u32).to_le_bytes());
            out.extend_from_slice(&offset.to_le_bytes());
            offset += img.len() as u32;
        }
        for img in images {
            out.extend_from_slice(&img);
        }
        out
    }

    fn bmp_entry(&self, size: u32) -> Vec<u8> {
        let px = self.rgba(size);
        let mask_row = size.div_ceil(32) * 4;
        let mut out = Vec::new();
        out.extend_from_slice(&40u32.to_le_bytes());
        out.extend_from_slice(&(size as i32).to_le_bytes());
        out.extend_from_slice(&((size * 2) as i32).to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&32u16.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(size * size * 4 + mask_row * size).to_le_bytes());
        out.extend_from_slice(&[0u8; 16]);
        for y in (0..size).rev() {
            for x in 0..size {
                let at = ((y * size + x) * 4) as usize;
                out.extend_from_slice(&[px[at + 2], px[at + 1], px[at], px[at + 3]]);
            }
        }
        out.extend(std::iter::repeat(0u8).take((mask_row * size) as usize));
        out
    }
}

fn hex(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

fn inside(pts: &[Pos2], x: f32, y: f32) -> bool {
    let mut odd = false;
    let mut j = pts.len() - 1;
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[j]);
        if (a.y > y) != (b.y > y) && x < (b.x - a.x) * (y - a.y) / (b.y - a.y) + a.x {
            odd = !odd;
        }
        j = i;
    }
    odd
}

// concave fill here---

type Cache = HashMap<(usize, usize), Vec<u32>>;

static TRIANGLES: Mutex<Option<Cache>> = Mutex::new(None);

fn fill_concave(painter: &egui::Painter, pts: &[Pos2], color: Color32, key: (usize, usize)) {
    let indices = {
        let mut cache = TRIANGLES.lock().unwrap_or_else(|e| e.into_inner());
        let map = cache.get_or_insert_with(HashMap::new);
        map.entry(key).or_insert_with(|| triangulate(pts)).clone()
    };
    let mut mesh = Mesh::default();
    for q in pts {
        mesh.colored_vertex(*q, color);
    }
    mesh.indices = indices;
    painter.add(egui::Shape::mesh(mesh));
}

pub fn triangulate(pts: &[Pos2]) -> Vec<u32> {
    let n = pts.len();
    if n < 3 {
        return Vec::new();
    }
    let area: f32 = (0..n).map(|i| {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        a.x * b.y - b.x * a.y
    }).sum();
    let ccw = area > 0.0;
    let mut idx: Vec<usize> = (0..n).collect();
    let mut out = Vec::with_capacity((n - 2) * 3);
    let cross = |a: Pos2, b: Pos2, c: Pos2| (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    let mut guard = 0;
    while idx.len() > 3 && guard < n * n {
        guard += 1;
        let m = idx.len();
        let mut clipped = false;
        for i in 0..m {
            let (ia, ib, ic) = (idx[(i + m - 1) % m], idx[i], idx[(i + 1) % m]);
            let (a, b, c) = (pts[ia], pts[ib], pts[ic]);
            let turn = cross(a, b, c);
            if (turn > 0.0) != ccw || turn.abs() < 1e-9 {
                continue;
            }
            let empty = idx.iter().all(|&j| {
                if j == ia || j == ib || j == ic {
                    return true;
                }
                let q = pts[j];
                let (d1, d2, d3) = (cross(a, b, q), cross(b, c, q), cross(c, a, q));
                let neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
                let pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
                neg && pos
            });
            if empty {
                out.extend_from_slice(&[ia as u32, ib as u32, ic as u32]);
                idx.remove(i);
                clipped = true;
                break;
            }
        }
        if !clipped {
            let m = idx.len();
            out.extend_from_slice(&[idx[0] as u32, idx[1] as u32, idx[2 % m] as u32]);
            idx.remove(1);
        }
    }
    if idx.len() == 3 {
        out.extend_from_slice(&[idx[0] as u32, idx[1] as u32, idx[2] as u32]);
    }
    out
}

// spinner here---

pub fn spinner(painter: &egui::Painter, centre: Pos2, radius: f32, t: f32, color: Color32) {
    let head = t * std::f32::consts::TAU * 1.35;
    let sweep = 0.55 + 0.85 * (0.5 - 0.5 * (t * std::f32::consts::TAU * 0.9).cos());
    let steps = 64;
    let w = radius * 0.14;
    for i in 0..steps {
        let f0 = i as f32 / steps as f32;
        let f1 = (i + 1) as f32 / steps as f32;
        let a0 = head - sweep * (1.0 - f0);
        let a1 = head - sweep * (1.0 - f1);
        let fade = f0.powf(0.55);
        let col = theme::wash(color, (235.0 * fade) as u8);
        painter.line_segment(
            [
                Pos2::new(centre.x + a0.cos() * radius, centre.y + a0.sin() * radius),
                Pos2::new(centre.x + a1.cos() * radius, centre.y + a1.sin() * radius),
            ],
            Stroke::new(w * (0.55 + 0.45 * fade), col),
        );
    }
    painter.circle_filled(Pos2::new(centre.x + head.cos() * radius, centre.y + head.sin() * radius), w * 0.5, color);
}
