use bevy::prelude::*;

pub mod asteroid;
pub mod earth;
pub mod jupiter;
pub mod mars;
pub mod mercury;
pub mod moon;
pub mod neptune;
pub mod pluto;
pub mod saturn;
pub mod sun;
pub mod uranus;
pub mod venus;

pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, _app: &mut App) {}
}
