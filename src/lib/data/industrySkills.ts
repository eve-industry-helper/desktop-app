import type { IndustrySkillDef, SkillGroup } from '#lib/types/frontend';

export const SKILL_GROUP_ORDER: SkillGroup[] = [
    'Production',
    'Science',
    'Reactions',
    'Resource Processing',
    'Trade'
];

export const INDUSTRY_SKILLS: IndustrySkillDef[] = [
    // --- Production ---
    { id: 3380,  name: 'Industry',                              group: 'Production', modifier: { type: 'manufacturing_time',             valuePerLevel: -0.04, unit: 'percent', label: 'manufacturing time' } },
    { id: 3388,  name: 'Advanced Industry',                     group: 'Production', modifier: { type: 'all_industry_time',              valuePerLevel: -0.03, unit: 'percent', label: 'all industry job time' } },
    { id: 3387,  name: 'Mass Production',                       group: 'Production', modifier: { type: 'manufacturing_slots',            valuePerLevel: 1,     unit: 'flat',    label: 'manufacturing slot' } },
    { id: 24625, name: 'Advanced Mass Production',              group: 'Production', modifier: { type: 'manufacturing_slots',            valuePerLevel: 1,     unit: 'flat',    label: 'manufacturing slot' } },
    { id: 24268, name: 'Supply Chain Management',               group: 'Production', modifier: { type: 'remote_range',                   valuePerLevel: 5,     unit: 'jumps',   label: 'remote manufacturing range' } },
    { id: 3395,  name: 'Advanced Small Ship Construction',      group: 'Production', modifier: { type: 'specialized_manufacturing_time', valuePerLevel: -0.01, unit: 'percent', label: 'T2 small ship build time' } },
    { id: 3397,  name: 'Advanced Medium Ship Construction',     group: 'Production', modifier: { type: 'specialized_manufacturing_time', valuePerLevel: -0.01, unit: 'percent', label: 'T2 medium ship build time' } },
    { id: 3398,  name: 'Advanced Large Ship Construction',      group: 'Production', modifier: { type: 'specialized_manufacturing_time', valuePerLevel: -0.01, unit: 'percent', label: 'T2 large ship build time' } },
    { id: 3396,  name: 'Advanced Industrial Ship Construction', group: 'Production', modifier: { type: 'specialized_manufacturing_time', valuePerLevel: -0.01, unit: 'percent', label: 'T2 industrial build time' } },
    { id: 22242, name: 'Capital Ship Construction',             group: 'Production', modifier: { type: 'specialized_manufacturing_time', valuePerLevel: -0.01, unit: 'percent', label: 'capital ship build time' } },
    { id: 26224, name: 'Drug Manufacturing',                    group: 'Production', modifier: { type: 'specialized_manufacturing_time', valuePerLevel: 0,     unit: 'flat',    label: 'booster manufacturing unlock' } },

    // --- Science (Research & Invention) ---
    { id: 3402,  name: 'Science',                         group: 'Science', modifier: { type: 'copy_time',        valuePerLevel: -0.05,  unit: 'percent', label: 'blueprint copy time' } },
    { id: 3403,  name: 'Research',                        group: 'Science', modifier: { type: 'te_research_time', valuePerLevel: -0.05,  unit: 'percent', label: 'TE research time' } },
    { id: 3409,  name: 'Metallurgy',                      group: 'Science', modifier: { type: 'me_research_time', valuePerLevel: -0.05,  unit: 'percent', label: 'ME research time' } },
    { id: 3406,  name: 'Laboratory Operation',            group: 'Science', modifier: { type: 'science_slots',    valuePerLevel: 1,      unit: 'flat',    label: 'science slot' } },
    { id: 24624, name: 'Advanced Laboratory Operation',   group: 'Science', modifier: { type: 'science_slots',    valuePerLevel: 1,      unit: 'flat',    label: 'science slot' } },
    { id: 24270, name: 'Scientific Networking',           group: 'Science', modifier: { type: 'remote_range',     valuePerLevel: 5,      unit: 'jumps',   label: 'remote science range' } },
    { id: 3408,  name: 'Research Project Management',     group: 'Science', modifier: { type: 'science_slots',    valuePerLevel: 1,      unit: 'flat',    label: 'research agent' } },
    { id: 23087, name: 'Amarr Encryption Methods',        group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.025,  unit: 'percent', label: 'Amarr invention chance' } },
    { id: 21790, name: 'Caldari Encryption Methods',      group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.025,  unit: 'percent', label: 'Caldari invention chance' } },
    { id: 23121, name: 'Gallente Encryption Methods',     group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.025,  unit: 'percent', label: 'Gallente invention chance' } },
    { id: 21791, name: 'Minmatar Encryption Methods',     group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.025,  unit: 'percent', label: 'Minmatar invention chance' } },
    { id: 30324, name: 'Sleeper Encryption Methods',      group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.025,  unit: 'percent', label: 'T3 invention chance' } },
    { id: 11444, name: 'Amarrian Starship Engineering',   group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.0333, unit: 'percent', label: 'invention chance' } },
    { id: 11454, name: 'Caldari Starship Engineering',    group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.0333, unit: 'percent', label: 'invention chance' } },
    { id: 11450, name: 'Gallentean Starship Engineering', group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.0333, unit: 'percent', label: 'invention chance' } },
    { id: 11445, name: 'Minmatar Starship Engineering',   group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.0333, unit: 'percent', label: 'invention chance' } },
    { id: 11452, name: 'Mechanical Engineering',          group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.0333, unit: 'percent', label: 'invention chance' } },
    { id: 11453, name: 'Electronic Engineering',          group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.0333, unit: 'percent', label: 'invention chance' } },
    { id: 11448, name: 'Plasma Physics',                  group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.0333, unit: 'percent', label: 'invention chance' } },
    { id: 11455, name: 'Quantum Physics',                 group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.0333, unit: 'percent', label: 'invention chance' } },
    { id: 11446, name: 'Graviton Physics',                group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.0333, unit: 'percent', label: 'invention chance' } },
    { id: 11443, name: 'Hydromagnetic Physics',           group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.0333, unit: 'percent', label: 'invention chance' } },
    { id: 11447, name: 'Laser Physics',                   group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.0333, unit: 'percent', label: 'invention chance' } },
    { id: 11451, name: 'Nuclear Physics',                 group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.0333, unit: 'percent', label: 'invention chance' } },
    { id: 11449, name: 'Electromagnetic Physics',         group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.0333, unit: 'percent', label: 'invention chance' } },
    { id: 11433, name: 'High Energy Physics',             group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.0333, unit: 'percent', label: 'invention chance' } },
    { id: 11529, name: 'Molecular Engineering',           group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.0333, unit: 'percent', label: 'invention chance' } },
    { id: 11442, name: 'Nanite Engineering',              group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.0333, unit: 'percent', label: 'invention chance' } },
    { id: 11441, name: 'Rocket Science',                  group: 'Science', modifier: { type: 'invention_chance', valuePerLevel: 0.0333, unit: 'percent', label: 'invention chance' } },

    // --- Reactions ---
    { id: 45746, name: 'Reactions',               group: 'Reactions', modifier: { type: 'reaction_time',  valuePerLevel: -0.04, unit: 'percent', label: 'reaction time' } },
    { id: 45748, name: 'Mass Reactions',          group: 'Reactions', modifier: { type: 'reaction_slots', valuePerLevel: 1,     unit: 'flat',    label: 'reaction slot' } },
    { id: 45749, name: 'Advanced Mass Reactions', group: 'Reactions', modifier: { type: 'reaction_slots', valuePerLevel: 1,     unit: 'flat',    label: 'reaction slot' } },
    { id: 45750, name: 'Remote Reactions',        group: 'Reactions', modifier: { type: 'remote_range',   valuePerLevel: 5,     unit: 'jumps',   label: 'remote reaction range' } },

    // --- Resource Processing ---
    { id: 3385,  name: 'Reprocessing',                    group: 'Resource Processing', modifier: { type: 'reprocessing_yield', valuePerLevel: 0.03, unit: 'percent', label: 'ore & ice yield' } },
    { id: 3389,  name: 'Reprocessing Efficiency',         group: 'Resource Processing', modifier: { type: 'reprocessing_yield', valuePerLevel: 0.02, unit: 'percent', label: 'ore & ice yield' } },
    { id: 12196, name: 'Scrapmetal Processing',           group: 'Resource Processing', modifier: { type: 'scrapmetal_yield',   valuePerLevel: 0.02, unit: 'percent', label: 'scrapmetal yield' } },
    { id: 18025, name: 'Ice Processing',                  group: 'Resource Processing', modifier: { type: 'ore_spec_yield',     valuePerLevel: 0.02, unit: 'percent', label: 'ice yield' } },
    { id: 60377, name: 'Simple Ore Processing',           group: 'Resource Processing', modifier: { type: 'ore_spec_yield',     valuePerLevel: 0.02, unit: 'percent', label: 'simple ore yield' } },
    { id: 60378, name: 'Coherent Ore Processing',         group: 'Resource Processing', modifier: { type: 'ore_spec_yield',     valuePerLevel: 0.02, unit: 'percent', label: 'coherent ore yield' } },
    { id: 60379, name: 'Variegated Ore Processing',       group: 'Resource Processing', modifier: { type: 'ore_spec_yield',     valuePerLevel: 0.02, unit: 'percent', label: 'variegated ore yield' } },
    { id: 60380, name: 'Complex Ore Processing',          group: 'Resource Processing', modifier: { type: 'ore_spec_yield',     valuePerLevel: 0.02, unit: 'percent', label: 'complex ore yield' } },
    { id: 60381, name: 'Abyssal Ore Processing',          group: 'Resource Processing', modifier: { type: 'ore_spec_yield',     valuePerLevel: 0.02, unit: 'percent', label: 'abyssal ore yield' } },
    { id: 12189, name: 'Mercoxit Ore Processing',         group: 'Resource Processing', modifier: { type: 'ore_spec_yield',     valuePerLevel: 0.02, unit: 'percent', label: 'mercoxit yield' } },
    { id: 46152, name: 'Ubiquitous Moon Ore Processing',  group: 'Resource Processing', modifier: { type: 'ore_spec_yield',     valuePerLevel: 0.02, unit: 'percent', label: 'R4 moon ore yield' } },
    { id: 46153, name: 'Common Moon Ore Processing',      group: 'Resource Processing', modifier: { type: 'ore_spec_yield',     valuePerLevel: 0.02, unit: 'percent', label: 'R8 moon ore yield' } },
    { id: 46154, name: 'Uncommon Moon Ore Processing',    group: 'Resource Processing', modifier: { type: 'ore_spec_yield',     valuePerLevel: 0.02, unit: 'percent', label: 'R16 moon ore yield' } },
    { id: 46155, name: 'Rare Moon Ore Processing',        group: 'Resource Processing', modifier: { type: 'ore_spec_yield',     valuePerLevel: 0.02, unit: 'percent', label: 'R32 moon ore yield' } },
    { id: 46156, name: 'Exceptional Moon Ore Processing', group: 'Resource Processing', modifier: { type: 'ore_spec_yield',     valuePerLevel: 0.02, unit: 'percent', label: 'R64 moon ore yield' } },

    // --- Trade ---
    { id: 16622, name: 'Accounting',                group: 'Trade', modifier: { type: 'sales_tax_reduction',  valuePerLevel: -0.11,  unit: 'percent', label: 'sales tax' } },
    { id: 3446,  name: 'Broker Relations',          group: 'Trade', modifier: { type: 'broker_fee_reduction', valuePerLevel: -0.003, unit: 'percent', label: 'broker fee' } },
    { id: 16597, name: 'Advanced Broker Relations', group: 'Trade', modifier: { type: 'relist_fee_reduction', valuePerLevel: -0.06,  unit: 'percent', label: 'relist fee' } },
    { id: 25235, name: 'Customs Code Expertise',    group: 'Trade', modifier: { type: 'poco_tax_reduction',   valuePerLevel: -0.10,  unit: 'percent', label: 'NPC POCO tax' } },
    { id: 3443,  name: 'Trade',                     group: 'Trade', modifier: { type: 'market_orders',        valuePerLevel: 4,      unit: 'flat',    label: 'market orders' } },
    { id: 3444,  name: 'Retail',                    group: 'Trade', modifier: { type: 'market_orders',        valuePerLevel: 8,      unit: 'flat',    label: 'market orders' } },
    { id: 16596, name: 'Wholesale',                 group: 'Trade', modifier: { type: 'market_orders',        valuePerLevel: 16,     unit: 'flat',    label: 'market orders' } },
    { id: 18580, name: 'Tycoon',                    group: 'Trade', modifier: { type: 'market_orders',        valuePerLevel: 32,     unit: 'flat',    label: 'market orders' } },
    { id: 16598, name: 'Marketing',                 group: 'Trade', modifier: { type: 'remote_range',         valuePerLevel: 5,      unit: 'jumps',   label: 'remote sell range' } },
    { id: 16594, name: 'Procurement',               group: 'Trade', modifier: { type: 'remote_range',         valuePerLevel: 5,      unit: 'jumps',   label: 'remote buy range' } },
    { id: 16595, name: 'Daytrading',                group: 'Trade', modifier: { type: 'remote_range',         valuePerLevel: 5,      unit: 'jumps',   label: 'remote order modify range' } },
    { id: 3447,  name: 'Visibility',                group: 'Trade', modifier: { type: 'remote_range',         valuePerLevel: 5,      unit: 'jumps',   label: 'remote buy order range' } }
];