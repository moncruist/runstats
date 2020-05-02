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
        println!("Track {:?}", track);
    } else {
        eprintln!("Parsing error");
    }
}
