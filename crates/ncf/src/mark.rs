use std::collections::HashMap;
use std::sync::Mutex;

use eframe::egui::{self, Color32, Mesh, Pos2, Rect, Stroke, Vec2};

use crate::icon::{self, Frame, Image, Tile};
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
pub enum Glyph {
    Parts(&'static [Part]),
    Png(&'static [u8]),
}

#[derive(Clone, Copy, Debug)]
pub struct AppMark {
    pub name: &'static str,
    pub glyph: Glyph,
    pub tile: Tile,
}

pub const DRAWN_TILE: Tile = Tile { mark: 400.0 / 512.0, ..crate::icon::BENTO_TILE };

pub const NATIVECOMPUTE: AppMark = AppMark::drawn(
    "NativeCompute",
    &[
        Part::Polygon { points: crate::brand::SLABS[0], ink: Ink::Text },
        Part::Polygon { points: crate::brand::SLABS[1], ink: Ink::Text },
        Part::Polygon { points: crate::brand::SLABS[2], ink: Ink::Text },
        Part::Polygon { points: crate::brand::SLABS[3], ink: Ink::Text },
    ],
);

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
    pub const fn drawn(name: &'static str, parts: &'static [Part]) -> AppMark {
        AppMark { name, glyph: Glyph::Parts(parts), tile: DRAWN_TILE }
    }

    pub const fn raster(name: &'static str, png: &'static [u8]) -> AppMark {
        AppMark { name, glyph: Glyph::Png(png), tile: crate::icon::BENTO_TILE }
    }

    pub fn parts(&self) -> &'static [Part] {
        match self.glyph {
            Glyph::Parts(p) => p,
            Glyph::Png(_) => &[],
        }
    }

    pub fn outlines(&self, rect: Rect) -> Vec<(Vec<Pos2>, Ink, bool)> {
        let side = rect.width().min(rect.height());
        let origin = rect.center() - Vec2::splat(side * 0.5);
        let at = |(x, y): (f32, f32)| origin + Vec2::new(x * side, y * side);
        self.parts()
            .iter()
            .map(|part| match *part {
                Part::Capsule { from, to, width, ink } => (capsule(at(from), at(to), width * side), ink, true),
                Part::Polygon { points, ink } => (points.iter().map(|q| at(*q)).collect(), ink, false),
            })
            .collect()
    }

