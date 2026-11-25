use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

// Right and bottom are exclusive. Desktop rectangles can have negative origins.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn between(a: Point, b: Point) -> Self {
        Self {
            left: a.x.min(b.x),
            top: a.y.min(b.y),
            right: a.x.max(b.x),
            bottom: a.y.max(b.y),
        }
    }

    pub fn contains(self, point: Point) -> bool {
        point.x >= self.left && point.x < self.right && point.y >= self.top && point.y < self.bottom
    }

    pub fn width(self) -> i32 {
        self.right - self.left
    }

    pub fn height(self) -> i32 {
        self.bottom - self.top
    }

    pub fn has_area(self) -> bool {
        self.width() > 0 && self.height() > 0
    }

    pub fn intersection(self, other: Self) -> Self {
        Self {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        }
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }

    pub fn clamp_edge(self, point: Point) -> Point {
        Point {
            x: point.x.clamp(self.left, self.right),
            y: point.y.clamp(self.top, self.bottom),
        }
    }

    pub fn local(self, point: Point) -> Point {
        Point {
            x: point.x - self.left,
            y: point.y - self.top,
        }
    }

    pub fn local_rect(self, rect: Self) -> Self {
        Self {
            left: rect.left - self.left,
            top: rect.top - self.top,
            right: rect.right - self.left,
            bottom: rect.bottom - self.top,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse_drag_across_a_negative_origin_uses_physical_pixels() {
        let desktop = Rect {
            left: -1920,
            top: -400,
            right: 2560,
            bottom: 1440,
        };
        let a = desktop.local(Point { x: 100, y: 80 });
        let b = desktop.local(Point { x: -100, y: -20 });
        let crop = Rect::between(a, b);
        assert_eq!(
            (crop.left, crop.top, crop.width(), crop.height()),
            (1820, 380, 200, 100)
        );
    }

    #[test]
    fn crop_intersection_rejects_empty_selections_and_clips_outside_pixels() {
        let frame = Rect {
            left: 0,
            top: 0,
            right: 100,
            bottom: 100,
        };
        assert!(!frame.intersection(Rect::default()).has_area());
        assert_eq!(
            frame.intersection(Rect {
                left: -5,
                top: 10,
                right: 110,
                bottom: 120
            }),
            Rect {
                left: 0,
                top: 10,
                right: 100,
                bottom: 100
            }
        );
        assert_eq!(
            frame.clamp_edge(Point { x: 130, y: -5 }),
            Point { x: 100, y: 0 }
        );
    }
}
