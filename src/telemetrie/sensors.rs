use bincode::{Decode, Encode};
use serde::Serialize;
use std::fmt;

/// Structure de données issus de la central inertiel
#[derive(Encode, Decode, Clone, Serialize, Copy)]
pub struct IMUData {
    pub status: u8,
    pub ax: f32,
    pub ay: f32,
    pub az: f32,
    pub temp: f32,
}

impl fmt::Display for IMUData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "R: {} P: {} Y: {} T: {}",
            self.ay,
            self.ax,
            self.az,
            self.temp)
    }
}

// Structure de données issus du GPS
#[derive(Encode, Decode, Clone, Serialize, Copy)]
pub struct GPSData {
    pub status: u8,
    pub lat_deg: f32,
    pub lat_min: f32,
    pub dir_lat: u8,
    pub long_deg: f32,
    pub long_min: f32,
    pub dir_long: u8,
    pub decli_mag: f32,
    pub cap_vrai: f32,
    pub cap_mag: f32,
    pub vitesse_sol: f32,
}

impl fmt::Display for GPSData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LAT: {}\" {} LONG: {}\" {}", self.lat_deg, self.lat_min, self.long_deg, self.long_min)
    }
}

/// Structure de données issus du capteur magnétique 3 axes
#[derive(Encode, Decode, Clone, Serialize, Copy)]
pub struct MAGData {
    pub status: u8,
    pub raw_x: f32,
    pub raw_y: f32,
    pub raw_z: f32,
    pub heading: f32,
}

impl fmt::Display for MAGData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Heading: {}", self.heading)
    }
}

/// Structure de données pour les données analogiques
#[derive(Encode, Decode, Clone, Debug, Copy, Serialize)]
pub struct AnalogData {
    pub status: u8,
    pub battery: f32,
}

impl fmt::Display for AnalogData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "B: {}", self.battery)
    }
}

/// Structure final
#[derive(Clone, Encode, Decode, Serialize, Copy)]
pub struct SensorsData {
    imu: IMUData,
    gps: GPSData,
    mag: MAGData,
    analog: AnalogData,
}

//unsafe impl Send for SensorsData {}

impl fmt::Display for SensorsData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IMU: {} GPS: {} MAG: {} Analog: {}", self.imu, self.gps, self.mag, self.analog)
    }
}

pub struct Sensors {}
impl Sensors {
    pub fn empty() -> SensorsData {
        SensorsData {
            imu: IMU::empty(),
            gps: GPS::empty(),
            mag: MAG::empty(),
            analog: Analog::empty(),
        }
    }
}

pub struct MAG {}
impl MAG {
    /// Retourne des données vide
    pub fn empty() -> MAGData {
        MAGData {
            status: 0xFF,
            raw_x: 0.0,
            raw_y: 0.0,
            raw_z: 0.0,            
            heading: 0.0,
        }
    }
}

pub struct IMU {}
impl IMU {
    /// Retourne des données vide
    pub fn empty() -> IMUData {
        IMUData {
            status: 0xFF,
            ax: 0.0,
            ay: 0.0,
            az: 0.0,
            temp: 0.0,
        } 
    }
}

pub struct GPS {}
impl GPS {
    /// Retourne des données vide
    pub fn empty() -> GPSData {
        GPSData {
            status: 0xFF,
            lat_deg: 0.0,
            lat_min: 0.0,
            dir_lat: b'N',
            long_deg: 0.0,
            long_min: 0.0,
            dir_long: b'W',
            decli_mag: 0.0,
            cap_vrai: 0.0,
            cap_mag: 0.0,
            vitesse_sol: 0.0,
        } 
    }
}

pub struct Analog {}
impl Analog {
    /// Retourne des données vide
    pub fn empty() -> AnalogData {
        AnalogData {
            status: 0xFF,
            battery: 0.0,
        } 
    }
}