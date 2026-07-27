use api_types::LatLng;

pub struct Route {
    pub waypoints: Vec<LatLng>,
    cum_m: Vec<f64>, // Cumulative meters at each waypoint.
    pub total_m: f64,
}

impl Route {
    /// La Gomera -> Antigua, sampled along a quadratic Bézier so the trade-wind route reads as
    /// one smooth southward curve.
    pub fn worlds_toughest_row() -> Self {
        // La Gomera.
        let start = LatLng {
            lat: 28.0916,
            lng: -17.1133,
        };

        // Antigua.
        let end = LatLng {
            lat: 17.05,
            lng: -61.77,
        };

        // Control point south of the midpoint; lower lat = deeper bow.
        let control = LatLng {
            lat: 16.0,
            lng: -39.44,
        };

        Self::new(bezier(start, control, end, 99))
    }

    pub fn new(waypoints: Vec<LatLng>) -> Self {
        let mut cum = vec![0.0];
        let mut acc = 0.0;
        for w in waypoints.windows(2) {
            acc += haversine_m(w[0], w[1]);
            cum.push(acc);
        }
        Route {
            waypoints,
            cum_m: cum,
            total_m: acc,
        }
    }

    pub fn position_at(&self, meters: f64) -> LatLng {
        let d = meters.clamp(0.0, self.total_m);
        for i in 1..self.cum_m.len() {
            if d <= self.cum_m[i] {
                let seg = self.cum_m[i] - self.cum_m[i - 1];
                let t = if seg > 0.0 {
                    (d - self.cum_m[i - 1]) / seg
                } else {
                    0.0
                };
                return lerp(self.waypoints[i - 1], self.waypoints[i], t);
            }
        }
        *self.waypoints.last().unwrap()
    }

    pub fn trail_to(&self, meters: f64) -> Vec<LatLng> {
        let d = meters.clamp(0.0, self.total_m);
        let mut trail = vec![self.waypoints[0]];
        for i in 1..self.cum_m.len() {
            if self.cum_m[i] <= d {
                trail.push(self.waypoints[i]);
            } else {
                trail.push(self.position_at(d));
                break;
            }
        }
        trail
    }
}

/// Sample a quadratic Bézier into `segments + 1` points (endpoints exact).
fn bezier(p0: LatLng, p1: LatLng, p2: LatLng, segments: usize) -> Vec<LatLng> {
    (0..=segments)
        .map(|i| {
            let t = i as f64 / segments as f64;
            let u = 1.0 - t;
            LatLng {
                lat: u * u * p0.lat + 2.0 * u * t * p1.lat + t * t * p2.lat,
                lng: u * u * p0.lng + 2.0 * u * t * p1.lng + t * t * p2.lng,
            }
        })
        .collect()
}

fn lerp(a: LatLng, b: LatLng, t: f64) -> LatLng {
    LatLng {
        lat: a.lat + (b.lat - a.lat) * t,
        lng: a.lng + (b.lng - a.lng) * t,
    }
}

fn haversine_m(a: LatLng, b: LatLng) -> f64 {
    let r = 6_371_000.0_f64;
    let (la1, la2) = (a.lat.to_radians(), b.lat.to_radians());
    let dla = (b.lat - a.lat).to_radians();
    let dlo = (b.lng - a.lng).to_radians();
    let h = (dla / 2.0).sin().powi(2) + la1.cos() * la2.cos() * (dlo / 2.0).sin().powi(2);
    2.0 * r * h.sqrt().asin()
}
