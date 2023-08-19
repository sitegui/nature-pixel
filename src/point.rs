use ndarray::{Ix2, NdIndex};
use std::fmt::Debug;
use std::ops::{Add, Mul, Neg, Range, Sub};

/// Defines a point that may or may not be inside the map space
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Point {
    pub x: isize,
    pub y: isize,
}

#[derive(Debug, Clone)]
pub struct CircleIter {
    x_range: Range<isize>,
    center: Point,
    radius: isize,
    map_size: isize,
    current_x: isize,
    current_y_range: Range<isize>,
}

#[derive(Debug, Clone)]
pub struct CircumferenceIter {
    x_range: Range<isize>,
    center: Point,
    radius: isize,
    map_size: isize,
    /// The next point at the current `x`
    next_point: Option<Point>,
}

impl Point {
    pub const X: Point = Point { x: 1, y: 0 };
    pub const Y: Point = Point { x: 0, y: 1 };
    pub const DIRECTIONS: [Point; 4] = [
        Point { x: 1, y: 0 },
        Point { x: 0, y: 1 },
        Point { x: -1, y: 0 },
        Point { x: 0, y: -1 },
    ];

    pub fn new<X, Y>(x: X, y: Y) -> Self
    where
        X: TryInto<isize>,
        <X as TryInto<isize>>::Error: Debug,
        Y: TryInto<isize>,
        <Y as TryInto<isize>>::Error: Debug,
    {
        Self {
            x: x.try_into().unwrap(),
            y: y.try_into().unwrap(),
        }
    }

    pub fn new_ij((i, j): (usize, usize)) -> Self {
        Point::new(j, i)
    }

    pub fn is_valid(self, map_size: usize) -> bool {
        Self::is_in_valid_range(self.x, map_size as isize)
            && Self::is_in_valid_range(self.y, map_size as isize)
    }

    /// Return the points up to distance 1 from this one. The points are not necessary valid
    pub fn surroundings(self) -> [Point; 5] {
        [
            Point::new(self.x - 1, self.y),
            Point::new(self.x, self.y - 1),
            Point::new(self.x, self.y),
            Point::new(self.x, self.y + 1),
            Point::new(self.x + 1, self.y),
        ]
    }

    /// Iterate over all valid points as far as `radius` from this point. The distance is measured
    /// in taxicab geometry, that is: `abs(delta_x) + abs(delta_y)`.
    ///
    /// The points are not returned in any specific order.
    pub fn circle(self, radius: usize, map_size: usize) -> CircleIter {
        let radius = radius as isize;
        let map_size = map_size as isize;

        CircleIter {
            x_range: Self::valid_range(self.x, radius, map_size),
            center: self,
            radius,
            map_size,
            current_x: isize::MIN,
            current_y_range: 0..0,
        }
    }

    /// Iterate over all valid points at `radius` from this point. The distance is measured
    /// in taxicab geometry, that is: `abs(delta_x) + abs(delta_y)`.
    ///
    /// The points are not returned in any specific order.
    pub fn circumference(self, radius: usize, map_size: usize) -> CircumferenceIter {
        let radius = radius as isize;
        let map_size = map_size as isize;

        CircumferenceIter {
            x_range: Self::valid_range(self.x, radius, map_size),
            center: self,
            radius,
            map_size,
            next_point: None,
        }
    }

    pub fn distance(self, another: Self) -> usize {
        let delta_x = self.x.abs_diff(another.x);
        let delta_y = self.y.abs_diff(another.y);
        delta_x + delta_y
    }

    pub fn turn_right(self) -> Self {
        Point {
            x: -self.y,
            y: self.x,
        }
    }

    pub fn turn_left(self) -> Self {
        Point {
            x: self.y,
            y: -self.x,
        }
    }

    pub fn turn_over(self) -> Self {
        Point {
            x: -self.x,
            y: -self.y,
        }
    }

    fn valid_range(center: isize, radius: isize, map_size: isize) -> Range<isize> {
        let start = (center - radius).max(0);
        let end = (center + radius + 1).min(map_size);
        start..end
    }

    fn is_in_valid_range(x_or_y: isize, map_size: isize) -> bool {
        x_or_y >= 0 && x_or_y < map_size
    }
}

unsafe impl NdIndex<Ix2> for Point {
    fn index_checked(&self, dim: &Ix2, strides: &Ix2) -> Option<isize> {
        let i = usize::try_from(self.y).ok()?;
        let j = usize::try_from(self.x).ok()?;
        (i, j).index_checked(dim, strides)
    }

