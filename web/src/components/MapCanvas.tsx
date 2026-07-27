import {useEffect, useRef, useState} from "react";
import {Map as MapLibreMap, Marker, NavigationControl, LngLatBounds} from "maplibre-gl";
import type {StyleSpecification, GeoJSONSource} from "maplibre-gl";
import "maplibre-gl/dist/maplibre-gl.css";
import type {Progress} from "../bindings/Progress";
import type {LatLng} from "../bindings/LatLng";

const MAP_STYLE: StyleSpecification = {
    version: 8,
    sources: {
        osm: {
            type: "raster",
            tiles: ["https://tile.openstreetmap.org/{z}/{x}/{y}.png"],
            tileSize: 256,
            attribution: "© OpenStreetMap contributors",
        },
    },
    layers: [{id: "osm", type: "raster", source: "osm"}],
};

const toLngLat = (p: LatLng): [number, number] => [p.lng, p.lat];

const lineFeature = (points: LatLng[]): GeoJSON.Feature<GeoJSON.LineString> => ({
    type: "Feature",
    geometry: {type: "LineString", coordinates: points.map(toLngLat)},
    properties: {},
});

export function MapCanvas({progress}: { progress: Progress | null }) {
    const containerRef = useRef<HTMLDivElement>(null);
    const mapRef = useRef<MapLibreMap | null>(null);
    const markerRef = useRef<Marker | null>(null);
    const fittedRef = useRef(false);
    const [ready, setReady] = useState(false);

    // Init once.
    useEffect(() => {
        if (!containerRef.current) return;

        const map = new MapLibreMap({
            container: containerRef.current,
            style: MAP_STYLE,
            center: [-40, 25],
            zoom: 2,
        });
        map.addControl(new NavigationControl(), "bottom-right");
        mapRef.current = map;
        map.on("load", () => setReady(true));

        return () => {
            map.remove();
            mapRef.current = null;
            markerRef.current = null;
            fittedRef.current = false;
            setReady(false);
        };
    }, []);

    // Create-or-update sources/layers whenever progress changes.
    useEffect(() => {
        const map = mapRef.current;
        if (!map || !ready || !progress) return;

        const routeData = lineFeature(progress.route);
        const trailData = lineFeature(progress.trail);

        if (!map.getSource("route")) {
            map.addSource("route", { type: "geojson", data: routeData });
            map.addLayer({
                id: "route",
                type: "line",
                source: "route",
                layout: { "line-join": "round" },
                paint: { "line-color": "#9ca3af", "line-width": 2, "line-dasharray": [2, 2] },
            });
        } else {
            (map.getSource("route") as GeoJSONSource).setData(routeData);
        }

        if (!map.getSource("trail")) {
            map.addSource("trail", { type: "geojson", data: trailData });
            map.addLayer({
                id: "trail",
                type: "line",
                source: "trail",
                layout: { "line-cap": "round", "line-join": "round" },
                paint: { "line-color": "#2563eb", "line-width": 5 },
            });
        } else {
            (map.getSource("trail") as GeoJSONSource).setData(trailData);
        }

        if (!markerRef.current) {
            markerRef.current = new Marker({color: "#2563eb"});
        }
        markerRef.current.setLngLat(toLngLat(progress.position)).addTo(map);

        if (!fittedRef.current && progress.route.length > 0) {
            const start = progress.route[0];
            const end = progress.route[progress.route.length - 1];
            new Marker({color: "#16a34a"}).setLngLat(toLngLat(start)).addTo(map);
            new Marker({color: "#dc2626"}).setLngLat(toLngLat(end)).addTo(map);

            const bounds = progress.route.reduce(
                (b, p) => b.extend(toLngLat(p)),
                new LngLatBounds(toLngLat(start), toLngLat(start)),
            );
            map.fitBounds(bounds, {padding: 120, maxZoom: 3.7, duration: 0});
            fittedRef.current = true;
        }
    }, [progress, ready]);

    return <div ref={containerRef} style={{position: "absolute", inset: 0}}/>;
}
