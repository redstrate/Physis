// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-FileCopyrightText: 2026 OhKannaDuh
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

/// Transformation within the world space.
#[binrw]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
#[allow(dead_code)] // most of the fields are unused at the moment
pub struct Transformation {
    /// X, Y, Z of the location in world space.
    pub translation: [f32; 3],
    /// Yaw, pitch and roll of the rotation in world space.
    pub rotation: [f32; 3],
    /// Width, height and depth of the scale in world space.
    pub scale: [f32; 3],
}

#[cfg(feature = "glam")]
impl From<Transformation> for glam::Affine3A {
    fn from(t: Transformation) -> Self {
        use glam::{Affine3A, EulerRot, Quat, Vec3};

        let translation = Vec3::from(t.translation);
        let scale = Vec3::from(t.scale);

        let rotation = Quat::from_euler(EulerRot::XYZ, t.rotation[0], t.rotation[1], t.rotation[2]);

        Affine3A::from_scale_rotation_translation(scale, rotation, translation)
    }
}

#[cfg(feature = "glam")]
impl From<glam::Affine3A> for Transformation {
    fn from(a: glam::Affine3A) -> Self {
        let (scale, rotation, translation) = a.to_scale_rotation_translation();

        let (x, y, z) = rotation.to_euler(glam::EulerRot::XYZ);

        Transformation {
            translation: translation.into(),
            rotation: [x, y, z],
            scale: scale.into(),
        }
    }
}

#[cfg(all(test, feature = "glam"))]
mod tests {
    use super::*;
    use glam::{Affine3A, EulerRot, Quat, Vec3};

    const EPS: f32 = 1e-5;

    fn approx_eq(a: [f32; 3], b: [f32; 3]) -> bool {
        (a[0] - b[0]).abs() < EPS && (a[1] - b[1]).abs() < EPS && (a[2] - b[2]).abs() < EPS
    }

    #[test]
    /// Test turning a Transformation instance into an Affine3A
    fn test_forward_conversion() {
        let t = Transformation {
            translation: [1.0, 2.0, 3.0],
            rotation: [0.1, 0.2, 0.3],
            scale: [2.0, 2.0, 2.0],
        };

        let affine: Affine3A = t.into();

        let (scale, rotation, translation) = affine.to_scale_rotation_translation();
        let (x, y, z) = rotation.to_euler(EulerRot::XYZ);

        assert!(approx_eq(translation.into(), [1.0, 2.0, 3.0]));
        assert!(approx_eq(scale.into(), [2.0, 2.0, 2.0]));
        assert!((x - 0.1).abs() < EPS);
        assert!((y - 0.2).abs() < EPS);
        assert!((z - 0.3).abs() < EPS);
    }

    #[test]
    /// Test turning a Affine3A instance into a Transformation
    fn test_reverse_conversion() {
        let translation = Vec3::new(4.0, 5.0, 6.0);
        let scale = Vec3::new(1.5, 1.5, 1.5);
        let rotation = Quat::from_euler(EulerRot::XYZ, 0.4, 0.5, 0.6);

        let affine = Affine3A::from_scale_rotation_translation(scale, rotation, translation);
        let t: Transformation = affine.into();

        assert!(approx_eq(t.translation, [4.0, 5.0, 6.0]));
        assert!(approx_eq(t.scale, [1.5, 1.5, 1.5]));

        let (x, y, z) = rotation.to_euler(EulerRot::XYZ);
        assert!((t.rotation[0] - x).abs() < EPS);
        assert!((t.rotation[1] - y).abs() < EPS);
        assert!((t.rotation[2] - z).abs() < EPS);
    }

    #[test]
    /// Test turning a Transformation instance into an Affine3A and back again
    fn test_round_trip() {
        let original = Transformation {
            translation: [7.0, 8.0, 9.0],
            rotation: [0.7, 0.8, 0.9],
            scale: [0.5, 0.5, 0.5],
        };

        let affine: Affine3A = original.into();
        let round_trip: Transformation = affine.into();

        assert!(approx_eq(original.translation, round_trip.translation));
        assert!(approx_eq(original.scale, round_trip.scale));

        assert!((original.rotation[0] - round_trip.rotation[0]).abs() < EPS);
        assert!((original.rotation[1] - round_trip.rotation[1]).abs() < EPS);
        assert!((original.rotation[2] - round_trip.rotation[2]).abs() < EPS);
    }
}
