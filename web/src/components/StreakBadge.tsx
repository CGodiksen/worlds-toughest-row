import type { Stats } from "../stats";
import { overlay } from "../styles/overlay";

export function StreakBadge({ stats: s }: { stats: Stats }) {
    const active = s.currentStreak > 0;

    return (
        <div style={overlay({
            top: 16,
            left: "50%",
            transform: "translateX(-50%)",
            padding: "8px 16px",
            display: "flex",
            alignItems: "center",
            gap: 10,
        })}>
            <span style={{ fontSize: "1.4rem", filter: active ? "none" : "grayscale(1)", opacity: active ? 1 : 0.45 }}>
                🔥
            </span>
            <div style={{ fontSize: "1.05rem", fontWeight: 700 }}>
                {s.currentStreak}
                <span style={{ fontSize: ".8rem", fontWeight: 500, opacity: 0.7 }}>
                    {" "}day{s.currentStreak === 1 ? "" : "s"}
                </span>
            </div>
        </div>
    );
}
