// Runstats
// Copyright (C) 2020  Konstantin Zhukov
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
mod gpx_parser;
mod stats;

pub use gpx_parser::read_gpx;

use chrono::{DateTime, Utc};
use std::time::Duration;

#[derive(Debug, Copy, Clone)]
pub struct TrackPoint {
    latitude: f64,
    longitude: f64,
    elevation: f64,
    time: DateTime<Utc>,
    heart_rate: u8,
    cadence: u8,
}

impl TrackPoint {
    pub fn new() -> TrackPoint {
        TrackPoint {
            latitude: 0.0,
            longitude: 0.0,
            elevation: 0.0,
            time: Utc::now(),
            heart_rate: 0,
            cadence: 0
        }
    }

    pub fn from_coordinates(latitude: f64, longitude: f64) -> TrackPoint {
        TrackPoint {
            latitude,
            longitude,
            elevation: 0.0,
            time: Utc::now(),
            heart_rate: 0,
            cadence: 0
        }
    }
}

#[derive(Debug)]
pub struct Track {
    name: String,
    start_time: Option<DateTime<Utc>>,
    route: Vec<TrackPoint>,
}

impl Track {
    pub fn new() -> Track {
        Track {
            name: String::new(),
            start_time: None,
            route: Vec::new(),
        }
    }

    pub fn distance(&self) -> u64 {
        let distance = stats::calc_track_distance(&self.route);
        if distance > 0.0 {
            distance as u64
        } else {
            0
        }
    }

    pub fn duration(&self) -> Duration {
        stats::calc_track_duration(&self.route)
    }
}

#[derive(Debug)]
pub enum ParseError {
    XmlError,
}
