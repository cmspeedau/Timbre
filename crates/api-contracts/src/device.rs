//! Output device and zone contracts.

use serde::{Deserialize, Serialize};
use timbre_shared_types::{DeviceId, ZoneId};

use crate::format::AudioFormat;

/// A capability an output device advertises.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceCapability {
    /// Accepts raw PCM streams over the network.
    NetworkStreaming,
    /// Can render bit-perfect output in an exclusive mode.
    BitPerfectExclusive,
    /// Reports a volume control.
    VolumeControl,
    /// Can be synchronized with other devices in a group.
    Synchronizable,
}

/// What an output device can do.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// Advertised capabilities.
    pub capabilities: Vec<DeviceCapability>,
    /// Formats the device accepts natively, without conversion.
    pub supported_formats: Vec<AudioFormat>,
}

/// An output device.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Device {
    /// Identity.
    pub id: DeviceId,
    /// Display name.
    pub name: String,
    /// Advertised capabilities.
    pub capabilities: DeviceCapabilities,
    /// Whether the device is currently reachable.
    pub online: bool,
}

/// A named playback destination, backed by one or more devices.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zone {
    /// Identity.
    pub id: ZoneId,
    /// Display name.
    pub name: String,
    /// The devices that render this zone.
    pub devices: Vec<DeviceId>,
    /// Whether the devices are grouped for synchronized playback.
    pub grouped: bool,
}
