import { useEffect, useState } from "react";
import type { Progress } from "./bindings/Progress";
import { getProgress, advance } from "./api";

export function App() {
    const [progress, setProgress] = useState<Progress | null>(null);
    const [busy, setBusy] = useState(false);

    useEffect(() => {
        getProgress().then(setProgress).catch(console.error);
    }, []);

    const onAdvance = async () => {
        setBusy(true);
        try {
            setProgress(await advance());
        } finally {
            setBusy(false);
        }
    };

    if (!progress) return <p>Loading…</p>;

    return (
        <main style={{ fontFamily: "system-ui", maxWidth: 480, margin: "3rem auto", textAlign: "center" }}>
            <h1>World's Toughest Row</h1>
            <p>{(progress.total_meters / 1000).toFixed(1)} km · {progress.percent_complete.toFixed(1)}%</p>
            <p>{(progress.meters_remaining / 1000).toFixed(0)} km remaining</p>
            <button onClick={onAdvance} disabled={busy} style={{ fontSize: "1.5rem", padding: "1rem 2rem" }}>
                {busy ? "…" : "+500 m"}
            </button>
        </main>
    );
}
