import type { Entry } from "./bindings/Entry";

const DAY = 86_400_000;

function startOfDay(t: number): number {
    const d = new Date(t);
    d.setHours(0, 0, 0, 0);
    return d.getTime();
}

// Timestamp of the Monday that starts the week containing `t`.
function weekStart(t: number): number {
    const midnight = startOfDay(t);
    const dow = (new Date(midnight).getDay() + 6) % 7; // 0 = Monday.
    return midnight - dow * DAY;
}

function dayKey(d: Date): string {
    return `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}`;
}
function monthKey(d: Date): string {
    return `${d.getFullYear()}-${d.getMonth()}`;
}

export interface Stats {
    bestDay: number;
    today: number;
    bestWeek: number;
    thisWeek: number;
    bestMonth: number;
    thisMonth: number;
    bestSession: number;
    lastSession: number;
    longestStreak: number;
    currentStreak: number;
}

export function computeStats(entries: Entry[]): Stats {
    const now = new Date();
    const todayKey = dayKey(now);
    const thisWeekKey = weekStart(now.getTime());
    const thisMonthKey = monthKey(now);

    const byDay = new Map<string, number>();
    const byWeek = new Map<number, number>();
    const byMonth = new Map<string, number>();
    let bestSession = 0;

    for (const e of entries) {
        const d = new Date(e.created_at);
        byDay.set(dayKey(d), (byDay.get(dayKey(d)) ?? 0) + e.meters);
        byWeek.set(weekStart(d.getTime()), (byWeek.get(weekStart(d.getTime())) ?? 0) + e.meters);
        byMonth.set(monthKey(d), (byMonth.get(monthKey(d)) ?? 0) + e.meters);
        if (e.meters > bestSession) bestSession = e.meters;
    }

    // Active local days, ascending, for streak walking.
    const days = [...byDay.keys()]
        .map((key) => {
            const [y, m, dd] = key.split("-").map(Number);
            return new Date(y, m, dd).getTime();
        })
        .sort((a, b) => a - b);

    let longestStreak = 0;
    let run = 0;
    for (let i = 0; i < days.length; i++) {
        // Round the diff so daylight-saving hour shifts do not break a run.
        run = i > 0 && Math.round((days[i] - days[i - 1]) / DAY) === 1 ? run + 1 : 1;
        if (run > longestStreak) longestStreak = run;
    }

    let currentStreak = 0;
    if (days.length) {
        // Only counts if the last active day was today or yesterday.
        const gap = Math.round((startOfDay(now.getTime()) - days[days.length - 1]) / DAY);
        if (gap <= 1) {
            currentStreak = 1;
            for (let i = days.length - 1; i > 0; i--) {
                if (Math.round((days[i] - days[i - 1]) / DAY) === 1) currentStreak++;
                else break;
            }
        }
    }

    // Entries arrive newest-first, so entries[0] is the latest session.
    const lastSession = entries.length ? entries[0].meters : 0;

    return {
        bestDay: Math.max(0, ...byDay.values()),
        today: byDay.get(todayKey) ?? 0,
        bestWeek: Math.max(0, ...byWeek.values()),
        thisWeek: byWeek.get(thisWeekKey) ?? 0,
        bestMonth: Math.max(0, ...byMonth.values()),
        thisMonth: byMonth.get(thisMonthKey) ?? 0,
        bestSession,
        lastSession,
        longestStreak,
        currentStreak,
    };
}
