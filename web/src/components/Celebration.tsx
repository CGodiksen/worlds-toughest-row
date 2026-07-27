import { useEffect, useRef } from "react";
import confetti from "canvas-confetti";

export function Celebration({ done }: { done: boolean }) {
    const firedRef = useRef(false);

    useEffect(() => {
        if (!done) {
            firedRef.current = false;
            return;
        }
        if (firedRef.current) return;
        firedRef.current = true;

        const end = Date.now() + 10000;
        const rand = (min: number, max: number) => Math.random() * (max - min) + min;

        const frame = () => {
            confetti({
                particleCount: 30,
                startVelocity: 30,
                spread: 360,
                ticks: 60,
                origin: { x: rand(0.1, 0.9), y: rand(0.1, 0.5) },
            });
            if (Date.now() < end) setTimeout(frame, 250);
        };
        frame();
    }, [done]);

    return null;
}
