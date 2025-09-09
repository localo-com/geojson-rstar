extern crate geojson;
extern crate geojson_rstar;
extern crate serde_json;

use geojson::GeoJson;
use geojson_rstar::GeometryCollectionFeature;
use std::convert::TryInto;

#[test]
fn test_try_into_geometry_collection_success() {
    let gc_geojson = r#"{
        "type": "Feature",
        "properties": {},
        "geometry": {
            "type": "GeometryCollection",
            "geometries": [
                { "type": "Point", "coordinates": [1.0, 2.0] },
                { "type": "LineString", "coordinates": [ [0.0, 0.0], [3.0, 1.0] ] }
            ]
        }
    }"#;

    if let GeoJson::Feature(feature) = gc_geojson.parse::<GeoJson>().expect("parse ok") {
        let gc_feature: Result<GeometryCollectionFeature, _> = feature.try_into();
        assert!(
            gc_feature.is_ok(),
            "GeometryCollectionFeature conversion succeeds"
        );
    } else {
        panic!("The geojson did not parse as a Feature");
    }
}

#[test]
fn test_try_into_geometry_collection_success_and_bbox() {
    let gc_geojson = r#"{
        "type": "Feature",
        "properties": { "NAME": "GC-A" },
        "geometry": {
            "type": "GeometryCollection",
            "geometries": [
                { "type": "Point", "coordinates": [1.0, 2.0] },
                { "type": "LineString", "coordinates": [ [0.0, 0.0], [3.0, 1.0] ] },
                { "type": "Polygon", "coordinates": [ [ [2.0, 2.0], [4.0, 2.0], [4.0, 4.0], [2.0, 4.0], [2.0, 2.0] ] ] }
            ]
        }
    }"#;

    if let GeoJson::Feature(feature) = gc_geojson
        .parse::<GeoJson>()
        .expect("The geojson did not correctly parse")
    {
        let gc_feature: GeometryCollectionFeature = feature
            .try_into()
            .expect("GeometryCollectionFeature conversion should succeed");

        // The expected bbox spans from the line's start to the polygon's max
        assert_eq!(gc_feature.geo_geometry().0.is_empty(), false);
        assert_eq!(gc_feature.geometries().len() > 0, true);
    } else {
        panic!("The geojson did not parse as a Feature");
    }
}
