# Analyzing the Green Line Extension with OpenStreetMap

This presentation chronicles my attempts to use Rust to understand how the Green Line Extension will affect commutes in Somerville.

![Transit times after](etc/times-after.png)

This map shows transit times to downtown ranging from 10 minutes (orange) to 40 minutes (blue).

## About the GLX

The Green Line Extension will add seven new stations to the north end of the MBTA green line. It will cost about $3 billion and is slated for completion about two years from now.

![Planned Green Line stations](etc/glx-wikipedia.png)

[Map](https://commons.wikimedia.org/wiki/File:Green_Line_Extension.svg) Courtesy Wikipedia user Pi.1415926535, CC-BY-SA 3.0


I want to look at how much the Green Line will change transit times for residents living near planned stations.

## 🐃 Yak Shaving 🐃 

I'd like to analyze transit times... so I clearly need to write an OpenStreetMap file ingester and a GPU-based map
renderer!?

- Ingester
- Renderer
- Glue

## OpenStreetMap

OpenStreetMap is like Wikipedia for geographical data. Besides literally streets, it contains over 5 billion buildings, parks, train tracks, trees, benches, and much more. This data set powers many maps that you see around the Internet.

## Getting the data

[Geofabrik GmbH](http://download.geofabrik.de/) provides downloads of various regions, down to U.S. states. I chose to download the Massachusetts data in `.pbf` format, The data is is encoded with Google's protocol buffers, a binary
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

## Optimizing ingestion

Because the file is split into blocks, I was able to decode each block individually, resulting in a huge speedup.

## The actual analysis

My model of a typical commute is:

1. Walk to the station
2. Wait for the train
3. Train goes downtown

In this model, each resident chooses the station that will result in the lowest total transit time.

Possible extensions:

- Other destinations, like Harvard, MIT, and Kendall are huge cent
- Other modes of transport
- Larger geographic region

## Walking Times

"As the crow flies" with a constant penalty to account for the fact that people don't fly.

I expect this to introduce some error, especially near difficult-to-pass objects like I-93 and McGrath Highway.

Possible extension: pathfinding with actual sidewalk data

## Transit time from station to downtown

**Existing lines:** use schedules, test calibration by hand

**GLX:** measure distance between stations, comparing to existing D branch of the Green Line. I'm assuming that the trains will move at the same average speed, 19 MPH, because the density of stations is not too far off (thanks JBR!).

After doing this, I found a copy of the 2009 [Draft Environmental Impact Report](http://www.somervillestep.org/files/GreenLineDEIR_text_1009.pdf) which thankfully confirmed the sanity of these estimates.

## Estimated Transit Times

Measured in minutes to Lechmere.

- Lechmere: 0
- Union: 3 (DEIR: 4.5)
- E. Somerville: 3 (no DEIR)
- Gilman: 5 (DEIR: 5)
- Magoun: 6 (DEIR: 7)
- Ball: 9 (DEIR: 8.5)
- College Ave. 10 (DEIR: 10.25)

## Wait times

I also estimated the typical wait time between trains at each station.

On the low end, Lechmere will have an expected wait only of 2 minutes since it's on both Green Line branches. On the high end, the orange line is more like 9 minutes.

Possible extension: incorporate subway on-time performance

## Finding the best station for a given location

Currently: simply try each station for the given location and use the best one. This isn't always the *closest* station: for example, if you're halfway between Porter and Magoun, prefer Magoun.

[show a map w/ the best vs. closest for every station]

Possible extension: use heuristics to consider fewer stations

# Final Product

## Rendering a map

Goals: produce useful and informative maps of the Somerville area.

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

# Graphics

## Colors

- Using CIE L* a* b* color space, designed 
Well actually I'm using 

Hold lightness constant.


# Notes

## TO DO

- "Complete" presentation
- Clean code
- Fix horizontal-vertical squashing
- Render in whole screen
- Draw borders on un-drawn area
- Use correct depth... relations?

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