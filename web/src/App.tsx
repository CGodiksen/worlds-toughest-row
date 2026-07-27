import { MapCanvas } from "./components/MapCanvas";
import { StatsCard } from "./components/StatsCard";
import { AdvanceBar } from "./components/AdvanceBar";
import { HistoryPanel } from "./components/HistoryPanel";
import { useRowingData } from "./hooks/useRowingData";

export function App() {
    const { progress, entries, busy, advanceBy } = useRowingData();

    return (
        <div style={{ position: "fixed", inset: 0, fontFamily: "system-ui" }}>
            <MapCanvas progress={progress} />
            {progress && (
                <>
                    <StatsCard progress={progress} />
                    <AdvanceBar onAdvance={advanceBy} busy={busy} />
                    <HistoryPanel entries={entries} />
                </>
            )}
        </div>
    );
}
