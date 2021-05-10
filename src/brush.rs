use crate::kit::shape2d::{Fill, Shape, Stroke};
use crate::kit::Origin;
use crate::view::{ViewCoords, ViewExtent};

use rgx::core::{Rect, Rgba8};
use rgx::math::{Point2, Vector2};

use std::collections::BTreeSet;
use std::fmt;

/// Input state of the brush.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum BrushState {
    /// Not currently drawing.
    NotDrawing,
    /// Drawing has just started.
    DrawStarted(ViewExtent),
    /// Drawing.
    Drawing(ViewExtent),
    /// Drawing has just ended.
    DrawEnded(ViewExtent),
}

/// Brush mode. Any number of these modes can be active at once.
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub enum BrushMode {
    /// Erase pixels.
    Erase,
    /// Draw on all frames at once.
    Multi,
    /// Pixel-perfect mode.
    Perfect,
}

impl fmt::Display for BrushMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Erase => "erase".fmt(f),
            Self::Multi => "multi".fmt(f),
            Self::Perfect => "perfect".fmt(f),
        }
    }
}

/// Brush context.
#[derive(Debug, Clone)]
pub struct Brush {
    /// Brush size in pixels.
    pub size: usize,
    /// Current brush state.
    pub state: BrushState,
    /// Current brush stroke.
    pub stroke: Vec<Point2<i32>>,
    /// Current stroke color.
    pub color: Rgba8,

    /// Currently active brush modes.
    modes: BTreeSet<BrushMode>,
    /// Current brush position.
    curr: Point2<i32>,
    /// Previous brush position.
    prev: Point2<i32>,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            size: 1,
            state: BrushState::NotDrawing,
            stroke: Vec::with_capacity(32),
            color: Rgba8::TRANSPARENT,
            modes: BTreeSet::new(),
            curr: Point2::new(0, 0),
            prev: Point2::new(0, 0),
        }
    }
}

impl Brush {
    /// Check whether the given mode is active.
    pub fn is_set(&self, m: BrushMode) -> bool {
        self.modes.contains(&m)
    }

    /// Activate the given brush mode.
    pub fn set(&mut self, m: BrushMode) -> bool {
        self.modes.insert(m)
    }

    /// De-activate the given brush mode.
    pub fn unset(&mut self, m: BrushMode) -> bool {
        self.modes.remove(&m)
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.modes.clear();
    }

    /// Run every frame by the session.
    pub fn update(&mut self) {
        if let BrushState::DrawEnded(_) = self.state {
            self.state = BrushState::NotDrawing;
            self.stroke.clear();
        }
    }

    /// Start drawing. Called when input is first pressed.
    pub fn start_drawing(
        &mut self,
        p: ViewCoords<i32>,
        color: Rgba8,
        extent: ViewExtent,
    ) {
        self.state = BrushState::DrawStarted(extent);
        self.color = color;
        self.stroke = Vec::with_capacity(32);
        self.draw(p);
    }

    /// Draw. Called while input is pressed.
    pub fn draw(&mut self, p: ViewCoords<i32>) {
        self.prev = if let BrushState::DrawStarted(_) = self.state {
            *p
        } else {
            self.curr
        };
        self.curr = *p;

        Brush::line(self.prev, self.curr, &mut self.stroke);
        self.stroke.dedup();

        if self.is_set(BrushMode::Perfect) {
            self.stroke = Brush::filter(&self.stroke);
        }

        match self.state {
            BrushState::Drawing(_) => {}
            BrushState::DrawStarted(extent) => {
                self.state = BrushState::Drawing(extent);
            }
            _ => unreachable!(),
        }
    }
    /// Stop drawing. Called when input is released.
    pub fn stop_drawing(&mut self) {
        match self.state {
            BrushState::DrawStarted(ex) | BrushState::Drawing(ex) => {
                self.state = BrushState::DrawEnded(ex);
            }
            _ => unreachable!(),
        }
    }

