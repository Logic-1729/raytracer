use std::sync::Arc;
use crate::aabb::Aabb;
use crate::hittable::{Hittable, HitRecord};
use crate::interval::Interval;
use crate::ray::Ray;
use crate::rtweekend::random::random_int;

pub struct BvhNode {
    left: Arc<dyn Hittable + Send + Sync>,
    right: Arc<dyn Hittable + Send + Sync>,
    bbox: Aabb,
}

impl BvhNode {
    pub fn new_from_list(objects: &mut [Arc<dyn Hittable + Send + Sync>]) -> Self {
        Self::new(objects, 0, objects.len())
    }

    pub fn new(objects: &mut [Arc<dyn Hittable + Send + Sync>], start: usize, end: usize) -> Self {
        let mut bbox = Aabb::new_empty();
        for object_index in start..end {
            bbox = Aabb::surrounding_box(&bbox, &objects[object_index].bounding_box());
        }
        let axis = bbox.longest_axis();
        let comparator = match axis {
            0 => box_x_compare,
            1 => box_y_compare,
            _ => box_z_compare,
        };

        let object_span = end - start;

        let (left, right): (Arc<dyn Hittable + Send + Sync>, Arc<dyn Hittable + Send + Sync>) = if object_span == 1 {
            (objects[start].clone(), objects[start].clone())
        } else if object_span == 2 {
            (objects[start].clone(), objects[start + 1].clone())
        } else {
            objects[start..end].sort_by(|a, b| comparator(a, b));
            let mid = start + object_span / 2;
            (
                Arc::new(BvhNode::new(objects, start, mid)),
                Arc::new(BvhNode::new(objects, mid, end)),
            )
        };

        Self { left, right, bbox }
    }
}

impl Hittable for BvhNode {
    fn hit(
        &self,
        r: &Ray,
        ray_t: Interval,
        rec: &mut HitRecord,
    ) -> bool {
        if !self.bbox.hit(r, ray_t) {
            return false;
        }

        let mut hit_left = false;
        let mut temp_rec = HitRecord::default();

        hit_left = self.left.hit(r, ray_t, &mut temp_rec);
        let mut hit_anything = hit_left;
        let mut closest = if hit_left { temp_rec.t } else { ray_t.max };

        if self.right.hit(r, Interval::new(ray_t.min, closest), &mut temp_rec) {
            hit_anything = true;
            closest = temp_rec.t;
        }

        if hit_anything {
            *rec = temp_rec;
        }

        hit_anything
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}

// 比较函数
fn box_compare(
    a: &Arc<dyn Hittable + Send + Sync>,
    b: &Arc<dyn Hittable + Send + Sync>,
    axis_index: usize,
) -> std::cmp::Ordering {
    let a_bbox = a.bounding_box();
    let b_bbox = b.bounding_box();
    let a_axis = a_bbox.axis_interval(axis_index);
    let b_axis = b_bbox.axis_interval(axis_index);
    a_axis.min.partial_cmp(&b_axis.min).unwrap()
}

fn box_x_compare(
    a: &Arc<dyn Hittable + Send + Sync>,
    b: &Arc<dyn Hittable + Send + Sync>,
) -> std::cmp::Ordering {
    box_compare(a, b, 0)
}

fn box_y_compare(
    a: &Arc<dyn Hittable + Send + Sync>,
    b: &Arc<dyn Hittable + Send + Sync>,
) -> std::cmp::Ordering {
    box_compare(a, b, 1)
}

fn box_z_compare(
    a: &Arc<dyn Hittable + Send + Sync>,
    b: &Arc<dyn Hittable + Send + Sync>,
) -> std::cmp::Ordering {
    box_compare(a, b, 2)
}