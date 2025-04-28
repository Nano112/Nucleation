use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: (i32, i32, i32),
    pub max: (i32, i32, i32),
}

impl BoundingBox {
    pub fn new(min: (i32, i32, i32), max: (i32, i32, i32)) -> Self {
        BoundingBox { min, max }
    }

    pub fn contains(&self, point: (i32, i32, i32)) -> bool {
        point.0 >= self.min.0 && point.0 <= self.max.0 &&
            point.1 >= self.min.1 && point.1 <= self.max.1 &&
            point.2 >= self.min.2 && point.2 <= self.max.2
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.0 <= other.max.0 && self.max.0 >= other.min.0 &&
            self.min.1 <= other.max.1 && self.max.1 >= other.min.1 &&
            self.min.2 <= other.max.2 && self.max.2 >= other.min.2
    }

    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            min: (
                self.min.0.min(other.min.0),
                self.min.1.min(other.min.1),
                self.min.2.min(other.min.2),
            ),
            max: (
                self.max.0.max(other.max.0),
                self.max.1.max(other.max.1),
                self.max.2.max(other.max.2),
            ),
        }
    }

    pub fn coords_to_index(&self, x: i32, y: i32, z: i32) -> usize {
        let (width, _, length) = self.get_dimensions();
        let dx = x - self.min.0;
        let dy = y - self.min.1;
        let dz = z - self.min.2;
        (dx + dz * width + dy * width * length) as usize
    }

    pub fn index_to_coords(&self, index: usize) -> (i32, i32, i32) {
        let (width, _, length) = self.get_dimensions();
        let dx = (index % width as usize) as i32;
        let dy = (index / (width * length) as usize) as i32;
        let dz = ((index / width as usize) % length as usize) as i32;
        (dx + self.min.0, dy + self.min.1, dz + self.min.2)
    }

    pub fn get_dimensions(&self) -> (i32, i32, i32) {
        (
            (self.max.0 - self.min.0 + 1),
            (self.max.1 - self.min.1 + 1),
            (self.max.2 - self.min.2 + 1),
        )
    }

    pub fn to_position_and_size(&self) -> ((i32, i32, i32), (i32, i32, i32)) {
        (self.min, self.get_dimensions())
    }

    pub fn from_position_and_size(position: (i32, i32, i32), size: (i32, i32, i32)) -> Self {
        let position2 = (position.0 + size.0, position.1 + size.1, position.2 + size.2);

        let offset_min = (
            -size.0.signum().min(0),
            -size.1.signum().min(0),
            -size.2.signum().min(0),
        );
        let offset_max = (
            -size.0.signum().max(0),
            -size.1.signum().max(0),
            -size.2.signum().max(0),
        );

        BoundingBox::new(
            (position.0.min(position2.0) + offset_min.0, position.1.min(position2.1) + offset_min.1, position.2.min(position2.2) + offset_min.2),
            (position.0.max(position2.0) + offset_max.0, position.1.max(position2.1) + offset_max.1, position.2.max(position2.2) + offset_max.2),
        )
    }

    pub fn volume(&self) -> u64 {
        let (width, height, length) = self.get_dimensions();
        width as u64 * height as u64 * length as u64
    }

    /// Returns an iterator over all coordinates in this bounding box.
    /// Iterates in x, z, y order for cache efficiency.
    pub fn iter_coords(&self) -> BoundingBoxIterator {
        BoundingBoxIterator {
            bbox: self.clone(),
            current: Some((self.min.0, self.min.1, self.min.2)),
        }
    }
}

/// Iterator for all coordinates in a bounding box
pub struct BoundingBoxIterator {
    bbox: BoundingBox,
    current: Option<(i32, i32, i32)>,
}

