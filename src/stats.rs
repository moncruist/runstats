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
use std::f64::consts::PI;
use std::time::Duration;

use super::TrackPoint;

/// In meters according to WGS84
const EARTH_RADIUS: f64 = 6371008.8;

fn deg2rad(angle: f64) -> f64 {
    angle * PI / 180.0
}

/// Calculates Great-circle distance
/// Vincenty formula from https://en.wikipedia.org/wiki/Great-circle_distance
fn distance(lat1: f64, long1: f64, lat2: f64, long2: f64) -> f64 {
    let sin_lat1 = deg2rad(lat1).sin();
    let cos_lat1 = deg2rad(lat1).cos();
    let sin_lat2 = deg2rad(lat2).sin();
    let cos_lat2 = deg2rad(lat2).cos();
    let sin_delta_long = deg2rad(long2 - long1).sin();
    let cos_delta_long = deg2rad(long2 - long1).cos();

    let a = (cos_lat2 * sin_delta_long).powi(2)
        + (cos_lat1 * sin_lat2 - sin_lat1 * cos_lat2 * cos_delta_long).powi(2);
    let a = a.sqrt();

    let b = sin_lat1 * sin_lat2 + cos_lat1 * cos_lat2 * cos_delta_long;

    let angle = (a / b).atan();

    angle * EARTH_RADIUS
}

pub fn calc_track_distance(points: &Vec<TrackPoint>) -> f64 {
    let mut total_distance = 0.0_f64;

    for point_idx in 0..points.len() {
        let next_idx = point_idx + 1;
        if next_idx == points.len() {
            break;
        }

        let point = &points[point_idx];
        let next_point = &points[next_idx];
        total_distance += distance(
            point.latitude,
            point.longitude,
            next_point.latitude,
            next_point.longitude,
        );
    }
    total_distance
}

pub fn calc_track_duration(points: &[TrackPoint]) -> Duration {
    let mut total_duration = Duration::new(0, 0);

    for point_idx in 0..points.len() {
        let next_idx = point_idx + 1;
        if next_idx == points.len() {
            break;
        }

        let point = &points[point_idx];
        let next_point = &points[next_idx];

        let duration = next_point
            .time
            .signed_duration_since(point.time)
            .to_std()
            .unwrap();
        total_duration += duration;
    }

    total_duration
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, NaiveDateTime, Utc};

    #[test]
    fn test_zero_distance() {
        let dist = distance(0.0, 0.0, 0.0, 0.0);

        assert!((dist - 0.0).abs() <= f64::EPSILON);
    }

    /// Check distance between Moscow and Saint Petersburg
    #[test]
    fn test_distance_between_cities() {
        let dist = distance(55.755826_f64, 37.6173_f64, 59.9342802_f64, 30.3350986_f64);

        assert!((dist - 633016.49).abs() <= 1.0);
    }

    #[test]
    fn test_multipoint_distance() {
        let points = vec![
            TrackPoint::from_coordinates(1.0, 2.0),
            TrackPoint::from_coordinates(1.5, 2.1),
            TrackPoint::from_coordinates(1.8, 2.2),
        ];

        let dist1 = distance(
            points[0].latitude,
            points[0].longitude,
            points[1].latitude,
            points[1].longitude,
        );
        let dist2 = distance(
            points[1].latitude,
            points[1].longitude,
            points[2].latitude,
            points[2].longitude,
        );
        let total = dist1 + dist2;

        assert_eq!(calc_track_distance(&points), total);
    }

    #[test]
    fn test_calc_track_duration_10_points() {
        let mut points: [TrackPoint; 10] = [TrackPoint::new(); 10];

        let step_sec: i64 = 3;
        let offset_sec: i64 = 100;
        let expected_duration_millis = step_sec as u128 * 1000 * (points.len() - 1) as u128;

        for i in 0..points.len() {
            let secs = offset_sec + step_sec * i as i64;
            points[i].time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(secs, 0), Utc);
        }

        assert_eq!(
            calc_track_duration(&points).as_millis(),
            expected_duration_millis
        );
    }

    #[test]
    fn test_calc_track_duration_1_point() {
        let mut points: [TrackPoint; 1] = [TrackPoint::new()];
        points[0].time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(123456, 0), Utc);

        assert_eq!(calc_track_duration(&points).as_millis(), 0);
    }

    #[test]
    fn test_calc_track_duration_2_same_point() {
        let mut points: [TrackPoint; 2] = [TrackPoint::new(); 2];
        points[0].time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(123456, 0), Utc);
        points[1].time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(123456, 0), Utc);

        assert_eq!(calc_track_duration(&points).as_millis(), 0);
    }
}
