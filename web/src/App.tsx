import { useMemo } from "react";
import { computeStats } from "./stats";
import { MapCanvas } from "./components/MapCanvas";
import { StatsCard } from "./components/StatsCard";
import { AdvanceBar } from "./components/AdvanceBar";
import { HistoryPanel } from "./components/HistoryPanel";
import { useRowingData } from "./hooks/useRowingData";
import {Celebration} from "./components/Celebration";
import { FinishBanner } from "./components/FinishBanner";
import { RecordsCard } from "./components/RecordsCard";
import { StreakBadge } from "./components/StreakBadge";

export function App() {
    const { progress, entries, busy, advanceBy, reset } = useRowingData();
    const stats = useMemo(() => computeStats(entries), [entries]);

    return (
        <div style={{ position: "fixed", inset: 0, fontFamily: "system-ui" }}>
            <MapCanvas progress={progress} />
            {progress && (
                <>
                    <StatsCard progress={progress} />
                    <StreakBadge stats={stats} />
                    <RecordsCard stats={stats} />
                    <AdvanceBar onAdvance={advanceBy} busy={busy} />
                    <HistoryPanel entries={entries} />
                    <Celebration done={progress.percent_complete >= 100} />
                    {progress.percent_complete >= 100 && <FinishBanner onReset={reset} busy={busy} />}
                </>
            )}
        </div>
    );
}
