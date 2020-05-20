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
use std::fs::File;
use std::io::{BufReader, Read};

use xml::attribute::OwnedAttribute;
use xml::name::OwnedName;
use xml::reader::XmlEvent;
use xml::EventReader;

use chrono::prelude::*;

use super::{ParseError, Track, TrackPoint};

#[derive(Debug, Copy, Clone, PartialEq)]
enum GpxXmlTag {
    Gpx,
    Metadata,
    Track,
    Name,
    TrackSegment,
    TrackPoint,
    Elevation,
    Time,
    ExtHeartRate,
    ExtCadence,
}

struct ParserContext {
    in_gpx: bool,
    in_metadata: bool,
    in_track: bool,
    in_track_segment: bool,
    in_track_point: bool,
    current_tag: Option<GpxXmlTag>,
    current_track_point: TrackPoint,
    should_sort_track: bool,
}

impl ParserContext {
    fn new() -> ParserContext {
        ParserContext {
            in_gpx: false,
            in_metadata: false,
            in_track: false,
            in_track_segment: false,
            in_track_point: false,
            current_tag: None,
            current_track_point: TrackPoint::new(),
            should_sort_track: false,
        }
    }
}

const TOPOGRAFIX_GPX_SCHEMA: &'static str = "http://www.topografix.com/GPX/1/1";
const GARMIN_TRACK_POINT_EXT_SCHEMA: &'static str =
    "http://www.garmin.com/xmlschemas/TrackPointExtension/v1";

