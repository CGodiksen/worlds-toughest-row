import { useState } from "react";
import { overlay } from "../styles/overlay";

interface Props {
    onAdvance: (meters?: number) => void;
    busy: boolean;
}

export function AdvanceBar({ onAdvance, busy }: Props) {
    const [meters, setMeters] = useState("");

    const submit = () => onAdvance(meters ? Number(meters) : undefined);

    return (
        <div
            style={overlay({
                bottom: 24,
                left: "50%",
                transform: "translateX(-50%)",
                padding: 12,
                display: "flex",
                gap: 8,
                alignItems: "center",
            })}
        >
            <input
                type="number"
                placeholder="500"
                value={meters}
                onChange={(e) => setMeters(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && submit()}
                style={{ width: 80, padding: "8px 10px", border: "1px solid #d1d5db", borderRadius: 8 }}
            />
            <span style={{ fontSize: ".85rem", opacity: 0.6 }}>m</span>
            <button
                disabled={busy}
                onClick={submit}
                style={{
                    padding: "8px 18px",
                    border: 0,
                    borderRadius: 8,
                    background: "#2563eb",
                    color: "#fff",
                    fontWeight: 600,
                    cursor: busy ? "default" : "pointer",
                }}
            >
                {busy ? "…" : "Row"}
            </button>
        </div>
    );
}
