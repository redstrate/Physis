// SPDX-FileCopyrightText: 2020 Inseok Lee
// SPDX-License-Identifier: MIT

use crate::havok::transform::HavokTransform;

pub trait HavokAnimation {
    fn duration(&self) -> f32;
    fn sample(&self, time: f32) -> Vec<HavokTransform>;
}
