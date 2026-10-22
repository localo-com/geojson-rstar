extern crate geojson;
extern crate geojson_rstar;
extern crate serde_json;

use geojson::GeoJson;
use geojson_rstar::Feature;
use std::convert::TryInto;

// serde_json's default float parser is not correctly rounded for every 17 digit
// input (see its `float_roundtrip` feature), so coordinates stay within 15
// significant digits. The properties below are known to parse exactly.
const POLYGON_GEOJSON: &str = r#"{
    "type": "Feature",
    "properties": {
        "shortest": 0.1,
        "long_fraction": 648.7083489267364,
        "projected": 452561.8181,
        "tiny": 1e-7,
        "huge": 1e21,
        "negative_zero": -0.0,
        "max": 1.7976931348623157e308,
        "min_positive": 2.2250738585072014e-308,
        "min_subnormal": 5e-324
    },
    "geometry": {
        "type": "Polygon",
        "coordinates": [[
            [-95.372464159, 45.87291786],
            [-96.096365835, 46.287388581],
            [-95.561709068, 44.807550251],
            [-95.372464159, 45.87291786]
        ]]
    }
}"#;

const FLOAT_PROPERTIES: [(&str, f64); 9] = [
    ("shortest", 0.1),
    ("long_fraction", 648.708_348_926_736_4),
    ("projected", 452_561.818_1),
    ("tiny", 1e-7),
    ("huge", 1e21),
    ("negative_zero", -0.0),
    ("max", f64::MAX),
    ("min_positive", f64::MIN_POSITIVE),
    ("min_subnormal", 5e-324),
];

fn parse_feature(geojson_string: &str) -> Feature {
    if let GeoJson::Feature(feature) = geojson_string
        .parse::<GeoJson>()
        .expect("The geojson did not correctly parse")
    {
        feature
            .try_into()
            .expect("The feature was correctly converted")
    } else {
        panic!("The geojson did not parse as a Feature");
    }
}

fn serialize_feature(feature: Feature) -> String {
    serde_json::to_string(&geojson::Feature::from(feature))
        .expect("The feature was serialized to JSON")
}

#[test]
fn test_json_round_trip_preserves_floats() {
    let feature = parse_feature(POLYGON_GEOJSON);
    let round_tripped = parse_feature(&serialize_feature(feature.clone()));

    assert_eq!(round_tripped, feature);

    let properties = match &round_tripped {
        Feature::Polygon(polygon) => polygon.properties.as_ref().unwrap(),
        other => panic!("Expected a polygon feature, got {:?}", other),
    };
    for (name, expected) in FLOAT_PROPERTIES {
        let actual = properties[name]
            .as_f64()
            .unwrap_or_else(|| panic!("{} is not a float", name));
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "{} changed from {:e} to {:e} after a JSON round trip",
            name,
            expected,
            actual
        );
    }
}

#[test]
fn test_json_serialization_float_format() {
    let serialized = serialize_feature(parse_feature(POLYGON_GEOJSON));

    assert_eq!(
        serialized,
        concat!(
            r#"{"type":"Feature","#,
            r#""bbox":[-96.096365835,44.807550251,-95.372464159,46.287388581],"#,
            r#""geometry":{"type":"Polygon","coordinates":[[[-95.372464159,45.87291786],"#,
            r#"[-96.096365835,46.287388581],[-95.561709068,44.807550251],"#,
            r#"[-95.372464159,45.87291786]]]},"#,
            r#""properties":{"huge":1e+21,"long_fraction":648.7083489267364,"#,
            r#""max":1.7976931348623157e+308,"min_positive":2.2250738585072014e-308,"#,
            r#""min_subnormal":5e-324,"negative_zero":-0.0,"projected":452561.8181,"#,
            r#""shortest":0.1,"tiny":1e-7}}"#,
        )
    );
}
