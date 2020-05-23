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

use super::{TrackPoint, Track};

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

/// Calculates distance taking into account elevations of two points
fn distance_with_elevation(dist: f64, elevation1: f64, elevation2: f64) -> f64 {
    let cathet1 = dist;
    let cathet2 = (elevation2 - elevation1).abs();

    let hypot = (cathet1 * cathet1 + cathet2 * cathet2).sqrt();
    hypot 
}

fn calc_track_distance_segment(points: &[TrackPoint]) -> f64 {
    let mut total_distance = 0.0_f64;

    for point_idx in 0..points.len() {
        let next_idx = point_idx + 1;
        if next_idx >= points.len() {
            break;
        }

        let point = &points[point_idx];
        let next_point = &points[next_idx];
        let horizontal_dist = distance(
            point.latitude,
            point.longitude,
            next_point.latitude,
            next_point.longitude,
        );
        total_distance += distance_with_elevation(horizontal_dist, point.elevation, next_point.elevation);
    }
    total_distance
}

pub fn calc_track_distance(track: &Track) -> u64 {
    let mut distance = 0.0;
    for segment in &track.route {
        distance += calc_track_distance_segment(&segment.points);
    }

    if distance > 0.0 {
        distance as u64
    } else {
        0
    }
}

fn duration_between_points(point1: &TrackPoint, point2: &TrackPoint) -> Duration {
    point2.time.signed_duration_since(point1.time).to_std().unwrap()
}

fn calc_track_duration_segment(points: &[TrackPoint]) -> Duration {
    if points.len() == 0 {
        return Duration::new(0, 0);
    }

    let mut total_duration = Duration::new(0, 0);

    for point_idx in 0..points.len() {
        let next_idx = point_idx + 1;
        if next_idx >= points.len() {
            break;
        }

        let point = &points[point_idx];
        let next_point = &points[next_idx];

        let duration = duration_between_points(point, next_point);
        total_duration += duration;
    }

    total_duration
}

pub fn calc_track_duration(track: &Track) -> Duration {
    let mut total_duration = Duration::new(0, 0);

    for segment in &track.route {
        total_duration += calc_track_duration_segment(&segment.points);
    }

    total_duration
}

pub fn calc_track_average_heart_rate(track: &Track) -> u8 {
    let mut total_duration_sec: u64 = 0;
    let mut sum: u64 = 0;

    for segment in &track.route {
        let mut single_point_segment = true;

        for i in 0..segment.points.len() {
            let point = &segment.points[i];
            if point.heart_rate == 0 {
                continue; // Skip invalid data
            }

            let next_idx = i + 1;
            if next_idx >= segment.points.len() {
                if single_point_segment {
                    // Count as one value for 1 seconds
                    sum += point.heart_rate as u64;
                    total_duration_sec += 1;
                }

                break;
            } 

            single_point_segment = false;
            let next_point = &segment.points[i + 1];

            if next_point.heart_rate == 0 {
                // Current point has HR, next one doesn't. Count as single HR value for 1 second
                sum += point.heart_rate as u64;
                total_duration_sec += 1;
            }

            // Both points has HR values. Use linear approximation for the values in between.
            let duration_sec = duration_between_points(point, next_point).as_secs();
            if duration_sec == 0 {
                continue;
            }

            let s = (point.heart_rate as u64 + next_point.heart_rate as u64) * duration_sec / 2;
            
            sum += s;
            total_duration_sec += duration_sec;
        }
    }

    if total_duration_sec != 0 {
        (sum / total_duration_sec) as u8
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, NaiveDateTime, Utc};
    use super::super::TrackSegment;

    fn new_date_time(seconds: i64) -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(seconds, 0), Utc)
    }

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
        let total = (dist1 + dist2) as u64;

        let mut segment = TrackSegment::new();
        segment.points = points;

        let mut track = Track::new();
        track.route.push(segment);

        assert_eq!(calc_track_distance(&track), total);
    }

    #[test]
    fn test_calc_track_distance_with_elevation() {
        let point1 = TrackPoint::from_coordinates(1.0, 1.0);
        let mut point2 = TrackPoint::from_coordinates(2.0, 1.0);

        // Make elevation of second point be equal to horizontal distance.
        // Thus, real distance should be `sqrt(2.0) * distance`
        let dist = distance(point1.latitude, point1.longitude, point2.latitude, point2.longitude);
        point2.elevation = dist;

        let mut segment = TrackSegment::new();
        segment.points.push(point1);
        segment.points.push(point2);

        let mut track = Track::new();
        track.route.push(segment);

        let expected_dist = (2.0_f64.sqrt() * dist) as u64;
        assert_eq!(calc_track_distance(&track), expected_dist);
    }

    #[test]
    fn test_calc_track_duration_10_points() {
        const POINTS_NUM: usize = 10;
        let mut points = Vec::with_capacity(POINTS_NUM);

        let step_sec: i64 = 3;
        let offset_sec: i64 = 100;
        let expected_duration_millis = step_sec as u128 * 1000 * (POINTS_NUM - 1) as u128;

        for i in 0..POINTS_NUM {
            let secs = offset_sec + step_sec * i as i64;
            let mut point = TrackPoint::new();
            point.time = new_date_time(secs);
            points.push(point);
        }

        let mut segment = TrackSegment::new();
        segment.points = points;

        let mut track = Track::new();
        track.route.push(segment);

        assert_eq!(
            calc_track_duration(&track).as_millis(),
            expected_duration_millis
        );
    }

    #[test]
    fn test_calc_track_duration_1_point() {
        let mut point = TrackPoint::new();
        point.time = new_date_time(123456);

        let mut segment = TrackSegment::new();
        segment.points.push(point);

        let mut track = Track::new();
        track.route.push(segment);

        assert_eq!(calc_track_duration(&track).as_millis(), 0);
    }

    #[test]
    fn test_calc_track_duration_2_same_point() {
        let mut points = Vec::with_capacity(2);[TrackPoint::new(); 2];
        let mut point = TrackPoint::new();
        point.time = new_date_time(123456);
        points.push(point);

        let mut point = TrackPoint::new();
        point.time = new_date_time(123456);
        points.push(point);

        let mut segment = TrackSegment::new();
        segment.points = points;

        let mut track = Track::new();
        track.route.push(segment);

        assert_eq!(calc_track_duration(&track).as_millis(), 0);
    }

    fn new_track_point_hr(seconds: i64, heart_rate: u8) -> TrackPoint {
        let mut point = TrackPoint::new();
        point.time = new_date_time(seconds);
        point.heart_rate = heart_rate;
        point
    }

    #[test]
    fn test_calc_track_average_heartrate() {
        let mut track = Track::new();

        let mut segment = TrackSegment::new();
        segment.points.push(new_track_point_hr(100, 100));
        segment.points.push(new_track_point_hr(110, 110));

        track.route.push(segment);

        let avg_heart_rate = calc_track_average_heart_rate(&track);
        assert_eq!(avg_heart_rate, 105);
    }

    #[test]
    fn test_calc_track_average_heartrate_multi_segment() {
        let mut track = Track::new();

        let mut segment = TrackSegment::new();
        segment.points.push(new_track_point_hr(100, 100));
        segment.points.push(new_track_point_hr(110, 110));

        track.route.push(segment);

        let mut segment = TrackSegment::new();
        segment.points.push(new_track_point_hr(120, 120));
        segment.points.push(new_track_point_hr(130, 130));
        track.route.push(segment);

        let avg_heart_rate = calc_track_average_heart_rate(&track);
        assert_eq!(avg_heart_rate, 115);
    }
}