    pub fn paint(&self, painter: &egui::Painter, rect: Rect) {
        let side = rect.width().min(rect.height());
        if side < 4.0 {
            return;
        }
        let ctx = painter.ctx();
        let px = (side * ctx.pixels_per_point()).round().clamp(8.0, 1024.0) as u32;
        let id = egui::Id::new(("ncf-mark", self.name, px));
        let tex: Option<egui::TextureHandle> = ctx.data(|d| d.get_temp(id));
        let tex = tex.unwrap_or_else(|| {
            let img = egui::ColorImage::from_rgba_unmultiplied([px as usize, px as usize], &self.rgba(px));
            let t = ctx.load_texture(format!("ncf-mark-{}-{px}", self.name), img, egui::TextureOptions::LINEAR);
            ctx.data_mut(|d| d.insert_temp(id, t.clone()));
            t
        });
        let at = Rect::from_center_size(rect.center(), Vec2::splat(side));
        painter.image(tex.id(), at, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
    }

    pub fn paint_glyph(&self, painter: &egui::Painter, rect: Rect) {
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
        (self.parts().as_ptr() as usize, part)
    }

    pub fn glyph_image(&self, px: u32) -> Option<std::sync::Arc<Image>> {
        match self.glyph {
            Glyph::Png(bytes) => icon::decode(bytes),
            Glyph::Parts(_) => Some(std::sync::Arc::new(self.drawn_image(px))),
        }
    }

    fn drawn_image(&self, px: u32) -> Image {
        let n = px.max(1) as usize;
        let parts: Vec<(Vec<Pos2>, [f32; 3], Rect)> = self
            .outlines(Rect::from_min_size(Pos2::ZERO, Vec2::splat(n as f32)))
            .into_iter()
            .map(|(pts, ink, _)| {
                let c = ink_rgb(ink);
                let bounds = Rect::from_points(&pts);
                (pts, [c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0], bounds)
            })
            .collect();
        const SS: usize = 4;
        let mut img = Image::new(n, n);
        for py in 0..n {
            for px in 0..n {
                let mut acc = [0.0f32; 4];
                for sy in 0..SS {
                    for sx in 0..SS {
                        let x = px as f32 + (sx as f32 + 0.5) / SS as f32;
                        let y = py as f32 + (sy as f32 + 0.5) / SS as f32;
                        let mut hit = None;
                        for (pts, col, b) in &parts {
                            if x >= b.min.x && x <= b.max.x && y >= b.min.y && y <= b.max.y && inside(pts, x, y) {
                                hit = Some(*col);
                            }
                        }
                        if let Some(c) = hit {
                            acc[0] += c[0];
                            acc[1] += c[1];
                            acc[2] += c[2];
                            acc[3] += 1.0;
                        }
                    }
                }
                let k = 1.0 / (SS * SS) as f32;
                img.px[py * n + px] = [acc[0] * k, acc[1] * k, acc[2] * k, acc[3] * k];
            }
        }
        img
    }

    pub fn compose(&self, px: u32, frame: Frame) -> Vec<u8> {
        let mark_px = ((self.tile.mark * px as f32).round().max(1.0)) as u32;
        let empty = Image::new(1, 1);
        let glyph = self.glyph_image(mark_px);
        icon::compose(&self.tile, glyph.as_deref().unwrap_or(&empty), px, frame).to_rgba()
    }

    pub fn svg(&self) -> String {
        let mut s = String::new();
        s.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 512 512\" width=\"512\" height=\"512\">\n");
        s.push_str(&format!("  <title>{}</title>\n", self.name));
        if let Glyph::Png(_) = self.glyph {
            let png = icon::png_bytes(&self.compose(512, Frame::Full), 512, 512);
            s.push_str(&format!("  <image width=\"512\" height=\"512\" href=\"data:image/png;base64,{}\"/>\n", icon::base64(&png)));
            s.push_str("</svg>\n");
            return s;
        }
        let t = &self.tile;
        let rgb = |c: [u8; 3]| format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2]);
        s.push_str(&format!("  <rect width=\"512\" height=\"512\" rx=\"{:.1}\" fill=\"{}\"/>\n", t.radius * 512.0, rgb(t.fill)));
        if let Some(g) = &t.glow {
            s.push_str("  <defs>\n    <radialGradient id=\"glow\" gradientUnits=\"userSpaceOnUse\"");
            s.push_str(&format!(" cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\">\n", g.at.0 * 512.0, g.at.1 * 512.0, g.reach.0 * 512.0));
            for (at, c) in g.stops {
                s.push_str(&format!(
                    "      <stop offset=\"{:.2}\" stop-color=\"{}\" stop-opacity=\"{:.3}\"/>\n",
                    at,
                    rgb([c[0], c[1], c[2]]),
                    c[3] as f32 / 255.0
                ));
            }
            s.push_str("    </radialGradient>\n  </defs>\n");
            s.push_str(&format!("  <rect width=\"512\" height=\"512\" rx=\"{:.1}\" fill=\"url(#glow)\"/>\n", t.radius * 512.0));
        }
        let poly = |pts: &[Pos2]| -> String { pts.iter().map(|q| format!("{:.2},{:.2}", q.x, q.y)).collect::<Vec<_>>().join(" ") };
        let box_side = t.mark * 512.0;
        let at = (512.0 - box_side) * 0.5;
        for (pts, ink, _) in self.outlines(Rect::from_min_size(Pos2::new(at, at), Vec2::splat(box_side))) {
            s.push_str(&format!("  <polygon points=\"{}\" fill=\"{}\"/>\n", poly(&pts), hex(ink_rgb(ink))));
        }
        s.push_str("</svg>\n");
        s
    }

    pub fn rgba(&self, size: u32) -> Vec<u8> {
        self.compose(size, Frame::Full)
    }

    pub fn icon_data(&self, size: u32) -> egui::IconData {
        egui::IconData { rgba: self.rgba(size), width: size, height: size }
    }

    pub fn png(&self, size: u32) -> Vec<u8> {
        icon::png_bytes(&self.rgba(size), size, size)
    }

    pub fn png_padded(&self, size: u32) -> Vec<u8> {
        icon::png_bytes(&self.compose(size, Frame::Padded), size, size)
    }

    pub fn icns(&self) -> Vec<u8> {
        const SLOTS: [(&[u8; 4], u32); 10] = [
            (b"icp4", 16),
            (b"icp5", 32),
            (b"ic11", 32),
            (b"ic12", 64),
            (b"ic07", 128),
            (b"ic13", 256),
            (b"ic08", 256),
            (b"ic14", 512),
            (b"ic09", 512),
            (b"ic10", 1024),
        ];
        let mut body = Vec::new();
        let mut made: Vec<(u32, Vec<u8>)> = Vec::new();
        for (kind, size) in SLOTS {
            if !made.iter().any(|(s, _)| *s == size) {
                made.push((size, self.png_padded(size)));
            }
            let png = &made.iter().find(|(s, _)| *s == size).map(|(_, p)| p.clone()).unwrap_or_default();
            body.extend_from_slice(kind);
            body.extend_from_slice(&((png.len() as u32 + 8).to_be_bytes()));
            body.extend_from_slice(png);
        }
        let mut out = Vec::with_capacity(body.len() + 8);
        out.extend_from_slice(b"icns");
        out.extend_from_slice(&((body.len() as u32 + 8).to_be_bytes()));
        out.extend_from_slice(&body);
        out
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

fn ink_rgb(ink: Ink) -> Color32 {
    match ink {
        Ink::Signal => tokens::DARK.signal,
        Ink::Chrome | Ink::Text => tokens::DARK.chrome_hi,
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

// logo here---

const LOGO_SUBROWS: usize = 5;

pub fn logo_width(height: f32) -> f32 {
    height * crate::brand::LOGO_SIZE.0 / crate::brand::LOGO_SIZE.1
}

pub fn logo_coverage(height_px: usize) -> (usize, usize, Vec<f32>) {
    let s = height_px as f32 / crate::brand::LOGO_SIZE.1;
    let w = (crate::brand::LOGO_SIZE.0 * s).ceil() as usize;
    let h = height_px;
    let mut cov = vec![0.0_f32; w * h];
    let mut xs: Vec<(f32, i32)> = Vec::new();
    let weight = 1.0 / LOGO_SUBROWS as f32;
    for sub in 0..h * LOGO_SUBROWS {
        let y = (sub as f32 + 0.5) / LOGO_SUBROWS as f32;
        xs.clear();
        for poly in crate::brand::LOGO {
            for k in 0..poly.len() {
                let (x0, y0) = poly[k];
                let (x1, y1) = poly[(k + 1) % poly.len()];
                let (x0, y0, x1, y1) = (x0 * s, y0 * s, x1 * s, y1 * s);
                if (y0 <= y) != (y1 <= y) {
                    let t = (y - y0) / (y1 - y0);
                    xs.push((x0 + t * (x1 - x0), if y1 > y0 { 1 } else { -1 }));
                }
            }
        }
        xs.sort_by(|a, b| a.0.total_cmp(&b.0));
        let row = &mut cov[(sub / LOGO_SUBROWS) * w..(sub / LOGO_SUBROWS + 1) * w];
        let (mut wind, mut start) = (0, 0.0_f32);
        for &(x, dir) in &xs {
            let before = wind;
            wind += dir;
            if before == 0 && wind != 0 {
                start = x;
            } else if before != 0 && wind == 0 {
                let (a, b) = (start.max(0.0), x.min(w as f32));
                let mut px = a.floor() as usize;
                while (px as f32) < b && px < w {
                    let overlap = b.min(px as f32 + 1.0) - a.max(px as f32);
                    row[px] += overlap.max(0.0) * weight;
                    px += 1;
                }
            }
        }
    }
    (w, h, cov)
}

fn logo_texture(ctx: &egui::Context, height_px: usize) -> egui::TextureHandle {
    let id = egui::Id::new(("ncf-logo", height_px));
    if let Some(tex) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
        return tex;
    }
    let (w, h, cov) = logo_coverage(height_px);
    let pixels = cov.iter().map(|c| Color32::from_white_alpha((c.clamp(0.0, 1.0) * 255.0).round() as u8)).collect();
    let image = egui::ColorImage { size: [w, h], pixels };
    let tex = ctx.load_texture(format!("ncf-logo-{height_px}"), image, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, tex.clone()));
    tex
}

pub fn logo(painter: &egui::Painter, left_mid: Pos2, height: f32, tint: Color32) -> Rect {
    let ppp = painter.ctx().pixels_per_point();
    let px = (height * ppp).round().max(4.0) as usize;
    let tex = logo_texture(painter.ctx(), px);
    let size = tex.size_vec2() / ppp;
    let min = painter.round_pos_to_pixels(Pos2::new(left_mid.x, left_mid.y - size.y * 0.5));
    let rect = Rect::from_min_size(min, size);
    painter.image(tex.id(), rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), tint);
    rect
}
