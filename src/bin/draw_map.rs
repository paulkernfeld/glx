extern crate env_logger;
extern crate wgpu;

use euclid;
use euclid::{Box2D, Point2D};
use log::*;
use std::collections::HashMap;

use glx::graphics;
use glx::graphics::*;
use glx::protos::osmformat::Way;
use glx::protos::*;
use glx::*;
use rayon::prelude::*;
use std::fs::File;

use geo_types::Point;

#[derive(Debug)]
struct Station {
    name: String,
    minutes_to_psdtx: f32,
    location_x_y: Point2D<f32>,
}

fn load_stations(centroid: Point<f32>) -> Vec<Station> {
    csv::Reader::from_reader(std::fs::File::open("stations.csv").unwrap())
        .records()
        .map(|row| {
            let row = row.unwrap();
            let lat = row[6].parse().unwrap();
            let lon = row[7].parse().unwrap();
            let minutes_to_boston_common = row[5].parse().unwrap();
            Station {
                name: row[0].to_string(),
                location_x_y: lon_lat_to_x_y(&centroid, (lon, lat)),
                minutes_to_psdtx: minutes_to_boston_common,
            }
        })
        .collect()
}

#[derive(Debug)]
struct BestStation {
    station_name: String,
    time: f32,
}

fn best_station(stations: &[Station], location_x_y: Point2D<f32>) -> BestStation {
    let station_time = |station: &Station| {
        let distance_walking = (location_x_y - station.location_x_y).length();
        // Average walking speed is about 5 kph: https://en.wikipedia.org/wiki/Walking
        let average_walking_speed_meters_per_minute = 5.0 * 1_000.0 / 60.0;
        let fudge_factor = 1.2;
        let walk_minutes =
            distance_walking / average_walking_speed_meters_per_minute * fudge_factor;
        walk_minutes + station.minutes_to_psdtx
    };

    let best_station = stations
        .iter()
        .min_by(|s1, s2| f32::partial_cmp(&station_time(s1), &station_time(s2)).unwrap())
        .unwrap();

    BestStation {
        station_name: best_station.name.clone(),
        time: station_time(best_station),
    }
}

fn make_styled_geoms(bb: Box2D<f32>) -> Vec<StyledGeom> {
    // Somerville city hall (93 Highland)
    let centroid: geo_types::Point<f32> = geo_types::Point::new(-71.098472, 42.386755);

    let stations = load_stations(centroid);

    info!("Loading OSM data...");
    let reader = File::open("pbf/massachusetts-latest.osm.pbf").unwrap();
    let vec_blob: Vec<BlobData> = read_blobs(reader).collect();
    let nodes: HashMap<i64, DenseNode> = vec_blob
        .par_iter()
        .map(|blob_data| {
            if let FileBlock::Primitive(primitive_block) = blob_data.deserialize() {
                iter_dense_nodeses(&primitive_block)
                    .flat_map(as_vec_dense_nodes)
                    .collect::<Vec<DenseNode>>()
            } else {
                vec![]
            }
        })
        .collect::<Vec<Vec<DenseNode>>>()
        .into_iter()
        .flatten()
        .into_iter()
        .map(|node: DenseNode| (node.id, node))
        .collect();

    let get_nodes_vec = |way: Way| -> Vec<DenseNode> {
        iter_node_ids(way)
            .map(|node_id| nodes[&node_id].clone())
            .collect()
    };

    let ways: Vec<MyWay> = vec_blob
        .par_iter()
        .map(|blob_data| {
            if let FileBlock::Primitive(primitive_block) = blob_data.deserialize() {
                into_vec_ways(primitive_block)
                    .into_iter()
                    .filter(|way: &MyWay| {
                        get_nodes_vec(way.way.clone())
                            .iter()
                            .any(|node| bb.contains(&dense_node_to_x_y(&node, centroid)))
                    })
                    .collect::<Vec<MyWay>>()
            } else {
                vec![]
            }
        })
        .collect::<Vec<Vec<MyWay>>>()
        .into_iter()
        .flatten()
        .collect();

    info!("{} ways loaded from OSM", ways.len());

    // Popular tags: https://taginfo.openstreetmap.org/tags
    ways.into_par_iter()
        .filter_map(|way: MyWay| {
            let nodes: Vec<_> = get_nodes_vec(way.way)
                .into_iter()
                .map(|node| dense_node_to_x_y(&node, centroid))
                .collect();
            if way.tags.contains_key("building") {
                let color =
                    get_gradient_color((best_station(&stations, nodes[0]).time - 10.0) / 40.0);
                Some(StyledGeom {
                    geom: Geom::Polygon(nodes),
                    color,
                })
            } else if way.tags.contains_key("highway") {
                // It seem like this is in feet
                let meters_per_foot: f32 = 1.0 / 3.0;
                let width = way
                    .tags
                    .get("width")
                    .unwrap_or(&String::from("3.0"))
                    .parse::<f32>()
                    .unwrap_or(3.0)
                    * meters_per_foot;
                Some(StyledGeom {
                    geom: Geom::Lines {
                        points: nodes,
                        width,
                    },
                    color: [0.7, 0.7, 0.7],
                })
            } else {
                Some(StyledGeom {
                    geom: Geom::Lines {
                        points: nodes,
                        width: 3.0,
                    },
                    color: [0.9, 0.9, 0.9],
                })
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_load_stations() {
        let stations = load_stations(geo_types::Point::new(-71.098472, 42.386755));

        let station: &Station = stations
            .iter()
            .find(|station| station.name == "Gilman")
            .unwrap();

        assert!(
            station.location_x_y.to_vector().length() < 500.0,
            "Gilman is pretty close to city hall"
        );
        assert!(
            station.location_x_y.to_vector().length() > 100.0,
            "Gilman is not THAT close to city hall"
        );
        assert_eq!(station.minutes_to_psdtx, 22.0);
    }

    #[test]
    fn test_best_station() {
        let stations = load_stations(geo_types::Point::new(-71.098472, 42.386755));

        let best_station: BestStation = best_station(&stations, Point2D::new(0.0, 0.0));

        assert_eq!(
            best_station.station_name, "Gilman",
            "Gilman is closest to city hall"
        );

        assert!(best_station.time > 23.0);
        assert!(best_station.time < 30.0);
    }
}

fn main() {
    env_logger::init();

    info!("Entering script...");

    let viewport = Box2D::new(Point2D::new(-2000.0, -2000.0), Point2D::new(2000.0, 2000.0));

    graphics::leggo(make_styled_geoms(viewport), viewport);
}
