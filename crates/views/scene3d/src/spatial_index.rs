use crate::world_scene_vm::{Position3, SceneAnchor};

#[derive(Debug, Clone, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn minus(&self, other: &Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    fn plus(&self, other: &Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    fn scale(&self, amount: f32) -> Self {
        Self::new(self.x * amount, self.y * amount, self.z * amount)
    }

    fn normalized(&self) -> Self {
        let length = self.dot(self).sqrt();

        if length == 0.0 {
            return Self::new(0.0, 0.0, 0.0);
        }

        Self::new(self.x / length, self.y / length, self.z / length)
    }
}

impl From<&Position3> for Vec3 {
    fn from(value: &Position3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpatialRay {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl SpatialRay {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalized(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpatialHit {
    pub selection_id: String,
    pub focus_target: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
struct SpatialNode {
    center: Vec3,
    radius: f32,
    selection_id: String,
    focus_target: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpatialIndex {
    nodes: Vec<SpatialNode>,
}

impl SpatialIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_sphere(
        &mut self,
        center: Vec3,
        radius: f32,
        selection_id: String,
        focus_target: &'static str,
    ) {
        self.nodes.push(SpatialNode {
            center,
            radius,
            selection_id,
            focus_target,
        });
    }

    pub fn insert_anchor(&mut self, anchor: SceneAnchor) {
        if !anchor.pickable {
            return;
        }

        self.insert_sphere(
            Vec3::from(&anchor.position),
            0.75,
            anchor.source_record_id,
            anchor.drill_down_target,
        );
    }

    pub fn first_hit(&self, ray: &SpatialRay) -> Option<SpatialHit> {
        self.nodes
            .iter()
            .filter_map(|node| ray_sphere_distance(ray, node).map(|distance| (distance, node)))
            .min_by(|left, right| left.0.total_cmp(&right.0))
            .map(|(_, node)| SpatialHit {
                selection_id: node.selection_id.clone(),
                focus_target: node.focus_target,
            })
    }
}

fn ray_sphere_distance(ray: &SpatialRay, node: &SpatialNode) -> Option<f32> {
    let offset = ray.origin.minus(&node.center);
    let a = ray.direction.dot(&ray.direction);
    let b = 2.0 * offset.dot(&ray.direction);
    let c = offset.dot(&offset) - node.radius * node.radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return None;
    }

    let sqrt = discriminant.sqrt();
    let near = (-b - sqrt) / (2.0 * a);
    let far = (-b + sqrt) / (2.0 * a);
    let distance = if near >= 0.0 { near } else { far };

    (distance >= 0.0).then_some(distance)
}

#[allow(dead_code)]
fn point_on_ray(ray: &SpatialRay, distance: f32) -> Vec3 {
    ray.origin.plus(&ray.direction.scale(distance))
}
