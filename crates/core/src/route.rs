use api_types::LatLng;

pub struct Route {
    pub waypoints: Vec<LatLng>,
    cum_m: Vec<f64>, // Cumulative meters at each waypoint.
    pub total_m: f64,
}

impl Route {
    pub fn worlds_toughest_row() -> Self {
        // Placeholder waypoints La Gomera -> Antigua; refine when we build the map.
        Self::new(vec![
            LatLng { lat: 28.0916, lng: -17.1133 }, // La Gomera.
            LatLng { lat: 24.0,    lng: -30.0    },
            LatLng { lat: 20.0,    lng: -45.0    },
            LatLng { lat: 17.05,   lng: -61.77   }, // Antigua.
        ])
    }

    pub fn new(waypoints: Vec<LatLng>) -> Self {
        let mut cum = vec![0.0];
        let mut acc = 0.0;
        for w in waypoints.windows(2) {
            acc += haversine_m(w[0], w[1]);
            cum.push(acc);
        }
        Route { waypoints, cum_m: cum, total_m: acc }
    }

    pub fn position_at(&self, meters: f64) -> LatLng {
        let d = meters.clamp(0.0, self.total_m);
        for i in 1..self.cum_m.len() {
            if d <= self.cum_m[i] {
                let seg = self.cum_m[i] - self.cum_m[i - 1];
                let t = if seg > 0.0 { (d - self.cum_m[i - 1]) / seg } else { 0.0 };
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

fn lerp(a: LatLng, b: LatLng, t: f64) -> LatLng {
    LatLng { lat: a.lat + (b.lat - a.lat) * t, lng: a.lng + (b.lng - a.lng) * t }
}

fn haversine_m(a: LatLng, b: LatLng) -> f64 {
    let r = 6_371_000.0_f64;
    let (la1, la2) = (a.lat.to_radians(), b.lat.to_radians());
    let dla = (b.lat - a.lat).to_radians();
    let dlo = (b.lng - a.lng).to_radians();
    let h = (dla / 2.0).sin().powi(2) + la1.cos() * la2.cos() * (dlo / 2.0).sin().powi(2);
    2.0 * r * h.sqrt().asin()
}
