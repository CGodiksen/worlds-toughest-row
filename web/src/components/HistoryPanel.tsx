import { useState } from "react";
import type { Entry } from "../bindings/Entry";
import { overlay } from "../styles/overlay";

export function HistoryPanel({ entries }: { entries: Entry[] }) {
    const [open, setOpen] = useState(false);

    return (
        <div style={overlay({ bottom: 24, left: 16, width: open ? 280 : "auto", padding: open ? 12 : 0 })}>
            <button
                onClick={() => setOpen((o) => !o)}
                style={{
                    all: "unset",
                    cursor: "pointer",
                    display: "flex",
                    alignItems: "center",
                    gap: 6,
                    padding: open ? "0 4px 8px" : "8px 14px",
                    fontSize: ".85rem",
                    fontWeight: 600,
                }}
            >
                <span style={{ display: "inline-block", transform: open ? "rotate(90deg)" : "none", transition: "transform .15s" }}>
                    ▸
                </span>
                History
            </button>

            {open && (
                <ul style={{ listStyle: "none", margin: 0, padding: 0, maxHeight: 220, overflowY: "auto", paddingRight: 10 }}>
                    {entries.length === 0 && (
                        <li style={{ opacity: 0.6, fontSize: ".8rem", padding: "6px 4px" }}>No entries yet</li>
                    )}
                    {entries.map((e) => (
                        <li
                            key={e.id}
                            style={{
                                display: "flex",
                                justifyContent: "space-between",
                                gap: 12,
                                padding: "6px 4px",
                                borderTop: "1px solid #f1f5f9",
                                fontSize: ".8rem",
                            }}
                        >
                            <span>{e.meters} m</span>
                            <span style={{ opacity: 0.6 }}>{new Date(e.created_at).toLocaleString("en-GB")}</span>
                        </li>
                    ))}
                </ul>
            )}
        </div>
    );
}
