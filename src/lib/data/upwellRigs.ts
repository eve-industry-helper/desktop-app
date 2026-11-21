import type { RigSize } from './structures';

export interface RigDef {
    name: string;
    size: RigSize;
    category: 'ec' | 'refinery';
}

export const UPWELL_RIGS: RigDef[] = [
    // Medium Engineering Complex (Raitaru)
    { name: 'Standup M-Set Equipment Manufacturing Material Efficiency I', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Equipment Manufacturing Material Efficiency II', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Equipment Manufacturing Time Efficiency I', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Equipment Manufacturing Time Efficiency II', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Ammunition Manufacturing Material Efficiency I', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Ammunition Manufacturing Material Efficiency II', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Drone and Fighter Manufacturing Material Efficiency I', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Drone and Fighter Manufacturing Material Efficiency II', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Basic Small Ship Manufacturing Material Efficiency I', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Basic Small Ship Manufacturing Material Efficiency II', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Basic Medium Ship Manufacturing Material Efficiency I', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Basic Medium Ship Manufacturing Material Efficiency II', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Advanced Small Ship Manufacturing Material Efficiency I', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Advanced Small Ship Manufacturing Material Efficiency II', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Advanced Medium Ship Manufacturing Material Efficiency I', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Advanced Medium Ship Manufacturing Material Efficiency II', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Advanced Component Manufacturing Material Efficiency I', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Advanced Component Manufacturing Material Efficiency II', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Structure Manufacturing Material Efficiency I', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Structure Manufacturing Material Efficiency II', size: 'M', category: 'ec' },
    { name: 'Standup M-Set ME Research Time Efficiency I', size: 'M', category: 'ec' },
    { name: 'Standup M-Set TE Research Time Efficiency I', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Blueprint Copy Time Efficiency I', size: 'M', category: 'ec' },
    { name: 'Standup M-Set Invention Cost Optimization I', size: 'M', category: 'ec' },

    // Large Engineering Complex (Azbel)
    { name: 'Standup L-Set Equipment Manufacturing Efficiency I', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Equipment Manufacturing Efficiency II', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Ammunition Manufacturing Efficiency I', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Ammunition Manufacturing Efficiency II', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Drone and Fighter Manufacturing Efficiency I', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Drone and Fighter Manufacturing Efficiency II', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Basic Small Ship Manufacturing Efficiency I', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Basic Small Ship Manufacturing Efficiency II', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Basic Medium Ship Manufacturing Efficiency I', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Basic Medium Ship Manufacturing Efficiency II', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Basic Large Ship Manufacturing Efficiency I', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Basic Large Ship Manufacturing Efficiency II', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Advanced Small Ship Manufacturing Efficiency I', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Advanced Small Ship Manufacturing Efficiency II', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Advanced Medium Ship Manufacturing Efficiency I', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Advanced Medium Ship Manufacturing Efficiency II', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Advanced Large Ship Manufacturing Efficiency I', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Advanced Large Ship Manufacturing Efficiency II', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Capital Ship Manufacturing Efficiency I', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Capital Ship Manufacturing Efficiency II', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Advanced Component Manufacturing Efficiency I', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Advanced Component Manufacturing Efficiency II', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Structure Manufacturing Efficiency I', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Structure Manufacturing Efficiency II', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Laboratory Optimization I', size: 'L', category: 'ec' },
    { name: 'Standup L-Set Laboratory Optimization II', size: 'L', category: 'ec' },

    // X-Large Engineering Complex (Sotiyo)
    { name: 'Standup XL-Set Equipment and Consumable Manufacturing Efficiency I', size: 'XL', category: 'ec' },
    { name: 'Standup XL-Set Equipment and Consumable Manufacturing Efficiency II', size: 'XL', category: 'ec' },
    { name: 'Standup XL-Set Ship Manufacturing Efficiency I', size: 'XL', category: 'ec' },
    { name: 'Standup XL-Set Ship Manufacturing Efficiency II', size: 'XL', category: 'ec' },
    { name: 'Standup XL-Set Structure and Component Manufacturing Efficiency I', size: 'XL', category: 'ec' },
    { name: 'Standup XL-Set Structure and Component Manufacturing Efficiency II', size: 'XL', category: 'ec' },
    { name: 'Standup XL-Set Laboratory Optimization I', size: 'XL', category: 'ec' },
    { name: 'Standup XL-Set Laboratory Optimization II', size: 'XL', category: 'ec' },

    // Medium Refinery (Athanor)
    { name: 'Standup M-Set Reactor Material Efficiency I', size: 'M', category: 'refinery' },
    { name: 'Standup M-Set Reactor Material Efficiency II', size: 'M', category: 'refinery' },
    { name: 'Standup M-Set Reactor Time Efficiency I', size: 'M', category: 'refinery' },
    { name: 'Standup M-Set Reactor Time Efficiency II', size: 'M', category: 'refinery' },
    { name: 'Standup M-Set Biochemical Reactor Material Efficiency I', size: 'M', category: 'refinery' },
    { name: 'Standup M-Set Biochemical Reactor Material Efficiency II', size: 'M', category: 'refinery' },
    { name: 'Standup M-Set Composite Reactor Material Efficiency I', size: 'M', category: 'refinery' },
    { name: 'Standup M-Set Composite Reactor Material Efficiency II', size: 'M', category: 'refinery' },
    { name: 'Standup M-Set Hybrid Reactor Material Efficiency I', size: 'M', category: 'refinery' },
    { name: 'Standup M-Set Hybrid Reactor Material Efficiency II', size: 'M', category: 'refinery' },
    { name: 'Standup M-Set Moon Ore Grading Processor I', size: 'M', category: 'refinery' },
    { name: 'Standup M-Set Moon Ore Grading Processor II', size: 'M', category: 'refinery' },
    { name: 'Standup M-Set Asteroid Ore Grading Processor I', size: 'M', category: 'refinery' },
    { name: 'Standup M-Set Asteroid Ore Grading Processor II', size: 'M', category: 'refinery' },
    { name: 'Standup M-Set Ice Grading Processor I', size: 'M', category: 'refinery' },
    { name: 'Standup M-Set Ice Grading Processor II', size: 'M', category: 'refinery' },

    // Large Refinery (Tatara)
    { name: 'Standup L-Set Reactor Efficiency I', size: 'L', category: 'refinery' },
    { name: 'Standup L-Set Reactor Efficiency II', size: 'L', category: 'refinery' },
    { name: 'Standup L-Set Reprocessing Efficiency I', size: 'L', category: 'refinery' },
    { name: 'Standup L-Set Reprocessing Efficiency II', size: 'L', category: 'refinery' }
];