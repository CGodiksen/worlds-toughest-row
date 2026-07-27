import { useCallback, useEffect, useState } from "react";

import type { Progress } from "../bindings/Progress";
import type { Snapshot } from "../bindings/Snapshot";
import type { Entry } from "../bindings/Entry";
import { getProgress, advance, history, reset as resetApi } from "../api";

export function useRowingData() {
    const [progress, setProgress] = useState<Progress | null>(null);
    const [entries, setEntries] = useState<Entry[]>([]);
    const [busy, setBusy] = useState(false);

    const refresh = useCallback(async () => {
        const [p, h] = await Promise.all([getProgress(), history()]);
        setProgress(p);
        setEntries(h);
    }, []);

    useEffect(() => {
        refresh().catch(console.error);
    }, [refresh]);

    // Live updates. The server pushes a full snapshot whenever anything changes, including rows auto-ingested from the
    // erg. EventSource auto-reconnects.
    useEffect(() => {
        const es = new EventSource("/api/events");
        es.onmessage = (e) => {
            const snap: Snapshot = JSON.parse(e.data);
            setProgress(snap.progress);
            setEntries(snap.entries);
        };
        return () => es.close();
    }, []);

    const advanceBy = useCallback(async (meters?: number) => {
        setBusy(true);
        try {
            setProgress(await advance(meters));
            setEntries(await history());
        } finally {
            setBusy(false);
        }
    }, []);

    const reset = useCallback(async () => {
        setBusy(true);
        try {
            setProgress(await resetApi());
            setEntries(await history());
        } finally {
            setBusy(false);
        }
    }, []);

    return { progress, entries, busy, advanceBy, reset };
}
