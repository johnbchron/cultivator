use bevy::ecs::component::Component;
use hexx::Hex;

/// A component containing all map-based location data for a hex tile.
/// This is used for all game logic concerning hexes.
///
///
/// `pos` is the hex coordinates of the tile.
///
/// `level` is the level of the tile. This is used for describing the order
/// of stacked hexes.
///
/// `map_index` is the index of the map that the tile belongs to.
#[derive(Component, Hash, PartialEq, Eq, Default)]
pub struct HexPosition {
  pub pos:       Hex,
  pub level:     HexLevel,
  pub map_index: u8,
}

/// An enum that represents the level of a hex tile.
#[derive(Hash, PartialEq, Eq)]
pub enum HexLevel {
  Ground,
}

impl Default for HexLevel {
  fn default() -> Self {
    Self::Ground
  }
}
