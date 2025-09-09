extern crate geojson;
extern crate geojson_rstar;
extern crate rstar;
extern crate serde_json;

use geojson::GeoJson;
use geojson_rstar::MultiLineStringFeature;
use rstar::RTree;
use std::convert::TryInto;

#[test]
fn test_try_into_multilinestring_feature_success() {
    let geojson_string = r#"{ "type": "Feature", "properties": { "NAME": "MLS-A" }, "geometry": { "type": "MultiLineString", "coordinates": [ [ [0.0, 0.0], [1.0, 1.0] ], [ [1.0, 0.0], [2.0, 1.0] ] ] } }"#;

    if let GeoJson::Feature(feature) = geojson_string
        .parse::<GeoJson>()
        .expect("The geojson did not correctly parse")
    {
        let feature: Result<MultiLineStringFeature, _> = feature.try_into();
        assert!(
            feature.is_ok(),
            "MultiLineStringFeature converts from MultiLineString geometry"
        );
    } else {
        panic!("The geojson did not parse as a Feature");
    }
}

#[test]
fn test_multilinestring_nearest_neighbor() {
    let json = r#"{
"type": "FeatureCollection",
"features": [
  { "type": "Feature", "properties": { "NAME": "MLS-A" }, "geometry": { "type": "MultiLineString", "coordinates": [ [ [0.0, 0.0], [1.0, 1.0] ], [ [1.0, 0.0], [2.0, 1.0] ] ] } },
  { "type": "Feature", "properties": { "NAME": "MLS-B" }, "geometry": { "type": "MultiLineString", "coordinates": [ [ [10.0, 10.0], [11.0, 10.0] ], [ [10.5, 10.5], [11.5, 10.5] ] ] } }
]
}"#;

    let search_point = [0.5, 0.5];

    if let Ok(GeoJson::FeatureCollection(collection)) = json.parse::<GeoJson>() {
        let features = collection
            .features
            .into_iter()
            .map(|f| f.try_into())
            .collect::<Result<Vec<MultiLineStringFeature>, _>>()
            .expect("The features were correctly converted");

        let tree = RTree::bulk_load(features);
        let nearest = tree
            .nearest_neighbor(&search_point)
            .expect("There is a nearest neighbor");
        assert_eq!(
            nearest.properties.as_ref().unwrap().get("NAME"),
            Some(&serde_json::Value::String("MLS-A".into()))
        );
    } else {
        panic!("The geojson did not parse correctly");
    }
}

#[test]
fn test_multilinestring_rejects_empty_or_malformed() {
    // Empty multilinestring should fail
    let empty = r#"{ "type": "Feature", "properties": {}, "geometry": { "type": "MultiLineString", "coordinates": [] } }"#;
    if let GeoJson::Feature(f) = empty.parse::<GeoJson>().unwrap() {
        let res: Result<MultiLineStringFeature, _> = f.try_into();
        assert!(res.is_err(), "Empty MultiLineString should be malformed");
    }
}
