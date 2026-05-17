// Copyright 2020 Boyd Johnson
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    conversion::create_geo_geometry_collection, error::GeoJsonConversionError,
    generic::GenericFeature, json::JsonObject, LineStringFeature, MultiLineStringFeature,
    MultiPointFeature, MultiPolygonFeature, PointFeature, PolygonFeature,
};
use geo::{algorithm::bounding_rect::BoundingRect, Coord, Rect};
use geojson::{feature::Id, Bbox, Geometry, GeometryValue};
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub struct GeometryCollectionFeature {
    bbox: Bbox,
    geometries: Vec<Geometry>,
    pub id: Option<Id>,
    pub properties: Option<JsonObject>,
    pub foreign_members: Option<JsonObject>,
}

impl GeometryCollectionFeature {
    pub fn geometries(&self) -> &[Geometry] {
        &self.geometries
    }

    pub fn geo_geometry(&self) -> geo::GeometryCollection<f64> {
        create_geo_geometry_collection(self.geometries())
    }
}

impl From<GeometryCollectionFeature> for geojson::Feature {
    fn from(val: GeometryCollectionFeature) -> Self {
        let geometry = geojson::Geometry::new(geojson::GeometryValue::GeometryCollection {
            geometries: val.geometries,
        });

        geojson::Feature {
            id: val.id,
            properties: val.properties,
            foreign_members: val.foreign_members,
            geometry: Some(geometry),
            bbox: Some(val.bbox),
        }
    }
}

impl TryFrom<geojson::Feature> for GeometryCollectionFeature {
    type Error = GeoJsonConversionError;

    fn try_from(feature: geojson::Feature) -> Result<GeometryCollectionFeature, Self::Error> {
        <Self as GenericFeature<GeometryCollectionFeature, Vec<Geometry>>>::try_from(feature)
    }
}

impl GenericFeature<GeometryCollectionFeature, Vec<Geometry>> for GeometryCollectionFeature {
    fn take_geometry_type(
        feature: &mut geojson::Feature,
    ) -> Result<Vec<Geometry>, GeoJsonConversionError> {
        if let geojson::GeometryValue::GeometryCollection { geometries } = feature
            .geometry
            .take()
            .ok_or_else(|| {
                let id = feature.id.clone();
                GeoJsonConversionError::MissingGeometry(id)
            })?
            .value
        {
            Ok(geometries)
        } else {
            Err(GeoJsonConversionError::IncorrectGeometryValue(
                "Error: did not find GeometryCollection feature".to_string(),
            ))
        }
    }

    fn check_geometry(
        geometry: &Vec<Geometry>,
        feature: &geojson::Feature,
    ) -> Result<(), GeoJsonConversionError> {
        for geom in geometry {
            match &geom.value {
                GeometryValue::Point { coordinates } => {
                    PointFeature::check_geometry(coordinates, feature)?
                }
                GeometryValue::LineString { coordinates } => {
                    LineStringFeature::check_geometry(coordinates, feature)?
                }
                GeometryValue::Polygon { coordinates } => {
                    PolygonFeature::check_geometry(coordinates, feature)?
                }
                GeometryValue::MultiPoint { coordinates } => {
                    MultiPointFeature::check_geometry(coordinates, feature)?
                }
                GeometryValue::MultiLineString { coordinates } => {
                    MultiLineStringFeature::check_geometry(coordinates, feature)?
                }
                GeometryValue::MultiPolygon { coordinates } => {
                    MultiPolygonFeature::check_geometry(coordinates, feature)?
                }
                GeometryValue::GeometryCollection { geometries } => {
                    GeometryCollectionFeature::check_geometry(geometries, feature)?
                }
            };
        }
        Ok(())
    }

    fn compute_bbox(feature: &mut geojson::Feature, geometry: &Vec<Geometry>) -> Bbox {
        feature.bbox.take().unwrap_or_else(|| {
            let geo_geometry_collection = create_geo_geometry_collection(geometry);
            let polygons: Vec<geo::Polygon<f64>> = convert_bounding_rect(geo_geometry_collection)
                .into_iter()
                .map(|v| v.into())
                .collect();
            let bounds = geo::MultiPolygon::from(polygons)
                .bounding_rect()
                .expect("Polygons have a bounding rectangle");
            vec![
                bounds.min().x,
                bounds.min().y,
                bounds.max().x,
                bounds.max().y,
            ]
        })
    }

    fn create_self(
        feature: geojson::Feature,
        bbox: Bbox,
        geometry: Vec<Geometry>,
    ) -> GeometryCollectionFeature {
        GeometryCollectionFeature {
            bbox,
            id: feature.id,
            geometries: geometry,
            properties: feature.properties,
            foreign_members: feature.foreign_members,
        }
    }
}

fn convert_bounding_rect(geo_geometry_collection: geo::GeometryCollection<f64>) -> Vec<Rect<f64>> {
    geo_geometry_collection
        .into_iter()
        .flat_map(|geo_geom| match geo_geom {
            geo::Geometry::Point(p) => vec![Rect::new(
                Coord::from((p.x(), p.y())),
                Coord::from((p.x(), p.y())),
            )],
            geo::Geometry::LineString(l) => {
                vec![l.bounding_rect().expect("Expect a bounding rect")]
            }
            geo::Geometry::Polygon(p) => vec![p.bounding_rect().expect("Expect a bounding rect")],
            geo::Geometry::MultiPoint(p) => {
                vec![p.bounding_rect().expect("Expect a bounding rect")]
            }
            geo::Geometry::MultiLineString(l) => {
                vec![l.bounding_rect().expect("Expect a bounding rect")]
            }
            geo::Geometry::MultiPolygon(p) => {
                vec![p.bounding_rect().expect("Expect a bounding rect")]
            }
            geo::Geometry::Line(_) => {
                panic!("GeoJson GeometryCollection Geometry turned into Line, incorrect.");
            }
            geo::Geometry::Rect(_) => {
                panic!("GeoJson GeometryCollection Geometry can not contain Rect");
            }
            geo::Geometry::Triangle(_) => {
                panic!("GeoJson GeometryCollection Geometry can not contain Triangle");
            }
            geo::Geometry::GeometryCollection(g) => convert_bounding_rect(g),
        })
        .collect()
}
