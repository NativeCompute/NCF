use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glow {
    pub at: (f32, f32),
    pub reach: (f32, f32),
    pub stops: [(f32, [u8; 4]); 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tile {
    pub fill: [u8; 3],
    pub glow: Option<Glow>,
    pub radius: f32,
    pub mark: f32,
}

pub const BENTO_TILE: Tile = Tile {
    fill: [0x13, 0x13, 0x18],
    glow: Some(Glow {
        at: (0.0, 0.0),
        reach: (0.75, 0.75),
        stops: [(0.0, [255, 122, 70, 217]), (0.36, [255, 106, 61, 77]), (0.70, [255, 106, 61, 0])],
    }),
    radius: 185.0 / 824.0,
    mark: 650.0 / 824.0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frame {
    Full,
    Padded,
}

pub const PAD: f32 = 100.0 / 1024.0;

pub struct Image {
    pub width: usize,
    pub height: usize,
    pub px: Vec<[f32; 4]>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Image {
        Image { width, height, px: vec![[0.0; 4]; width * height] }
    }

    pub fn at(&self, x: usize, y: usize) -> [f32; 4] {
        self.px[y * self.width + x]
    }

    pub fn to_rgba(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.px.len() * 4);
        for c in &self.px {
            let a = c[3].clamp(0.0, 1.0);
            let un = |v: f32| if a > 0.0 { (v / a).clamp(0.0, 1.0) } else { 0.0 };
            out.push((un(c[0]) * 255.0).round() as u8);
            out.push((un(c[1]) * 255.0).round() as u8);
            out.push((un(c[2]) * 255.0).round() as u8);
            out.push((a * 255.0).round() as u8);
        }
        out
    }
}

// decode here---

static DECODED: Mutex<Option<HashMap<usize, Arc<Image>>>> = Mutex::new(None);

pub fn decode(bytes: &'static [u8]) -> Option<Arc<Image>> {
    let key = bytes.as_ptr() as usize;
    if let Some(img) = DECODED.lock().ok().and_then(|g| g.as_ref().and_then(|m| m.get(&key).cloned())) {
        return Some(img);
    }
    let img = Arc::new(decode_png(bytes)?);
    if let Ok(mut g) = DECODED.lock() {
        g.get_or_insert_with(HashMap::new).insert(key, img.clone());
    }
    Some(img)
}

pub fn decode_png(bytes: &[u8]) -> Option<Image> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    let (w, h) = (info.width as usize, info.height as usize);
    let channels = match info.color_type {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Grayscale => 1,
        png::ColorType::Indexed => return None,
    };
    let mut img = Image::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let i = y * info.line_size + x * channels;
            let s = |k: usize| buf[i + k] as f32 / 255.0;
            let (r, g, b, a) = match channels {
                4 => (s(0), s(1), s(2), s(3)),
                3 => (s(0), s(1), s(2), 1.0),
                2 => (s(0), s(0), s(0), s(1)),
                _ => (s(0), s(0), s(0), 1.0),
            };
            img.px[y * w + x] = [r * a, g * a, b * a, a];
        }
    }
    Some(img)
}

// resample here---

fn weights(src: usize, dst: usize) -> Vec<Vec<(usize, f32)>> {
    let scale = src as f32 / dst as f32;
    (0..dst)
        .map(|d| {
            let (a, b) = (d as f32 * scale, (d as f32 + 1.0) * scale);
            if scale <= 1.0 {
                let c = ((a + b) * 0.5 - 0.5).clamp(0.0, (src - 1) as f32);
                let i = c.floor() as usize;
                let f = c - i as f32;
                let j = (i + 1).min(src - 1);
                return vec![(i, 1.0 - f), (j, f)];
            }
            let mut w = Vec::new();
            let mut s = a.floor() as usize;
            while (s as f32) < b && s < src {
                let lo = (s as f32).max(a);
                let hi = ((s + 1) as f32).min(b);
                if hi > lo {
                    w.push((s, (hi - lo) / scale));
                }
                s += 1;
            }
            w
        })
        .collect()
}

