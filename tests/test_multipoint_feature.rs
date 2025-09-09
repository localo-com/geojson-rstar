extern crate geojson;
extern crate geojson_rstar;
extern crate rstar;
extern crate serde_json;

use geojson::GeoJson;
use geojson_rstar::MultiPointFeature;
use rstar::RTree;
use std::convert::TryInto;

#[test]
fn test_try_into_multipoint_feature_success() {
    let geojson_string = r#"{
        "type": "Feature",
        "properties": {"NAME": "ClusterA"},
        "geometry": {
            "type": "MultiPoint",
            "coordinates": [ [0.0, 0.0], [1.0, 1.0] ]
        }
    }"#;

    if let GeoJson::Feature(feature) = geojson_string
        .parse::<GeoJson>()
        .expect("The geojson did not correctly parse")
    {
        let feature: Result<MultiPointFeature, _> = feature.try_into();
        assert!(
            feature.is_ok(),
            "MultiPointFeature converts from MultiPoint geometry"
        );
    } else {
        panic!("The geojson did not parse as a Feature");
    }
}

#[test]
fn test_multipoint_nearest_neighbor() {
    let multipoints_geojson = r#"{
"type": "FeatureCollection",
"features": [
  { "type": "Feature", "properties": { "NAME": "ClusterA" }, "geometry": { "type": "MultiPoint", "coordinates": [ [0.0, 0.0], [1.0, 1.0] ] } },
  { "type": "Feature", "properties": { "NAME": "ClusterB" }, "geometry": { "type": "MultiPoint", "coordinates": [ [10.0, 10.0], [11.0, 11.0] ] } }
]
}"#;

    let search_point = [0.5, 0.5];

    if let Ok(GeoJson::FeatureCollection(feature_collection)) =
        multipoints_geojson.parse::<GeoJson>()
    {
        let features = feature_collection
            .features
            .into_iter()
            .map(|f| f.try_into())
            .collect::<Result<Vec<MultiPointFeature>, _>>()
            .expect("The features were correctly converted as MultiPointFeatures");
        let r_tree = RTree::bulk_load(features);
        let nearest = r_tree
            .nearest_neighbor(&search_point)
            .expect("There is a nearest MultiPointFeature in the RTree");

        assert_eq!(
            nearest.properties.as_ref().unwrap().get("NAME"),
            Some(serde_json::Value::String("ClusterA".to_string())).as_ref()
        );
    } else {
        panic!("The geojson did not parse as a FeatureCollection correctly");
    }
}

#[test]
fn test_multipoint_rejects_empty_or_malformed() {
    // Empty multipoint should fail conversion
    let empty = r#"{ "type": "Feature", "properties": {}, "geometry": { "type": "MultiPoint", "coordinates": [] } }"#;
    if let GeoJson::Feature(f) = empty.parse::<GeoJson>().unwrap() {
        let res: Result<MultiPointFeature, _> = f.try_into();
        assert!(res.is_err(), "Empty MultiPoint should be malformed");
    }

    // Malformed: one position has 3 elements
    let malformed = r#"{ "type": "Feature", "properties": {}, "geometry": { "type": "MultiPoint", "coordinates": [[0,0,0]] } }"#;
    if let GeoJson::Feature(f) = malformed.parse::<GeoJson>().unwrap() {
        let res: Result<MultiPointFeature, _> = f.try_into();
        assert!(res.is_err(), "Malformed MultiPoint should be rejected");
    }
}
