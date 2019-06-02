# TO DO
- Complete presentation
- Clean code
- Fix horizontal-vertical squashing
- Render in whole screen
- Draw borders on un-drawn area
- Use correct depth... relations?

# Analyzing the Green Line Extension with OpenStreetMap

The Green Line Extension will add seven new stations to the north end of the MBTA green line. It will cost about $3
billion and is slated for completion about two years from now.

[insert a map showing the planned work, in comparison to red and green lines]

I want to look at how much the Green Line will change transit times.

## 🐃 Yak Shaving 🐃 

I'd like to analyze transit times.. so I clearly need to write an OpenStreetMap file ingester and a GPU-based map
renderer!?

- Ingester
- Renderer
- Glue

## OpenStreetMap

OpenStreetMap is like Wikipedia for geographical data. Besides literally streets, it contains buildings, parks, tracks,
and more. It powers many maps that you see around the Internet.

## Getting the data

[Geofabrik GmbH](http://download.geofabrik.de/) provides downloads of various regions, down to U.S. states. I chose to
download the Massachusetts data in `.pbf` format, The data is is encoded with Google's protocol buffers, a binary
encoding format. 

## OSM technical

I make use of two OSM concepts:

* A *node* is basically a point in space with a lat/long
* A *way* is a list of nodes, e.g. road, building, really anything that's not a point

## Interpreting the data

## Nodes

Data:

- ID
- latitude + longitude 
- Key-value tags, e.g. `amenity`: `vending_machine`, `vending`: `public_transport_tickets`

## Ways

Data:

- ID
- List of nodes
- Key-value tags, e.g. `highway`: `tertiary`, `name`: `Pearl Street`
- Attribution

## OSM PBF format

This is a protocol buffer format for efficiently storing OSM data.

It uses a few neat tricks:

- Splitting each file into blocks to allow parallel processing
- Storing data in columnar formats
- Referring to nodes and strings by ID (string interning)
- Delta-encoding sequences of ints (saves space)

## Splitting into blocks

The file is organized in triplets:

```
- #1 block header size in bytes
- #1 block header
- #1 block
- #2 block header size in bytes
- #2 block header
- #2 block
...
```

```protobuf
message BlobHeader {
  required string type = 1;     <- There are multiple types of blocks
  ...
  required int32 datasize = 3;  <- The size of the upcoming block
}
```

## Columnar data in protobuf

A new trick for me! Each array is the same length:

```protobuf
message DenseNodes {
   repeated sint64 id = 1 [packed = true];

   optional DenseInfo denseinfo = 5;

   repeated sint64 lat = 8 [packed = true];
   repeated sint64 lon = 9 [packed = true];

   // Alternates key, value, key, value, ...
   repeated int32 keys_vals = 10 [packed = true]; 
}
```

Since proto3, scalar numeric fields are packed by default.

## Delta-encoding IDs

Original data: `100`, `101`, `103`, `104`

Delta-encoded: `100`, `1`, `2`, `1`

Why is this good? With protobuf varints, the space required to encode an integer is proportional to the log of its size. Since OSM contains over 5 billion nodes, we're probably encoding most ints in 1 byte instead of 5 bytes, saving 80% space!

## Ingesting the data

I made my own ingester for OSM PBF data! Not sure if that was a good idea.

Challenges:

- The OSM PBF format is a little chaotic, having evolved over time
- Keeping up with the performance tricks
- I needed many layers of mapping, filtering and flat mapping

## Back to the actual analysis

Generally, the methodology is:

1. Calculate the transit time from a station to Boston Common
2. Calculate the walking time from any place to each station
3. Find the best station for each place

Possible extensions:

- Other destinations
- Other modes of transport
- Larger geographic region

## Transit time from station to downtown

**Existing lines:** use schedules, test calibration by hand

**GLX:** measure distance between stations, comparing to existing D branch of the Green Line. I'm assuming that the trains will move at the same average speed, 19 MPH, because the density of stations is not too far off.

## Estimated Transit Times

Measured in minutes to Lechmere

- Lechmere: 0
- Union: 3/4.5
- E. Somerville: 3/?
- Gilman: 5/5
- Magoun: 6/7
- Ball: 9/8.5
- College Ave. 10/10.25

I also estimated the typical wait time between trains at each station.

On the low end, Lechmere will have an expected wait only of 2 minutes since it's on both Green Line branches. On the high end, Sullivan and Assembly Row are more like 9 minutes.

http://www.somervillestep.org/files/GreenLineDEIR_text_1009.pdf

Possible extension: incorporate subway on-time performance

## Walking Times

"As the crow flies" with a constant penalty.

I expect this to introduce some error, especially near difficult-to-pass objects like I-93 and McGrath Highway.

Possible extension: pathfinding with actual sidewalk data

## Finding the best station for a given location

Currently: simply try each station for the given location and use the best one. This isn't always the *closest* station: for example, if you're halfway between Porter and Magoun, prefer Magoun.

[show a map w/ the best vs. closest for every station]

## Rendering a map

Goals: produce useful and informative maps of Somerville.

## What should the map show?

1. It should show the new, old, and/or difference in travel time.
2. I should be able to orient myself and to approximately find a particular building.
3. It should look inoffensive.
4. It should convey the scale of the city.



# End

## Other useful data

- Wikipedia, as usual 
- Google Maps is great
- OpenStreetMap

## Corners cut

- I'm only looking at travel time into Boston. However, there are many other locations worth traveling to.
- This analysis only considers rail transit, excluding car, bike, bus, and more.
- I'm using a crude model of walking, ignoring the environment. In particular, this will be inaccurate around large
  barriers, like McGrath Highway


# Notes

blue to yellow as primary hue dimension, b/c resilient to deutera

- [https://leafletjs.com/](Leaflet): A nice JS renderer for OSM data. Looks like this is what's used on OSM's site. The text is pretty aliased and it's a bit noisy.
- https://maptimeboston.github.io/d3-maptime/#
- https://www.mapbox.com/tour/#maps
- http://sotm-eu.org/slides/79.pdf
- https://programmingdesignsystems.com/color/color-schemes/index.html

## Acknowledgements

OpenStreetMap

Top-level libs include:

- wgpu
- lyon

...and HUNDREDS of lower-level crates that make this possible!

How could you get involved?