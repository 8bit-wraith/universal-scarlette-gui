//! Audio routing data structures

use serde::{Deserialize, Serialize};

/// Audio port type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortType {
    /// Analog input
    AnalogIn,
    /// Analog output
    AnalogOut,
    /// S/PDIF input
    SpdifIn,
    /// S/PDIF output
    SpdifOut,
    /// ADAT input
    AdatIn,
    /// ADAT output
    AdatOut,
    /// Mixer output
    MixerOut,
    /// PCM (DAW) input
    PcmIn,
    /// PCM (DAW) output
    PcmOut,
    /// DSP input
    DspIn,
    /// DSP output
    DspOut,
}

/// Audio port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub port_type: PortType,
    pub index: usize,
    pub name: String,
}

/// Routing matrix - maps sources to destinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMatrix {
    /// Available sources
    pub sources: Vec<Port>,
    /// Available destinations
    pub destinations: Vec<Port>,
    /// Current routing: destination index -> source index
    pub routes: Vec<Option<usize>>,
}

impl RoutingMatrix {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            destinations: Vec::new(),
            routes: Vec::new(),
        }
    }

    /// Set a route from source to destination
    pub fn set_route(&mut self, dest_idx: usize, source_idx: Option<usize>) {
        if dest_idx < self.routes.len() {
            self.routes[dest_idx] = source_idx;
        }
    }

    /// Get the source for a destination
    pub fn get_route(&self, dest_idx: usize) -> Option<usize> {
        self.routes.get(dest_idx).copied().flatten()
    }
}

impl Default for RoutingMatrix {
    fn default() -> Self {
        Self::new()
    }
}
