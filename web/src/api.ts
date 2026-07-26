import type { Progress } from "./bindings/Progress";
import type { Entry } from "./bindings/Entry";

export async function getProgress(): Promise<Progress> {
    const r = await fetch("/api/progress");
    if (!r.ok) throw new Error(await r.text());
    return r.json();
}

export async function advance(meters?: number): Promise<Progress> {
    const r = await fetch("/api/entries", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ meters: meters ?? null }),
    });
    if (!r.ok) throw new Error(await r.text());
    return r.json();
}

export async function history(): Promise<Entry[]> {
    const r = await fetch("/api/entries");
    if (!r.ok) throw new Error(await r.text());
    return r.json();
}
