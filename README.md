# A Prometheus and OpenMetrics parser

The `nom-openmetrics` crate supports [Prometheus] and [OpenMetrics] metrics
exposition formats, but does not validate either format.

```rust
use nom_openmetrics::parser::prometheus;

let input = "";

let (_remaining, output) = prometheus(&input).unwrap();

println!("{output:?}");
```

Outputs:

```
[
    Family {
        descriptors: [
            Help {
                metric: "adsb_aircraft_observed_recent",
                help: "Number of aircraft observed in the last minute",
            },
            Type {
                metric: "adsb_aircraft_observed_recent",
                type: Gauge,
            },
        ],
        samples: [
            Sample {
                name: "adsb_aircraft_observed_recent",
                labels: [
                    Label {
                        name: "frequency",
                        value: "1090",
                    },
                ],
                number: 37.0,
            },
            Sample {
                name: "adsb_aircraft_observed_recent",
                labels: [
                    Label {
                        name: "frequency",
                        value: "978",
                    },
                ],
                number: 1.0,
            },
        ],
    },
]
```

Only a complete nom parser is implemented

[OpenMetrics]: https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md
[Prometheus]: https://prometheus.io/docs/instrumenting/exposition_formats/
