#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, Ordering};

use eframe::egui::{self, Id};

static ENABLED: AtomicBool = AtomicBool::new(true);

pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

// curves here---

pub fn smootherstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

pub fn ease_out_expo(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t >= 1.0 {
        1.0
    } else {
        1.0 - (-9.0 * t).exp2()
    }
}

pub fn ease_out_elastic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t <= 0.0 || t >= 1.0 {
        return t;
    }
    let c = std::f32::consts::TAU / 3.0;
    (-9.0 * t).exp2() * ((t * 10.0 - 0.75) * c).sin() + 1.0
}

pub fn bounce(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t >= 1.0 {
        return 1.0;
    }
    1.0 - (-6.2 * t).exp() * (8.4 * t).cos()
}

pub fn bounce_travel(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t >= 1.0 {
        return 1.0;
    }
    1.0 - (-2.2 * t).exp() * (2.55 * t).cos() * (1.0 - t).sqrt()
}

pub fn bounce_in(ctx: &egui::Context, id: impl std::hash::Hash, on: bool, secs: f32) -> f32 {
    if !enabled() {
        return if on { 1.0 } else { 0.0 };
    }
    let raw = ctx.animate_bool_with_time(Id::new(id), on, secs);
    if on {
        bounce(raw)
    } else {
        smootherstep(raw)
    }
}

pub fn ease_out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
}

pub fn pop(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t >= 1.0 {
        return 1.0;
    }
    1.0 - (-7.0 * t).exp() * (10.0 * t).cos()
}

// drivers here---

pub fn ease(ctx: &egui::Context, id: impl std::hash::Hash, on: bool, secs: f32) -> f32 {
    if !enabled() {
        return if on { 1.0 } else { 0.0 };
    }
    smootherstep(ctx.animate_bool_with_time(Id::new(id), on, secs))
}

pub fn to(ctx: &egui::Context, id: impl std::hash::Hash, target: f32, secs: f32) -> f32 {
    if !enabled() {
        return target;
    }
    ctx.animate_value_with_time(Id::new(id), target, secs)
}

pub fn glide(ctx: &egui::Context, id: impl std::hash::Hash, target: f32, secs: f32) -> f32 {
    glide_on(ctx, id, target, secs, ease_out_expo)
}

pub fn spring_to(ctx: &egui::Context, id: impl std::hash::Hash, target: f32, secs: f32) -> f32 {
    glide_on(ctx, id, target, secs, bounce)
}

fn glide_on(
    ctx: &egui::Context,
    id: impl std::hash::Hash,
    target: f32,
    secs: f32,
    curve: fn(f32) -> f32,
) -> f32 {
    if !enabled() {
        return target;
    }
    let id = Id::new(id);
    let raw = ctx.animate_value_with_time(id.with("raw"), target, secs);
    let from = ctx.data_mut(|d| *d.get_temp_mut_or(id.with("from"), raw));
    let span = target - from;
    if span.abs() < 0.0001 {
        ctx.data_mut(|d| d.insert_temp(id.with("from"), target));
        return target;
    }
    let t = ((raw - from) / span).clamp(0.0, 1.0);
    if t >= 0.999 {
        ctx.data_mut(|d| d.insert_temp(id.with("from"), target));
    }
    from + span * curve(t)
}

pub fn clock(ctx: &egui::Context) -> f32 {
    ctx.input(|i| i.time) as f32
}

pub fn breathe(ctx: &egui::Context, period: f32) -> f32 {
    if !enabled() {
        return 0.5;
    }
    let t = clock(ctx) / period.max(0.05);
    0.5 - 0.5 * (t * std::f32::consts::TAU).cos()
}

pub fn keep_going(ctx: &egui::Context) {
    if enabled() {
        ctx.request_repaint();
    }
}

pub const HOVER: f32 = 0.24;
