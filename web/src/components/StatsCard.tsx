import type { Progress } from "../bindings/Progress";
import { overlay } from "../styles/overlay";

export function StatsCard({ progress }: { progress: Progress }) {
    const rowed = progress.total_meters;
    const total = progress.total_meters + progress.meters_remaining;
    const pct = progress.percent_complete;

    return (
        <div style={overlay({ top: 16, left: 16, width: 260, padding: 16 })}>
            <h1 style={{ margin: 0, fontSize: "1.1rem" }}>World's Toughest Row</h1>
            <p style={{ margin: "8px 0 6px", fontSize: ".85rem", opacity: 0.7 }}>
                {(rowed / 1000).toFixed(0)} / {(total / 1000).toFixed(0)} km ·{" "}
                {(progress.meters_remaining / 1000).toFixed(0)} km left
            </p>
            <div style={{ height: 8, borderRadius: 4, background: "#e5e7eb", overflow: "hidden" }}>
                <div style={{ width: `${pct}%`, height: "100%", background: "#2563eb" }} />
            </div>
            <p style={{ margin: "4px 0 0", fontSize: ".8rem", textAlign: "right" }}>{pct.toFixed(1)}%</p>
        </div>
    );
}