pub fn resize(src: &Image, w: usize, h: usize) -> Image {
    let (w, h) = (w.max(1), h.max(1));
    let wx = weights(src.width, w);
    let wy = weights(src.height, h);
    let mut mid = Image::new(w, src.height);
    for y in 0..src.height {
        for (x, ws) in wx.iter().enumerate() {
            let mut acc = [0.0f32; 4];
            for &(sx, k) in ws {
                let c = src.at(sx, y);
                for n in 0..4 {
                    acc[n] += c[n] * k;
                }
            }
            mid.px[y * w + x] = acc;
        }
    }
    let mut out = Image::new(w, h);
    for (y, ws) in wy.iter().enumerate() {
        for x in 0..w {
            let mut acc = [0.0f32; 4];
            for &(sy, k) in ws {
                let c = mid.at(x, sy);
                for n in 0..4 {
                    acc[n] += c[n] * k;
                }
            }
            out.px[y * w + x] = acc;
        }
    }
    out
}

// blur here---

fn blur_line(src: &[f32], dst: &mut [f32], r: usize) {
    let n = src.len();
    let mut pre = vec![0.0f32; n + 1];
    for i in 0..n {
        pre[i + 1] = pre[i] + src[i];
    }
    let span = (2 * r + 1) as f32;
    for (i, d) in dst.iter_mut().enumerate() {
        let lo = i.saturating_sub(r);
        let hi = (i + r + 1).min(n);
        *d = (pre[hi] - pre[lo]) / span;
    }
}

fn box_blur(a: &mut [f32], w: usize, h: usize, r: usize) {
    if r == 0 {
        return;
    }
    let mut line = vec![0.0f32; w.max(h)];
    let mut out = vec![0.0f32; w.max(h)];
    for y in 0..h {
        line[..w].copy_from_slice(&a[y * w..(y + 1) * w]);
        blur_line(&line[..w], &mut out[..w], r);
        a[y * w..(y + 1) * w].copy_from_slice(&out[..w]);
    }
    for x in 0..w {
        for y in 0..h {
            line[y] = a[y * w + x];
        }
        blur_line(&line[..h], &mut out[..h], r);
        for y in 0..h {
            a[y * w + x] = out[y];
        }
    }
}

pub fn gaussian(a: &mut [f32], w: usize, h: usize, sigma: f32) {
    if sigma < 0.3 {
        return;
    }
    let ideal = (12.0 * sigma * sigma / 3.0 + 1.0).sqrt();
    let r = ((ideal - 1.0) * 0.5).round().max(1.0) as usize;
    for _ in 0..3 {
        box_blur(a, w, h, r);
    }
}

fn shifted(mask: &[f32], w: usize, h: usize, dx: f32, dy: f32) -> Vec<f32> {
    let (ox, oy) = (dx.round() as isize, dy.round() as isize);
    let mut out = vec![0.0f32; mask.len()];
    for y in 0..h as isize {
        for x in 0..w as isize {
            let (sx, sy) = (x - ox, y - oy);
            if sx >= 0 && sy >= 0 && (sx as usize) < w && (sy as usize) < h {
                out[(y as usize) * w + x as usize] = mask[(sy as usize) * w + sx as usize];
            }
        }
    }
    out
}

pub fn shadow(mask: &[f32], w: usize, h: usize, dy: f32, blur: f32, alpha: f32) -> Vec<f32> {
    let mut s = shifted(mask, w, h, 0.0, dy);
    gaussian(&mut s, w, h, blur * 0.5);
    for v in &mut s {
        *v *= alpha;
    }
    s
}

// tile here---

pub fn tile_cover(x: f32, y: f32, left: f32, top: f32, side: f32, radius: f32) -> f32 {
    let r = radius.min(side * 0.5);
    let (cx, cy) = (left + side * 0.5, top + side * 0.5);
    let half = side * 0.5 - r;
    let qx = (x - cx).abs() - half;
    let qy = (y - cy).abs() - half;
    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    let d = outside + qx.max(qy).min(0.0) - r;
    (0.5 - d).clamp(0.0, 1.0)
}