const TOPOGRAFIX_GPX_MAPPINGS: [(&'static str, GpxXmlTag); 8] = [
    ("gpx", GpxXmlTag::Gpx),
    ("metadata", GpxXmlTag::Metadata),
    ("trk", GpxXmlTag::Track),
    ("name", GpxXmlTag::Name),
    ("trkseg", GpxXmlTag::TrackSegment),
    ("trkpt", GpxXmlTag::TrackPoint),
    ("ele", GpxXmlTag::Elevation),
    ("time", GpxXmlTag::Time),
];

const GARMIN_TRACK_POINT_EXT_MAPPINGS: [(&'static str, GpxXmlTag); 2] = [
    ("hr", GpxXmlTag::ExtHeartRate),
    ("cad", GpxXmlTag::ExtCadence),
];

fn find_tag_in_mapping(tag: &str, mapping: &[(&'static str, GpxXmlTag)]) -> Option<GpxXmlTag> {
    let found = mapping.iter().find(|&&(mapped_tag, _)| mapped_tag == tag);
    match found {
        Some((_, value)) => Some(*value),
        None => None,
    }
}

fn parse_gpx_xml_tag(name: &OwnedName) -> Option<GpxXmlTag> {
    if name.namespace.is_none() {
        return None;
    }

    let namespace = name.namespace.as_ref().unwrap().as_str();
    let tag = name.local_name.as_str();
    match namespace {
        TOPOGRAFIX_GPX_SCHEMA => find_tag_in_mapping(tag, &TOPOGRAFIX_GPX_MAPPINGS),
        GARMIN_TRACK_POINT_EXT_SCHEMA => find_tag_in_mapping(tag, &GARMIN_TRACK_POINT_EXT_MAPPINGS),
        _ => None,
    }
}

fn parse_start_xml_element(
    tag: GpxXmlTag,
    attributes: &Vec<OwnedAttribute>,
    context: &mut ParserContext,
) -> Result<(), ParseError> {
    context.current_tag = Some(tag);

    match tag {
        GpxXmlTag::Gpx => context.in_gpx = true,
        GpxXmlTag::Metadata => {
            if !context.in_gpx {
                return Err(ParseError::XmlError);
            }

            context.in_metadata = true;
        }
        GpxXmlTag::Time => {
            if !context.in_gpx {
                return Err(ParseError::XmlError);
            }
        }
        GpxXmlTag::Track => context.in_track = true,
        GpxXmlTag::Name => {
            if !context.in_gpx || !context.in_track {
                return Err(ParseError::XmlError);
            }
        }
        GpxXmlTag::TrackSegment => context.in_track_segment = true,
        GpxXmlTag::TrackPoint => {
            if !context.in_gpx
                || !context.in_track
                || !context.in_track_segment
                || attributes.len() < 2
            {
                return Err(ParseError::XmlError);
            }

            context.in_track_point = true;
            context.current_track_point = TrackPoint::new();

            let mut latitude_found = false;
            let mut longitude_found = false;

            for attr in attributes {
                if attr.name.local_name == "lat" {
                    latitude_found = true;
                    match attr.value.parse::<f64>() {
                        Ok(parsed) => context.current_track_point.latitude = parsed,
                        Err(_) => return Err(ParseError::XmlError),
                    }
                } else if attr.name.local_name == "lon" {
                    longitude_found = true;
                    match attr.value.parse::<f64>() {
                        Ok(parsed) => context.current_track_point.longitude = parsed,
                        Err(_) => return Err(ParseError::XmlError),
                    }
                }
            }

            if !latitude_found || !longitude_found {
                return Err(ParseError::XmlError);
            }
        }
        GpxXmlTag::Elevation => {
            if !context.in_gpx
                || !context.in_track
                || !context.in_track_segment
                || !context.in_track_point
            {
                return Err(ParseError::XmlError);
            }
        }
        GpxXmlTag::ExtHeartRate => {
            if !context.in_gpx
                || !context.in_track
                || !context.in_track_segment
                || !context.in_track_point
            {
                return Err(ParseError::XmlError);
            }
        }
        GpxXmlTag::ExtCadence => {
            if !context.in_gpx
                || !context.in_track
                || !context.in_track_segment
                || !context.in_track_point
            {
                return Err(ParseError::XmlError);
            }
        }
    }

    Ok(())
}

fn parse_xml_characters(
    characters: String,
    track: &mut Track,
    context: &mut ParserContext,
) -> Result<(), ParseError> {
    if context.current_tag.is_none() {
        return Ok(());
    }

    match context.current_tag.unwrap() {
        GpxXmlTag::Time => {
            let start_time = DateTime::parse_from_rfc3339(&characters);
            match start_time {
                Err(_) => return Err(ParseError::XmlError),
                _ => {}
            }
            let start_time = DateTime::<Utc>::from(start_time.unwrap());

            if context.in_metadata {
                track.start_time = Some(start_time);
            } else if context.in_track_point {
                context.current_track_point.time = start_time;

                // Check whether current track point comes after the latest point.
                // If not, it should sort track later
                if (track.route.len() > 0) && (!context.should_sort_track) {
                    let latest_point = &track.route[track.route.len() - 1];
                    if latest_point.time.gt(&context.current_track_point.time) {
                        context.should_sort_track = true;
                    }
                }
            }
        }
        GpxXmlTag::Name => {
            track.name = characters;
        }
        GpxXmlTag::Elevation => match characters.parse::<f64>() {
            Ok(parsed) => context.current_track_point.elevation = parsed,
            Err(_) => return Err(ParseError::XmlError),
        },
        GpxXmlTag::ExtHeartRate => match characters.parse::<u8>() {
            Ok(parsed) => context.current_track_point.heart_rate = parsed,
            Err(_) => return Err(ParseError::XmlError),
        },
        GpxXmlTag::ExtCadence => match characters.parse::<u8>() {
            Ok(parsed) => context.current_track_point.cadence = parsed,
            Err(_) => return Err(ParseError::XmlError),
        },
        _ => {}
    }

    Ok(())
}

fn parse_end_xml_element(tag: GpxXmlTag, track: &mut Track, context: &mut ParserContext) {
    context.current_tag = None;

    match tag {
        GpxXmlTag::Gpx => context.in_gpx = false,
        GpxXmlTag::Metadata => context.in_metadata = false,
        GpxXmlTag::Track => context.in_track = false,
        GpxXmlTag::TrackSegment => context.in_track_segment = false,
        GpxXmlTag::TrackPoint => track.route.push(context.current_track_point),
        _ => {}
    }
}

fn read_gpx_from<R: Read>(reader: BufReader<R>) -> Result<Track, ParseError> {
    let parser = EventReader::new(reader);
    let mut track = Track::new();
    let mut context = ParserContext::new();

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                let tag = parse_gpx_xml_tag(&name);
                if tag.is_none() {
                    continue;
                }

                let tag = tag.unwrap();
                if let Err(err) = parse_start_xml_element(tag, &attributes, &mut context) {
                    return Err(err);
                }
            }
            Ok(XmlEvent::EndElement { name, .. }) => {
                let tag = parse_gpx_xml_tag(&name);
                if tag.is_none() {
                    continue;
                }

                let tag = tag.unwrap();
                parse_end_xml_element(tag, &mut track, &mut context);
            }
            Ok(XmlEvent::Characters(characters)) => {
                if let Err(err) = parse_xml_characters(characters, &mut track, &mut context) {
                    return Err(err);
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                return Err(ParseError::XmlError);
            }
            _ => {}
        }
    }

    if context.should_sort_track {
        track.route.sort_by(|a, b| a.time.cmp(&b.time));
    }

    Ok(track)
}

pub fn read_gpx(path: &str) -> Result<Track, ParseError> {
    let file = File::open(path).unwrap();
    let file = BufReader::new(file);

    read_gpx_from(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_case::test_case;

    #[test_case("gpx", GpxXmlTag::Gpx; "gpx tag")]
    #[test_case("metadata", GpxXmlTag::Metadata; "metadata tag")]
    #[test_case("trk", GpxXmlTag::Track; "trk tag")]
    #[test_case("name", GpxXmlTag::Name; "name tag")]
    #[test_case("trkseg", GpxXmlTag::TrackSegment; "trkseg tag")]
    #[test_case("trkpt", GpxXmlTag::TrackPoint; "trkpt tag")]
    #[test_case("ele", GpxXmlTag::Elevation; "ele tag")]
    #[test_case("time", GpxXmlTag::Time; "time tag")]
    fn test_topografix_gpx_mapping(tag: &str, expected: GpxXmlTag) {
        let name = OwnedName {
            local_name: String::from(tag),
            namespace: Some(String::from(TOPOGRAFIX_GPX_SCHEMA)),
            prefix: None,
        };

        let parsed = parse_gpx_xml_tag(&name);
        assert_eq!(parsed, Some(expected));
    }

    #[test_case("hr", GpxXmlTag::ExtHeartRate; "hr tag")]
    #[test_case("cad", GpxXmlTag::ExtCadence; "cad tag")]
    fn test_garmin_track_point_ext_gpx_mapping(tag: &str, expected: GpxXmlTag) {
        let name = OwnedName {
            local_name: String::from(tag),
            namespace: Some(String::from(GARMIN_TRACK_POINT_EXT_SCHEMA)),
            prefix: None,
        };

        let parsed = parse_gpx_xml_tag(&name);
        assert_eq!(parsed, Some(expected));
    }

    #[test]
    fn test_unknown_namespace_gpx_mapping() {
        let name = OwnedName {
            local_name: String::from("gpx"),
            namespace: Some(String::from("")),
            prefix: None,
        };

        let parsed = parse_gpx_xml_tag(&name);
        assert_eq!(parsed, None);
    }

    #[test]
    fn test_unknown_tag_gpx_mapping() {
        let name = OwnedName {
            local_name: String::from("unknown"),
            namespace: Some(String::from(TOPOGRAFIX_GPX_SCHEMA)),
            prefix: None,
        };

        let parsed = parse_gpx_xml_tag(&name);
        assert_eq!(parsed, None);
    }

    #[test]
    fn test_parsing_simple_gpx() {
        let gpx_str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<gpx creator=\"StravaGPX\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" 
xsi:schemaLocation=\"http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/1/1/gpx.xsd http://www.garmin.com/xmlschemas/GpxExtensions/v3 http://www.garmin.com/xmlschemas/GpxExtensionsv3.xsd http://www.garmin.com/xmlschemas/TrackPointExtension/v1 http://www.garmin.com/xmlschemas/TrackPointExtensionv1.xsd\" 
version=\"1.1\" 
xmlns=\"http://www.topografix.com/GPX/1/1\" 
xmlns:gpxtpx=\"http://www.garmin.com/xmlschemas/TrackPointExtension/v1\" 
xmlns:gpxx=\"http://www.garmin.com/xmlschemas/GpxExtensions/v3\">
</gpx>
        ".as_bytes();
        let reader = BufReader::new(gpx_str);

        let result = read_gpx_from(reader);
        assert!(result.is_ok());
        let track = result.unwrap();
        assert_eq!(track.name, "");
        assert_eq!(track.start_time, None);
        assert_eq!(track.route.len(), 0);
    }

    #[test]
    fn test_parsing_metadata_gpx() {
        let gpx_str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<gpx creator=\"StravaGPX\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" 
xsi:schemaLocation=\"http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/1/1/gpx.xsd http://www.garmin.com/xmlschemas/GpxExtensions/v3 http://www.garmin.com/xmlschemas/GpxExtensionsv3.xsd http://www.garmin.com/xmlschemas/TrackPointExtension/v1 http://www.garmin.com/xmlschemas/TrackPointExtensionv1.xsd\" 
version=\"1.1\" 
xmlns=\"http://www.topografix.com/GPX/1/1\" 
xmlns:gpxtpx=\"http://www.garmin.com/xmlschemas/TrackPointExtension/v1\" 
xmlns:gpxx=\"http://www.garmin.com/xmlschemas/GpxExtensions/v3\">
    <metadata>
        <time>2020-04-22T16:01:58Z</time>
    </metadata>
</gpx>
        ".as_bytes();
        let reader = BufReader::new(gpx_str);

        let result = read_gpx_from(reader);
        assert!(result.is_ok());
        let track = result.unwrap();
        assert_eq!(track.name, "");
        assert_eq!(track.route.len(), 0);

        let expected_time = Utc.ymd(2020, 4, 22).and_hms(16, 01, 58);
        assert_eq!(track.start_time, Some(expected_time));
    }

    #[test]
    fn test_parsing_gpx() {
        let gpx_str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<gpx creator=\"StravaGPX\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" 
xsi:schemaLocation=\"http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/1/1/gpx.xsd http://www.garmin.com/xmlschemas/GpxExtensions/v3 http://www.garmin.com/xmlschemas/GpxExtensionsv3.xsd http://www.garmin.com/xmlschemas/TrackPointExtension/v1 http://www.garmin.com/xmlschemas/TrackPointExtensionv1.xsd\" 
version=\"1.1\" 
xmlns=\"http://www.topografix.com/GPX/1/1\" 
xmlns:gpxtpx=\"http://www.garmin.com/xmlschemas/TrackPointExtension/v1\" 
xmlns:gpxx=\"http://www.garmin.com/xmlschemas/GpxExtensions/v3\">
    <metadata>
        <time>2020-04-22T16:01:58Z</time>
    </metadata>
    <trk>
        <name>Test run</name>
        <type>9</type>
        <trkseg>
            <trkpt lat=\"10.1025420\" lon=\"15.1583540\">
                <ele>478.2</ele>
                <time>2020-04-22T16:01:58Z</time>
                <extensions>
                    <gpxtpx:TrackPointExtension>
                        <gpxtpx:hr>95</gpxtpx:hr>
                        <gpxtpx:cad>79</gpxtpx:cad>
                    </gpxtpx:TrackPointExtension>
                </extensions>
            </trkpt>
            <trkpt lat=\"10.1025432\" lon=\"15.1583542\">
                <ele>480.3</ele>
                <time>2020-04-22T16:02:04Z</time>
                <extensions>
                    <gpxtpx:TrackPointExtension>
                        <gpxtpx:hr>98</gpxtpx:hr>
                        <gpxtpx:cad>80</gpxtpx:cad>
                    </gpxtpx:TrackPointExtension>
                </extensions>
            </trkpt>
        </trkseg>
    </trk>
</gpx>".as_bytes();
        let reader = BufReader::new(gpx_str);

        let result = read_gpx_from(reader);
        assert!(result.is_ok());
        let track = result.unwrap();
        assert_eq!(track.name, "Test run");
        assert_eq!(track.route.len(), 2);

        let expected_time = Utc.ymd(2020, 4, 22).and_hms(16, 01, 58);
        assert_eq!(track.start_time, Some(expected_time));

        let point_0_time = Utc.ymd(2020, 4, 22).and_hms(16, 01, 58);
        assert_eq!(track.route[0].latitude, 10.1025420);
        assert_eq!(track.route[0].longitude, 15.1583540);
        assert_eq!(track.route[0].elevation, 478.2);
        assert_eq!(track.route[0].time, point_0_time);
        assert_eq!(track.route[0].heart_rate, 95);
        assert_eq!(track.route[0].cadence, 79);

        let point_1_time = Utc.ymd(2020, 4, 22).and_hms(16, 02, 04);
        assert_eq!(track.route[1].latitude, 10.1025432);
        assert_eq!(track.route[1].longitude, 15.1583542);
        assert_eq!(track.route[1].elevation, 480.3);
        assert_eq!(track.route[1].time, point_1_time);
        assert_eq!(track.route[1].heart_rate, 98);
        assert_eq!(track.route[1].cadence, 80);
    }

    #[test]
    fn test_parsing_gpx_with_invalid_point_order() {
        let gpx_str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<gpx creator=\"StravaGPX\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" 
xsi:schemaLocation=\"http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/1/1/gpx.xsd http://www.garmin.com/xmlschemas/GpxExtensions/v3 http://www.garmin.com/xmlschemas/GpxExtensionsv3.xsd http://www.garmin.com/xmlschemas/TrackPointExtension/v1 http://www.garmin.com/xmlschemas/TrackPointExtensionv1.xsd\" 
version=\"1.1\" 
xmlns=\"http://www.topografix.com/GPX/1/1\" 
xmlns:gpxtpx=\"http://www.garmin.com/xmlschemas/TrackPointExtension/v1\" 
xmlns:gpxx=\"http://www.garmin.com/xmlschemas/GpxExtensions/v3\">
    <metadata>
        <time>2020-04-22T16:01:58Z</time>
    </metadata>
    <trk>
        <name>Test run</name>
        <type>9</type>
        <trkseg>
            <trkpt lat=\"10.1025432\" lon=\"15.1583542\">
                <ele>480.3</ele>
                <time>2020-04-22T16:02:04Z</time>
                <extensions>
                    <gpxtpx:TrackPointExtension>
                        <gpxtpx:hr>98</gpxtpx:hr>
                        <gpxtpx:cad>80</gpxtpx:cad>
                    </gpxtpx:TrackPointExtension>
                </extensions>
            </trkpt>
            <trkpt lat=\"10.1025420\" lon=\"15.1583540\">
                <ele>478.2</ele>
                <time>2020-04-22T16:01:58Z</time>
                <extensions>
                    <gpxtpx:TrackPointExtension>
                        <gpxtpx:hr>95</gpxtpx:hr>
                        <gpxtpx:cad>79</gpxtpx:cad>
                    </gpxtpx:TrackPointExtension>
                </extensions>
            </trkpt>
        </trkseg>
    </trk>
</gpx>".as_bytes();
        let reader = BufReader::new(gpx_str);

        let result = read_gpx_from(reader);
        assert!(result.is_ok());
        let track = result.unwrap();
        assert_eq!(track.name, "Test run");
        assert_eq!(track.route.len(), 2);

        let expected_time = Utc.ymd(2020, 4, 22).and_hms(16, 01, 58);
        assert_eq!(track.start_time, Some(expected_time));

        let point_0_time = Utc.ymd(2020, 4, 22).and_hms(16, 01, 58);
        assert_eq!(track.route[0].latitude, 10.1025420);
        assert_eq!(track.route[0].longitude, 15.1583540);
        assert_eq!(track.route[0].elevation, 478.2);
        assert_eq!(track.route[0].time, point_0_time);
        assert_eq!(track.route[0].heart_rate, 95);
        assert_eq!(track.route[0].cadence, 79);

        let point_1_time = Utc.ymd(2020, 4, 22).and_hms(16, 02, 04);
        assert_eq!(track.route[1].latitude, 10.1025432);
        assert_eq!(track.route[1].longitude, 15.1583542);
        assert_eq!(track.route[1].elevation, 480.3);
        assert_eq!(track.route[1].time, point_1_time);
        assert_eq!(track.route[1].heart_rate, 98);
        assert_eq!(track.route[1].cadence, 80);
    }
}
