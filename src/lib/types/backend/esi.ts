export interface CharacterProfile {
    character_id: number;
    character_name: string;
    portrait_url: string;
}

export interface Asset {
    item_id: number;
    type_id: number;
    location_id: number;
    location_flag: string;
    location_type: string;
    quantity: number;
    is_singleton: boolean;
    is_blueprint_copy: boolean | null;
}

export interface UniverseIdMatch {
    id: number;
    name: string;
}

export interface UniverseName {
    id: number;
    name: string;
    category: string;
}

export interface Blueprint {
    item_id: number;
    type_id: number;
    location_id: number;
    location_flag: string;
    quantity: number;
    time_efficiency: number;
    material_efficiency: number;
    runs: number;
}

export interface IndustryJob {
    job_id: number;
    installer_id: number;
    facility_id: number;
    station_id: number;
    activity_id: number;
    blueprint_id: number;
    blueprint_type_id: number;
    blueprint_location_id: number;
    output_location_id: number;
    runs: number;
    cost: number | null;
    licensed_runs: number | null;
    probability: number | null;
    product_type_id: number | null;
    status: string;
    duration: number;
    start_date: string;
    end_date: string;
    pause_date: string | null;
    completed_date: string | null;
    completed_character_id: number | null;
    successful_runs: number | null;
}

export interface Position {
    x: number;
    y: number;
    z: number;
}

export interface StructureInfo {
    name: string;
    owner_id: number;
    position: Position | null;
    solar_system_id: number;
    type_id: number | null;
}

export interface CharacterSkill {
    skill_id: number;
    active_skill_level: number;
    trained_skill_level: number;
    skillpoints_in_skill: number;
}

export interface CharacterSkillsResponse {
    skills: CharacterSkill[];
    total_sp: number;
    unallocated_sp?: number;
}