fn glow_at(g: &Glow, u: f32, v: f32) -> [f32; 4] {
    let dx = (u - g.at.0) / g.reach.0.max(0.001);
    let dy = (v - g.at.1) / g.reach.1.max(0.001);
    let d = (dx * dx + dy * dy).sqrt();
    let pm = |c: [u8; 4]| {
        let a = c[3] as f32 / 255.0;
        [c[0] as f32 / 255.0 * a, c[1] as f32 / 255.0 * a, c[2] as f32 / 255.0 * a, a]
    };
    let s = &g.stops;
    if d <= s[0].0 {
        return pm(s[0].1);
    }
    for k in 0..s.len() - 1 {
        let (a, b) = (s[k], s[k + 1]);
        if d <= b.0 {
            let t = (d - a.0) / (b.0 - a.0).max(0.0001);
            let (x, y) = (pm(a.1), pm(b.1));
            return [x[0] + (y[0] - x[0]) * t, x[1] + (y[1] - x[1]) * t, x[2] + (y[2] - x[2]) * t, x[3] + (y[3] - x[3]) * t];
        }
    }
    pm(s[s.len() - 1].1)
}

fn over(top: [f32; 4], bottom: [f32; 4]) -> [f32; 4] {
    let k = 1.0 - top[3];
    [top[0] + bottom[0] * k, top[1] + bottom[1] * k, top[2] + bottom[2] * k, top[3] + bottom[3] * k]
}

pub fn compose(tile: &Tile, mark: &Image, px: u32, frame: Frame) -> Image {
    let n = px.max(1) as usize;
    let nf = n as f32;
    let margin = if frame == Frame::Padded { (nf * PAD).round() } else { 0.0 };
    let side = nf - 2.0 * margin;
    let scale = side / 824.0;
    let radius = tile.radius * side;
    let mut cover = vec![0.0f32; n * n];
    for y in 0..n {
        for x in 0..n {
            cover[y * n + x] = tile_cover(x as f32 + 0.5, y as f32 + 0.5, margin, margin, side, radius);
        }
    }
    let mut out = Image::new(n, n);
    if frame == Frame::Padded && side >= 32.0 {
        let a = shadow(&cover, n, n, 10.0 * scale, 24.0 * scale, 0.22);
        let b = shadow(&cover, n, n, 2.0 * scale, 6.0 * scale, 0.16);
        for i in 0..n * n {
            let s = over([0.0, 0.0, 0.0, b[i]], [0.0, 0.0, 0.0, a[i]]);
            out.px[i] = s;
        }
    }
    let fill = [tile.fill[0] as f32 / 255.0, tile.fill[1] as f32 / 255.0, tile.fill[2] as f32 / 255.0, 1.0];
    let mw = (tile.mark * side).round().max(1.0) as usize;
    let mh = ((mark.height as f32 * mw as f32 / mark.width.max(1) as f32).round().max(1.0)) as usize;
    let small = resize(mark, mw, mh);
    let mx = ((nf - mw as f32) * 0.5).round() as isize;
    let my = ((nf - mh as f32) * 0.5).round() as isize;
    let mut layer = Image::new(n, n);
    for y in 0..mh {
        for x in 0..mw {
            let (tx, ty) = (x as isize + mx, y as isize + my);
            if tx >= 0 && ty >= 0 && (tx as usize) < n && (ty as usize) < n {
                layer.px[ty as usize * n + tx as usize] = small.at(x, y);
            }
        }
    }
    let mark_shadow = if side >= 48.0 {
        let alpha: Vec<f32> = layer.px.iter().map(|c| c[3]).collect();
        Some(shadow(&alpha, n, n, 8.0 * scale, 20.0 * scale, 0.35))
    } else {
        None
    };
    for y in 0..n {
        for x in 0..n {
            let i = y * n + x;
            let c = cover[i];
            if c <= 0.0 {
                continue;
            }
            let mut px = fill;
            if let Some(g) = &tile.glow {
                let u = (x as f32 + 0.5 - margin) / side;
                let v = (y as f32 + 0.5 - margin) / side;
                px = over(glow_at(g, u, v), px);
            }
            if let Some(s) = &mark_shadow {
                px = over([0.0, 0.0, 0.0, s[i]], px);
            }
            px = over(layer.px[i], px);
            let clipped = [px[0] * c, px[1] * c, px[2] * c, px[3] * c];
            out.px[i] = over(clipped, out.px[i]);
        }
    }
    out
}

pub fn png_bytes(rgba: &[u8], w: u32, h: u32) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, w, h);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::High);
        if let Ok(mut writer) = encoder.write_header() {
            let _ = writer.write_image_data(rgba);
        }
    }
    out
}

pub fn base64(bytes: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(A[(n >> 18) as usize & 63] as char);
        out.push(A[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { A[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { A[n as usize & 63] as char } else { '=' });
    }
    out
}
