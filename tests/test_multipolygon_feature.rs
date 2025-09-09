extern crate geojson;
extern crate geojson_rstar;
extern crate rstar;
extern crate serde_json;

use geojson::GeoJson;
use geojson_rstar::MultiPolygonFeature;
use rstar::RTree;
use std::convert::TryInto;

#[test]
fn test_try_into_multipolygon_feature_success() {
    let geojson_string = r#"{ "type": "Feature", "properties": { "NAME": "MP-A" }, "geometry": { "type": "MultiPolygon", "coordinates": [ [ [ [0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0], [0.0, 0.0] ] ] ] } }"#;

    if let GeoJson::Feature(feature) = geojson_string
        .parse::<GeoJson>()
        .expect("The geojson did not correctly parse")
    {
        let feature: Result<MultiPolygonFeature, _> = feature.try_into();
        assert!(
            feature.is_ok(),
            "MultiPolygonFeature converts from MultiPolygon geometry"
        );
    } else {
        panic!("The geojson did not parse as a Feature");
    }
}

#[test]
fn test_multipolygon_nearest_neighbor() {
    let json = r#"{
"type": "FeatureCollection",
"features": [
  { "type": "Feature", "properties": { "NAME": "MP-A" }, "geometry": { "type": "MultiPolygon", "coordinates": [ [ [ [0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0], [0.0, 0.0] ] ] ] } },
  { "type": "Feature", "properties": { "NAME": "MP-B" }, "geometry": { "type": "MultiPolygon", "coordinates": [ [ [ [10.0, 10.0], [12.0, 10.0], [12.0, 12.0], [10.0, 12.0], [10.0, 10.0] ] ] ] } }
]
}"#;

    let search_point = [0.5, 0.5];

    if let Ok(GeoJson::FeatureCollection(collection)) = json.parse::<GeoJson>() {
        let features = collection
            .features
            .into_iter()
            .map(|f| f.try_into())
            .collect::<Result<Vec<MultiPolygonFeature>, _>>()
            .expect("The features were correctly converted");

        let tree = RTree::bulk_load(features);
        let nearest = tree
            .nearest_neighbor(&search_point)
            .expect("There is a nearest neighbor");
        assert_eq!(
            nearest.properties.as_ref().unwrap().get("NAME"),
            Some(&serde_json::Value::String("MP-A".into()))
        );
    } else {
        panic!("The geojson did not parse correctly");
    }
}
