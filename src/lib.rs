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
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
    pub time: DateTime<Utc>,
    pub heart_rate: u8,
    pub cadence: u8,
}

impl TrackPoint {
    pub fn new() -> TrackPoint {
        TrackPoint {
            latitude: 0.0,
            longitude: 0.0,
            elevation: 0.0,
            time: Utc::now(),
            heart_rate: 0,
            cadence: 0,
        }
    }

    pub fn from_coordinates(latitude: f64, longitude: f64) -> TrackPoint {
        TrackPoint {
            latitude,
            longitude,
            elevation: 0.0,
            time: Utc::now(),
            heart_rate: 0,
            cadence: 0,
        }
    }
}

#[derive(Debug)]
pub struct TrackSegment {
    pub points: Vec<TrackPoint>,
}

impl TrackSegment {
    pub fn new() ->TrackSegment {
        TrackSegment { points: Vec::new() }
    }
}

#[derive(Debug)]
pub struct Track {
    pub name: String,
    pub creation_time: Option<DateTime<Utc>>,
    pub route: Vec<TrackSegment>,
}

impl Track {
    pub fn new() -> Track {
        Track {
            name: String::new(),
            creation_time: None,
            route: Vec::new(),
        }
    }

    pub fn distance(&self) -> u64 {
        stats::calc_track_distance(self)
    }

    pub fn duration(&self) -> Duration {
        stats::calc_track_duration(self)
    }

    pub fn avg_heart_rate(&self) -> u8 {
        stats::calc_track_average_heart_rate(&self)
    }
}

#[derive(Debug)]
pub enum ParseError {
    XmlError,
}
