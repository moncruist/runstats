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
use std::env;
use std::fs;

use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        eprintln!("Too few arguments");
        process::exit(1);
    }

    let gpx_path = &args[1];
    if !fs::metadata(gpx_path).is_ok() {
        eprintln!("File doesn't exist");
        process::exit(2);
    }

    if let Ok(track) = runstats::read_gpx(gpx_path) {
        println!("Track info:");
        println!("Distance (meters):\t{}", track.distance());
        println!("Duration (seconds):\t{}", track.duration().as_secs());
        println!("Avg heart rate (bpm):\t{}", track.avg_heart_rate());
        
        println!("Splits:");
        let splits = track.splits();
        for i in 0..splits.len() {
            let km = (i as u16 * 1000 + splits[i].distance) as f64 / 1000.0;
            println!("{} km:\t{} secs/km\t{} meters", km, splits[i].pace, splits[i].elevation_delta);
        }
        println!("Elevation:");
        let elevation_stats = track.elevation_stats();
        println!("Max elevation: {}", elevation_stats.max_elevation);
        println!("Min elevation: {}", elevation_stats.min_elevation);
        println!("Elevation gain: {}", elevation_stats.gain);
    } else {
        eprintln!("Parsing error");
    }
}
