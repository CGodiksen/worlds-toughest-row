import { overlay } from "../styles/overlay";

interface Props {
    onReset: () => void;
    busy: boolean;
}

export function FinishBanner({ onReset, busy }: Props) {
    return (
        <div
            style={overlay({
                top: 24,
                left: "50%",
                transform: "translateX(-50%)",
                padding: "20px 28px",
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                gap: 12,
                textAlign: "center",
            })}
        >
            <div style={{ fontSize: "1.4rem", fontWeight: 700 }}>🎉 You made it!</div>
            <div style={{ fontSize: ".85rem", opacity: 0.7 }}>La Gomera → Antigua complete</div>
            <button
                onClick={onReset}
                disabled={busy}
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
                {busy ? "…" : "Reset"}
            </button>
        </div>
    );
}
