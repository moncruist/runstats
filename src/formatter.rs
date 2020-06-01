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
use std::time::Duration;

pub fn format_time(time: u64) -> String {
    let seconds = time % 60;
    let minutes = (time / 60) % 60;
    let hours = (time / 3600) % 24;
    let days = time / (3600 * 24);

    if days > 0 {
        format!("{}d {}:{}:{}", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}:{}:{}", hours, minutes, seconds)
    } else {
        format!("{}:{}", minutes, seconds)
    }
}

pub fn format_duration(duration: Duration) -> String {
    format_time(duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time()
    {
        let time: u64 = 35;
        assert_eq!(format_time(time), "0:35");

        let time: u64 = time + 60 * 23;
        assert_eq!(format_time(time), "23:35");

        let time: u64 = time + 3600 * 11;
        assert_eq!(format_time(time), "11:23:35");

        let time: u64 = time + 60 * 60 * 24 * 4;
        assert_eq!(format_time(time), "4d 11:23:35");
    }

    #[test]
    fn test_format_duration()
    {
        let duration = Duration::from_secs(3 * 24 * 60 * 60 + 5 * 60 * 60 + 4 * 60 + 15);
        assert_eq!(format_duration(duration), "3d 5:4:15");
    }
}