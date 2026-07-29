import type { Stats } from "../stats";
import { overlay } from "../styles/overlay";

const m = (n: number) => n.toLocaleString("en-GB");

const rowStyle = {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "baseline",
    padding: "5px 0",
    borderTop: "1px solid #f1f5f9",
    fontSize: ".8rem",
} as const;

function PairRow({ label, current, record, unit }: {
    label: string; current: string; record: string; unit: string;
}) {
    return (
        <div style={rowStyle}>
            <span style={{ opacity: 0.75 }}>{label}</span>
            <span>
                <span style={{ opacity: 0.55 }}>{current}</span>
                <span style={{ opacity: 0.4 }}> / </span>
                <span style={{ fontWeight: 600 }}>{record}</span>
                <span style={{ opacity: 0.5 }}> {unit}</span>
            </span>
        </div>
    );
}

export function RecordsCard({ stats: s }: { stats: Stats }) {
    return (
        <div style={overlay({ top: 16, right: 16, width: 240, padding: 16 })}>
            <h2 style={{ margin: 0, fontSize: ".95rem" }}>Records</h2>
            <p style={{ margin: "2px 0 6px", fontSize: ".7rem", opacity: 0.5 }}>current / best</p>
            <PairRow label="Session" current={m(s.lastSession)} record={m(s.bestSession)} unit="m" />
            <PairRow label="Day" current={m(s.today)} record={m(s.bestDay)} unit="m" />
            <PairRow label="Week" current={m(s.thisWeek)} record={m(s.bestWeek)} unit="m" />
            <PairRow label="Month" current={m(s.thisMonth)} record={m(s.bestMonth)} unit="m" />
        </div>
    );
}