impl Iterator for BoundingBoxIterator {
    type Item = (i32, i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;

        // Calculate the next coordinate
        let mut next_x = current.0 + 1;
        let mut next_y = current.1;
        let mut next_z = current.2;

        // If we've reached the end of the x row, move to the next z
        if next_x > self.bbox.max.0 {
            next_x = self.bbox.min.0;
            next_z += 1;

            // If we've reached the end of the z plane, move to the next y
            if next_z > self.bbox.max.2 {
                next_z = self.bbox.min.2;
                next_y += 1;

                // If we've gone beyond the max y, we're done
                if next_y > self.bbox.max.1 {
                    self.current = None;
                    return Some(current);
                }
            }
        }

        self.current = Some((next_x, next_y, next_z));
        Some(current)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let volume = self.bbox.volume() as usize;

        if let Some(current) = self.current {
            // Calculate how many positions we've already visited
            let dx = current.0 - self.bbox.min.0;
            let dy = current.1 - self.bbox.min.1;
            let dz = current.2 - self.bbox.min.2;

            let (width, _, length) = self.bbox.get_dimensions();
            let visited = (dx + dz * width + dy * width * length) as usize;

            let remaining = volume - visited;
            (remaining, Some(remaining))
        } else {
            (0, Some(0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box_creation() {
        let bb = BoundingBox::new((0, 0, 0), (2, 2, 2));
        assert_eq!(bb.min, (0, 0, 0));
        assert_eq!(bb.max, (2, 2, 2));
    }

    #[test]
    fn test_contains() {
        let bb = BoundingBox::new((0, 0, 0), (2, 2, 2));
        assert!(bb.contains((0, 0, 0)));
        assert!(bb.contains((1, 1, 1)));
        assert!(bb.contains((2, 2, 2)));
        assert!(!bb.contains((-1, 0, 0)));
        assert!(!bb.contains((3, 0, 0)));
    }

    #[test]
    fn test_intersects() {
        let bb1 = BoundingBox::new((0, 0, 0), (2, 2, 2));
        let bb2 = BoundingBox::new((1, 1, 1), (3, 3, 3));
        let bb3 = BoundingBox::new((3, 3, 3), (4, 4, 4));

        assert!(bb1.intersects(&bb2));
        assert!(bb2.intersects(&bb1));
        assert!(bb2.intersects(&bb3));
        assert!(!bb1.intersects(&bb3));
    }

    #[test]
    fn test_union() {
        let bb1 = BoundingBox::new((0, 0, 0), (2, 2, 2));
        let bb2 = BoundingBox::new((1, 1, 1), (3, 3, 3));

        let union = bb1.union(&bb2);
        assert_eq!(union.min, (0, 0, 0));
        assert_eq!(union.max, (3, 3, 3));
    }

    #[test]
    fn test_coords_to_index() {
        let bb = BoundingBox::new((0, 0, 0), (2, 2, 2));

        assert_eq!(bb.coords_to_index(0, 0, 0), 0);
        assert_eq!(bb.coords_to_index(1, 0, 0), 1);
        assert_eq!(bb.coords_to_index(0, 0, 1), 3);
        assert_eq!(bb.coords_to_index(0, 1, 0), 9);
    }

    #[test]
    fn test_index_to_coords() {
        let bb = BoundingBox::new((0, 0, 0), (2, 2, 2));

        assert_eq!(bb.index_to_coords(0), (0, 0, 0));
        assert_eq!(bb.index_to_coords(1), (1, 0, 0));
        assert_eq!(bb.index_to_coords(3), (0, 0, 1));
        assert_eq!(bb.index_to_coords(9), (0, 1, 0));
    }

    #[test]
    fn test_get_dimensions() {
        let bb = BoundingBox::new((0, 0, 0), (2, 2, 2));
        assert_eq!(bb.get_dimensions(), (3, 3, 3));

        let bb = BoundingBox::new((-1, -1, -1), (1, 1, 1));
        assert_eq!(bb.get_dimensions(), (3, 3, 3));
    }

    #[test]
    fn test_volume() {
        let bb = BoundingBox::new((0, 0, 0), (2, 2, 2));
        assert_eq!(bb.volume(), 27);

        let bb = BoundingBox::new((-1, -1, -1), (1, 1, 1));
        assert_eq!(bb.volume(), 27);
    }

    #[test]
    fn test_iter_coords() {
        let bb = BoundingBox::new((0, 0, 0), (1, 1, 1));
        let coords: Vec<_> = bb.iter_coords().collect();

        assert_eq!(coords.len(), 8);
        assert!(coords.contains(&(0, 0, 0)));
        assert!(coords.contains(&(1, 0, 0)));
        assert!(coords.contains(&(0, 0, 1)));
        assert!(coords.contains(&(1, 0, 1)));
        assert!(coords.contains(&(0, 1, 0)));
        assert!(coords.contains(&(1, 1, 0)));
        assert!(coords.contains(&(0, 1, 1)));
        assert!(coords.contains(&(1, 1, 1)));
    }

    #[test]
    fn test_iter_coords_order() {
        let bb = BoundingBox::new((0, 0, 0), (1, 1, 1));
        let coords: Vec<_> = bb.iter_coords().collect();

        // The expected traversal order: x varies fastest, then z, then y
        let expected = vec![
            (0, 0, 0), (1, 0, 0),
            (0, 0, 1), (1, 0, 1),
            (0, 1, 0), (1, 1, 0),
            (0, 1, 1), (1, 1, 1),
        ];

        assert_eq!(coords, expected);
    }

}