    pub fn output(
        &self,
        stroke: Stroke,
        fill: Fill,
        scale: f32,
        origin: Origin,
    ) -> Vec<Shape> {
        let pixels = match self.state {
            BrushState::DrawStarted(extent)
            | BrushState::Drawing(extent)
            | BrushState::DrawEnded(extent) => {
                let ViewExtent { fw, nframes, .. } = extent;
                let stroke = self.stroke.clone();

                if self.is_set(BrushMode::Multi) {
                    let mut pixels = Vec::new();

                    for p in stroke {
                        let frame_index = (p.x as u32 / fw) as i32;
                        for i in 0..nframes as i32 - frame_index {
                            pixels.push(
                                p + Vector2::new((i as u32 * fw) as i32, 0),
                            );
                        }
                    }
                    pixels
                } else {
                    stroke
                }
            }
            _ => Vec::new(),
        };

        pixels
            .iter()
            .map(|p| {
                self.shape(
                    Point2::new(p.x as f32, p.y as f32),
                    stroke,
                    fill,
                    scale,
                    origin.clone(),
                )
            })
            .collect()
    }

    /// Return the shape that should be painted when the brush is at the given
    /// position with the given parameters. Takes an `Origin` which describes
    /// whether to align the position to the bottom-left of the shape, or the
    /// center.
    pub fn shape(
        &self,
        p: Point2<f32>,
        stroke: Stroke,
        fill: Fill,
        scale: f32,
        origin: Origin,
    ) -> Shape {
        let x = p.x;
        let y = p.y;

        let size = self.size as f32;

        let offset = match origin {
            Origin::Center => size * scale / 2.,
            Origin::BottomLeft => (self.size / 2) as f32 * scale,
            Origin::TopLeft => unreachable!(),
        };

        Shape::Rectangle(
            Rect::new(x, y, x + size * scale, y + size * scale)
                - Vector2::new(offset, offset),
            stroke,
            fill,
        )
    }

    ///////////////////////////////////////////////////////////////////////////

    /// Draw a line between two points. Uses Bresenham's line algorithm.
    fn line(
        mut p0: Point2<i32>,
        p1: Point2<i32>,
        canvas: &mut Vec<Point2<i32>>,
    ) {
        let dx = i32::abs(p1.x - p0.x);
        let dy = i32::abs(p1.y - p0.y);
        let sx = if p0.x < p1.x { 1 } else { -1 };
        let sy = if p0.y < p1.y { 1 } else { -1 };

        let mut err1 = (if dx > dy { dx } else { -dy }) / 2;
        let mut err2;

        loop {
            canvas.push(p0);

            if p0 == p1 {
                break;
            }

            err2 = err1;

            if err2 > -dx {
                err1 -= dy;
                p0.x += sx;
            }
            if err2 < dy {
                err1 += dx;
                p0.y += sy;
            }
        }
    }

    /// Filter a brush stroke to remove 'L' shapes. This is often called
    /// *pixel perfect* mode.
    fn filter(stroke: &[Point2<i32>]) -> Vec<Point2<i32>> {
        let mut filtered = Vec::with_capacity(stroke.len());

        if stroke.len() <= 2 {
            return stroke.to_owned();
        }

        let mut iter = (0..stroke.len()).into_iter();
        if let Some(i) = iter.next() {
            filtered.push(stroke[i]);
        }
        while let Some(i) = iter.next() {
            let p = stroke[i];

            if let Some(prev) = stroke.get(i - 1) {
                if let Some(next) = stroke.get(i + 1) {
                    if (prev.y == p.y && next.y != p.y && next.x == p.x)
                        || (prev.x == p.x && next.x != p.x && next.y == p.y)
                    {
                        if let Some(i) = iter.next() {
                            filtered.push(stroke[i]);
                        }
                        continue;
                    }
                }
            }
            filtered.push(p);
        }
        filtered
    }
}
