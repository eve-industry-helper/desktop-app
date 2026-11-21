import type { ParsedFitItem } from '$lib/types/frontend';

export interface EftModule {
    name: string;
    charge?: string;
    offline: boolean;
}

export interface EftStack {
    name: string;
    quantity: number;
}

export interface ParsedEftFit {
    hull: string | null;
    fitName: string;
    lows: EftModule[];
    meds: EftModule[];
    highs: EftModule[];
    rigs: string[];
    subsystems: string[];
    services: string[];
    drones: EftStack[];
    cargo: EftStack[];
    allItems: ParsedFitItem[];
}

const EMPTY_SLOT_REGEX = /^\[Empty\s+(low|med|high|rig|service)\s+slot]$/i;
const HEADER_REGEX = /^\[([^,\]]+)(?:,\s*([^\]]+))?]$/;
const OFFLINE_REGEX = /\s+\/offline$/i;
const STACK_REGEX = /^(.+?)\s+x(\d+)$/i;

function parseModuleLine(line: string): EftModule {
    const offline = OFFLINE_REGEX.test(line);
    const cleaned = line.replace(OFFLINE_REGEX, '').trim();
    const commaIdx = cleaned.indexOf(',');

    if (commaIdx !== -1) {
        return {
            name: cleaned.slice(0, commaIdx).trim(),
            charge: cleaned.slice(commaIdx + 1).trim() || undefined,
            offline
        };
    }

    return { name: cleaned, offline };
}

function parseStackLine(line: string): EftStack {
    const match = line.match(STACK_REGEX);
    if (match) {
        return {
            name: match[1].trim(),
            quantity: parseInt(match[2], 10)
        };
    }
    return { name: line, quantity: 1 };
}

export function parseEftFit(rawText: string): ParsedEftFit {
    const lines = rawText.replace(/\r\n/g, '\n').trim().split('\n');

    const result: ParsedEftFit = {
        hull: null,
        fitName: 'Imported Fit',
        lows: [],
        meds: [],
        highs: [],
        rigs: [],
        subsystems: [],
        services: [],
        drones: [],
        cargo: [],
        allItems: []
    };

    if (lines.length === 0 || !lines[0].trim()) {
        return result;
    }

    let startIndex = 0;
    const headerMatch = lines[0].trim().match(HEADER_REGEX);
    if (headerMatch && !EMPTY_SLOT_REGEX.test(lines[0].trim())) {
        result.hull = headerMatch[1].trim();
        if (headerMatch[2]) {
            result.fitName = headerMatch[2].trim();
        }
        startIndex = 1;
    }

    // Section indices:
    // 0: lows, 1: meds, 2: highs, 3: rigs, 4: subsystems, 5: services, 6: drones, 7: cargo
    let section = 0;
    let emptyLineRun = 0;

    for (let i = startIndex; i < lines.length; i++) {
        const line = lines[i].trim();

        if (!line) {
            emptyLineRun++;
            continue;
        }

        if (emptyLineRun > 0) {
            // Sections 2–7 (indices 0–5) are separated by 1 empty line
            while (section < 5 && emptyLineRun > 0) {
                section++;
                emptyLineRun--;
            }
            // Sections 7–9 (indices 5–7) are separated by 2 empty lines
            while (section >= 5 && section < 7 && emptyLineRun >= 2) {
                section++;
                emptyLineRun -= 2;
            }
            // Fallback for hand-edited fits with a single blank line between drones and cargo
            if (section >= 5 && section < 7 && emptyLineRun === 1) {
                section++;
            }
            emptyLineRun = 0;
        }

        if (EMPTY_SLOT_REGEX.test(line)) {
            continue;
        }

        switch (section) {
            case 0:
                result.lows.push(parseModuleLine(line));
                break;
            case 1:
                result.meds.push(parseModuleLine(line));
                break;
            case 2:
                result.highs.push(parseModuleLine(line));
                break;
            case 3:
                result.rigs.push(parseModuleLine(line).name);
                break;
            case 4:
                result.subsystems.push(parseModuleLine(line).name);
                break;
            case 5:
                result.services.push(parseModuleLine(line).name);
                break;
            case 6:
                result.drones.push(parseStackLine(line));
                break;
            default:
                result.cargo.push(parseStackLine(line));
                break;
        }
    }

    // Build aggregated item list for industry plans
    const counts = new Map<string, number>();
    const addCount = (name: string, qty = 1) => {
        if (!name) return;
        counts.set(name, (counts.get(name) ?? 0) + qty);
    };

    if (result.hull) addCount(result.hull, 1);
    result.lows.forEach(m => addCount(m.name));
    result.meds.forEach(m => addCount(m.name));
    result.highs.forEach(m => addCount(m.name));
    result.rigs.forEach(r => addCount(r));
    result.subsystems.forEach(s => addCount(s));
    result.services.forEach(s => addCount(s));
    result.drones.forEach(d => addCount(d.name, d.quantity));
    result.cargo.forEach(c => addCount(c.name, c.quantity));

    result.allItems = Array.from(counts.entries()).map(([name, quantity]) => ({
        name,
        quantity
    }));

    return result;
}