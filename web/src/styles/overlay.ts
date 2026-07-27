import type { CSSProperties } from "react";

// The shared "Google Maps floating card" look.
export function overlay(extra: CSSProperties): CSSProperties {
    return {
        position: "absolute",
        zIndex: 10,
        background: "rgba(255, 255, 255, 0.92)",
        backdropFilter: "blur(8px)",
        borderRadius: 12,
        boxShadow: "0 2px 12px rgba(0, 0, 0, 0.15)",
        ...extra,
    };
}
