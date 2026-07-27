import { useCallback, useEffect, useState } from "react";

import type { Progress } from "../bindings/Progress";
import type { Entry } from "../bindings/Entry";
import { getProgress, advance, history } from "../api";

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

    const advanceBy = useCallback(async (meters?: number) => {
        setBusy(true);
        try {
            setProgress(await advance(meters));
            setEntries(await history());
        } finally {
            setBusy(false);
        }
    }, []);

    return { progress, entries, busy, advanceBy };
}