    fn index_unchecked(&self, strides: &Ix2) -> isize {
        (self.y as usize, self.x as usize).index_unchecked(strides)
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<usize> for Point {
    type Output = Point;

    fn mul(self, rhs: usize) -> Self::Output {
        Point {
            x: self.x * rhs as isize,
            y: self.y * rhs as isize,
        }
    }
}

impl Mul<isize> for Point {
    type Output = Point;

    fn mul(self, rhs: isize) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Neg for Point {
    type Output = Point;

    fn neg(self) -> Self::Output {
        Point {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Iterator for CircleIter {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(y) = self.current_y_range.next() {
                return Some(Point::new(self.current_x, y));
            }

            self.current_x = self.x_range.next()?;
            let delta_x = (self.center.x - self.current_x).abs();
            let delta_y = self.radius - delta_x;
            self.current_y_range = Point::valid_range(self.center.y, delta_y, self.map_size);
        }
    }
}

impl Iterator for CircumferenceIter {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(point) = self.next_point.take() {
                return Some(point);
            }

            let x = self.x_range.next()?;
            let delta_x = (self.center.x - x).abs();
            let delta_y = self.radius - delta_x;

            if delta_y > 0 {
                let y = self.center.y - delta_y;
                if Point::is_in_valid_range(y, self.map_size) {
                    self.next_point = Some(Point::new(x, y));
                }
            }

            let y = self.center.y + delta_y;
            if Point::is_in_valid_range(y, self.map_size) {
                return Some(Point::new(x, y));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn circle() {
        let map_size = 100;

        let check = |center: Point, radius: usize, expected: &[[isize; 2]]| {
            let points: HashSet<_> = center
                .circle(radius, map_size)
                .map(|point| [point.x, point.y])
                .collect();

            let expected: HashSet<_> = expected.iter().copied().collect();
            assert_eq!(points, expected);
        };

        check(Point::new(10, 20), 0, &[[10, 20]]);
        check(
            Point::new(10, 20),
            1,
            &[[9, 20], [10, 19], [10, 20], [10, 21], [11, 20]],
        );
        check(
            Point::new(10, 20),
            2,
            &[
                [8, 20],
                [9, 19],
                [9, 20],
                [9, 21],
                [10, 18],
                [10, 19],
                [10, 20],
                [10, 21],
                [10, 22],
                [11, 19],
                [11, 20],
                [11, 21],
                [12, 20],
            ],
        );

        check(
            Point::new(1, 20),
            2,
            &[
                [0, 19],
                [0, 20],
                [0, 21],
                [1, 18],
                [1, 19],
                [1, 20],
                [1, 21],
                [1, 22],
                [2, 19],
                [2, 20],
                [2, 21],
                [3, 20],
            ],
        );

        check(
            Point::new(0, 20),
            2,
            &[
                [0, 18],
                [0, 19],
                [0, 20],
                [0, 21],
                [0, 22],
                [1, 19],
                [1, 20],
                [1, 21],
                [2, 20],
            ],
        );

        check(
            Point::new(98, 20),
            2,
            &[
                [96, 20],
                [97, 19],
                [97, 20],
                [97, 21],
                [98, 18],
                [98, 19],
                [98, 20],
                [98, 21],
                [98, 22],
                [99, 19],
                [99, 20],
                [99, 21],
            ],
        );

        check(
            Point::new(99, 20),
            2,
            &[
                [97, 20],
                [98, 19],
                [98, 20],
                [98, 21],
                [99, 18],
                [99, 19],
                [99, 20],
                [99, 21],
                [99, 22],
            ],
        );
    }

    #[test]
    fn circumference() {
        let map_size = 100;

        let check = |center: Point, radius: usize, expected: &[[isize; 2]]| {
            let points: HashSet<_> = center
                .circumference(radius, map_size)
                .map(|point| [point.x, point.y])
                .collect();

            let expected: HashSet<_> = expected.iter().copied().collect();
            assert_eq!(points, expected);
        };

        check(Point::new(10, 20), 0, &[[10, 20]]);
        check(
            Point::new(10, 20),
            1,
            &[[9, 20], [11, 20], [10, 21], [10, 19]],
        );
        check(
            Point::new(10, 20),
            2,
            &[
                [8, 20],
                [12, 20],
                [9, 21],
                [9, 19],
                [10, 22],
                [10, 18],
                [11, 21],
                [11, 19],
            ],
        );

        check(
            Point::new(1, 20),
            2,
            &[
                [3, 20],
                [0, 21],
                [0, 19],
                [1, 22],
                [1, 18],
                [2, 21],
                [2, 19],
            ],
        );

        check(
            Point::new(0, 20),
            2,
            &[[2, 20], [0, 22], [0, 18], [1, 21], [1, 19]],
        );

        check(
            Point::new(98, 20),
            2,
            &[
                [96, 20],
                [97, 21],
                [97, 19],
                [98, 22],
                [98, 18],
                [99, 21],
                [99, 19],
            ],
        );

        check(
            Point::new(99, 20),
            2,
            &[[97, 20], [98, 21], [98, 19], [99, 22], [99, 18]],
        );
    }
}
