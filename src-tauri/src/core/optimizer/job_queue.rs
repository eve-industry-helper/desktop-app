use std::cmp::Reverse;
use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};

use crate::core::models::sde::BlueprintActivity;
use crate::core::optimizer::dag::{NodeDecision, OptimizedNode};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobSummary {
    pub blueprint_type_id: u32,
    pub product_type_id: u32,
    pub product_name: String,
    pub activity: BlueprintActivity,
    pub runs: u64,
    pub output_qty: u64,
    pub job_cost: f64,
    pub duration_secs: u64,
    /// 1-based ASAP execution stage (Step 1 has no crafted prerequisites).
    pub stage: u32,
    /// Blueprint type IDs of crafted prerequisite jobs that must finish before this job starts.
    pub prerequisite_blueprint_ids: Vec<u32>,
    /// Product names of crafted downstream jobs that consume this job's output (empty for final outputs).
    pub used_by: Vec<String>,
}

/// Priority rank for sorting jobs within the same execution stage.
/// Lower values appear earlier. Designed so future blueprint prep activities
/// (e.g. Copying / Invention) can slot in at rank 0 ahead of Reactions and Manufacturing.
pub(crate) fn activity_sort_rank(activity: BlueprintActivity) -> u8 {
    match activity {
        BlueprintActivity::Reaction => 1,
        BlueprintActivity::Manufacturing => 2,
    }
}

fn collect_node_jobs(
    node: &OptimizedNode,
    jobs: &mut HashMap<u32, JobSummary>,
    prereqs: &mut HashMap<u32, HashSet<u32>>,
) {
    if node.decision != NodeDecision::Craft {
        return;
    }

    if let (Some(bp_id), Some(activity)) = (node.blueprint_type_id, node.activity) {
        let job = jobs.entry(bp_id).or_insert_with(|| JobSummary {
            blueprint_type_id: bp_id,
            product_type_id: node.type_id,
            product_name: node.name.clone(),
            activity,
            runs: 0,
            output_qty: 0,
            job_cost: 0.0,
            duration_secs: 0,
            stage: 1,
            prerequisite_blueprint_ids: Vec::new(),
            used_by: Vec::new(),
        });
        job.runs += node.runs;
        job.output_qty += node.output_qty;
        job.job_cost += node.job_cost;
        job.duration_secs += node.job_duration_secs;

        let bp_prereqs = prereqs.entry(bp_id).or_default();
        for child in &node.children {
            if child.decision == NodeDecision::Craft
                && let Some(child_bp_id) = child.blueprint_type_id
                && child_bp_id != bp_id
            {
                bp_prereqs.insert(child_bp_id);
            }
        }
    }

    for child in &node.children {
        collect_node_jobs(child, jobs, prereqs);
    }
}

/// Consolidates all crafted jobs across the evaluated DAG trees and returns them
/// ordered into ASAP (earliest-start) dependency stages.
pub fn build_ordered_job_queue(nodes: &[OptimizedNode]) -> Vec<JobSummary> {
    let mut jobs_map: HashMap<u32, JobSummary> = HashMap::new();
    let mut prereqs: HashMap<u32, HashSet<u32>> = HashMap::new();

    for node in nodes {
        collect_node_jobs(node, &mut jobs_map, &mut prereqs);
    }

    order_job_queue(jobs_map, &prereqs)
}

/// Orders a consolidated map of crafted jobs using Kahn's topological sort with ASAP stage levelling.
pub(crate) fn order_job_queue(
    mut jobs_map: HashMap<u32, JobSummary>,
    raw_prereqs: &HashMap<u32, HashSet<u32>>,
) -> Vec<JobSummary> {
    if jobs_map.is_empty() {
        return Vec::new();
    }

    let valid_ids: HashSet<u32> = jobs_map.keys().copied().collect();
    let mut prereqs: HashMap<u32, HashSet<u32>> = HashMap::with_capacity(valid_ids.len());
    let mut dependents: HashMap<u32, HashSet<u32>> = HashMap::with_capacity(valid_ids.len());
    let mut in_degree: HashMap<u32, usize> = HashMap::with_capacity(valid_ids.len());

    for &bp_id in &valid_ids {
        let filtered: HashSet<u32> = raw_prereqs
            .get(&bp_id)
            .map(|set| {
                set.iter()
                    .copied()
                    .filter(|p| *p != bp_id && valid_ids.contains(p))
                    .collect()
            })
            .unwrap_or_default();

        in_degree.insert(bp_id, filtered.len());
        for &p in &filtered {
            dependents.entry(p).or_default().insert(bp_id);
        }
        prereqs.insert(bp_id, filtered);
    }

    let mut stage_map: HashMap<u32, u32> = HashMap::with_capacity(valid_ids.len());
    let mut ready: VecDeque<u32> = VecDeque::new();

    let mut initial_ready: Vec<u32> = in_degree
        .iter()
        .filter_map(|(&bp_id, &deg)| (deg == 0).then_some(bp_id))
        .collect();
    initial_ready.sort_unstable();

    for bp_id in initial_ready {
        stage_map.insert(bp_id, 1);
        ready.push_back(bp_id);
    }

    while let Some(u) = ready.pop_front() {
        let u_stage = stage_map[&u];
        if let Some(consumers) = dependents.get(&u) {
            let mut sorted_consumers: Vec<u32> = consumers.iter().copied().collect();
            sorted_consumers.sort_unstable();

            for v in sorted_consumers {
                let candidate_stage = u_stage + 1;
                let entry = stage_map.entry(v).or_insert(candidate_stage);
                if candidate_stage > *entry {
                    *entry = candidate_stage;
                }

                if let Some(deg) = in_degree.get_mut(&v) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        ready.push_back(v);
                    }
                }
            }
        }
    }

    // Defensive fallback if malformed/cyclic data left any nodes unvisited
    if stage_map.len() < valid_ids.len() {
        let fallback_stage = stage_map.values().copied().max().unwrap_or(0) + 1;
        for &bp_id in &valid_ids {
            stage_map.entry(bp_id).or_insert(fallback_stage);
        }
    }

    let name_by_bp: HashMap<u32, String> = jobs_map
        .iter()
        .map(|(&bp_id, job)| (bp_id, job.product_name.clone()))
        .collect();

    let mut queue: Vec<JobSummary> = jobs_map
        .drain()
        .map(|(bp_id, mut job)| {
            job.stage = stage_map.get(&bp_id).copied().unwrap_or(1);

            let mut bp_prereqs: Vec<u32> = prereqs
                .get(&bp_id)
                .map(|s| s.iter().copied().collect())
                .unwrap_or_default();
            bp_prereqs.sort_unstable();
            job.prerequisite_blueprint_ids = bp_prereqs;

            let mut used_by: Vec<String> = dependents
                .get(&bp_id)
                .map(|consumers| {
                    consumers
                        .iter()
                        .filter_map(|c| name_by_bp.get(c).cloned())
                        .collect()
                })
                .unwrap_or_default();
            used_by.sort();
            used_by.dedup();
            job.used_by = used_by;

            job
        })
        .collect();

    queue.sort_by(|a, b| {
        a.stage
            .cmp(&b.stage)
            .then_with(|| activity_sort_rank(a.activity).cmp(&activity_sort_rank(b.activity)))
            .then_with(|| Reverse(a.duration_secs).cmp(&Reverse(b.duration_secs)))
            .then_with(|| a.product_name.cmp(&b.product_name))
            .then_with(|| a.blueprint_type_id.cmp(&b.blueprint_type_id))
    });

    queue
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::too_many_arguments)]
    fn craft_node(
        type_id: u32,
        bp_id: u32,
        name: &str,
        activity: BlueprintActivity,
        runs: u64,
        output_qty: u64,
        job_cost: f64,
        duration_secs: u64,
        children: Vec<OptimizedNode>,
    ) -> OptimizedNode {
        OptimizedNode {
            type_id,
            name: name.to_string(),
            required_qty: output_qty,
            from_local_stock: 0,
            net_qty: output_qty,
            decision: NodeDecision::Craft,
            is_craftable: true,
            activity: Some(activity),
            blueprint_type_id: Some(bp_id),
            runs,
            output_qty,
            me_level: 10,
            te_level: 20,
            unit_buy_price: 0.0,
            buy_cost: 0.0,
            craft_cost: Some(job_cost),
            job_cost,
            job_duration_secs: duration_secs,
            chosen_cost: job_cost,
            haulable_stock: Vec::new(),
            children,
        }
    }

    fn buy_node(type_id: u32, name: &str, qty: u64) -> OptimizedNode {
        OptimizedNode {
            type_id,
            name: name.to_string(),
            required_qty: qty,
            from_local_stock: 0,
            net_qty: qty,
            decision: NodeDecision::Buy,
            is_craftable: false,
            activity: None,
            blueprint_type_id: None,
            runs: 0,
            output_qty: 0,
            me_level: 0,
            te_level: 0,
            unit_buy_price: 10.0,
            buy_cost: (qty as f64) * 10.0,
            craft_cost: None,
            job_cost: 0.0,
            job_duration_secs: 0,
            chosen_cost: (qty as f64) * 10.0,
            haulable_stock: Vec::new(),
            children: Vec::new(),
        }
    }

    #[test]
    fn empty_and_buy_only_produce_empty_queue() {
        assert!(build_ordered_job_queue(&[]).is_empty());

        let nodes = vec![buy_node(34, "Tritanium", 1000)];
        assert!(build_ordered_job_queue(&nodes).is_empty());
    }

    #[test]
    fn linear_chain_orders_bottom_up() {
        let hexite = craft_node(
            1,
            101,
            "Hexite",
            BlueprintActivity::Reaction,
            5,
            1000,
            100.0,
            3600,
            vec![buy_node(50, "Moon Goo", 500)],
        );
        let ferrogel = craft_node(
            2,
            102,
            "Ferrogel",
            BlueprintActivity::Reaction,
            3,
            600,
            200.0,
            7200,
            vec![hexite],
        );
        let thruster = craft_node(
            3,
            103,
            "Plasma Thruster",
            BlueprintActivity::Manufacturing,
            10,
            10,
            300.0,
            1800,
            vec![ferrogel],
        );
        let ship = craft_node(
            4,
            104,
            "Huginn",
            BlueprintActivity::Manufacturing,
            1,
            1,
            5000.0,
            21600,
            vec![thruster],
        );

        let queue = build_ordered_job_queue(&[ship]);
        let names_and_stages: Vec<(&str, u32)> = queue
            .iter()
            .map(|j| (j.product_name.as_str(), j.stage))
            .collect();

        assert_eq!(
            names_and_stages,
            vec![
                ("Hexite", 1),
                ("Ferrogel", 2),
                ("Plasma Thruster", 3),
                ("Huginn", 4),
            ]
        );
        assert_eq!(queue[0].used_by, vec!["Ferrogel"]);
        assert_eq!(queue[3].used_by, Vec::<String>::new());
        assert_eq!(queue[3].prerequisite_blueprint_ids, vec![103]);
    }

    #[test]
    fn asymmetric_t2_ship_uses_asap_staging_and_activity_duration_sorting() {
        // Huginn depends on:
        // - Bellicose (T1 hull, MFG, 2h) -> no crafted prereqs => Stage 1
        // - Plasma Thruster (MFG, 1h) -> depends on:
        //     - Ferrogel (RXN, 3h) -> depends on Hexite (RXN, 1h) => Hexite Stage 1, Ferrogel Stage 2
        //     - Titanium Carbide (RXN, 2h) -> no crafted prereqs => Stage 1
        // - Construction Blocks (Buy) -> ignored
        let hexite = craft_node(1, 101, "Hexite", BlueprintActivity::Reaction, 5, 1000, 10.0, 3600, vec![]);
        let ferrogel = craft_node(2, 102, "Ferrogel", BlueprintActivity::Reaction, 3, 600, 20.0, 10800, vec![hexite]);
        let ti_carbide = craft_node(3, 103, "Titanium Carbide", BlueprintActivity::Reaction, 4, 800, 15.0, 7200, vec![]);
        let thruster = craft_node(
            4,
            104,
            "Plasma Thruster",
            BlueprintActivity::Manufacturing,
            10,
            10,
            50.0,
            3600,
            vec![ferrogel, ti_carbide],
        );
        let bellicose = craft_node(
            5,
            105,
            "Bellicose",
            BlueprintActivity::Manufacturing,
            1,
            1,
            40.0,
            7200,
            vec![buy_node(34, "Tritanium", 50000)],
        );
        let huginn = craft_node(
            6,
            106,
            "Huginn",
            BlueprintActivity::Manufacturing,
            1,
            1,
            500.0,
            21600,
            vec![bellicose, thruster, buy_node(99, "Construction Blocks", 20)],
        );

        let queue = build_ordered_job_queue(&[huginn]);
        let summary: Vec<(&str, u32, BlueprintActivity)> = queue
            .iter()
            .map(|j| (j.product_name.as_str(), j.stage, j.activity))
            .collect();

        // Stage 1: Reactions (Titanium Carbide 2h before Hexite 1h), then Manufacturing (Bellicose 2h)
        // Stage 2: Ferrogel
        // Stage 3: Plasma Thruster
        // Stage 4: Huginn
        assert_eq!(
            summary,
            vec![
                ("Titanium Carbide", 1, BlueprintActivity::Reaction),
                ("Hexite", 1, BlueprintActivity::Reaction),
                ("Bellicose", 1, BlueprintActivity::Manufacturing),
                ("Ferrogel", 2, BlueprintActivity::Reaction),
                ("Plasma Thruster", 3, BlueprintActivity::Manufacturing),
                ("Huginn", 4, BlueprintActivity::Manufacturing),
            ]
        );
    }

    #[test]
    fn diamond_and_shortcut_edges_place_consumer_after_longest_prereq_path() {
        // A -> B -> D and A -> D directly => A:1, B:2, D:3
        let a_for_b = craft_node(1, 101, "Base Mat", BlueprintActivity::Reaction, 2, 200, 10.0, 1000, vec![]);
        let b = craft_node(2, 102, "Mid Comp", BlueprintActivity::Manufacturing, 1, 1, 20.0, 2000, vec![a_for_b]);
        let a_for_d = craft_node(1, 101, "Base Mat", BlueprintActivity::Reaction, 3, 300, 15.0, 1500, vec![]);
        let d = craft_node(3, 103, "Final Hull", BlueprintActivity::Manufacturing, 1, 1, 50.0, 5000, vec![b, a_for_d]);

        let queue = build_ordered_job_queue(&[d]);
        assert_eq!(queue.len(), 3);

        assert_eq!(queue[0].product_name, "Base Mat");
        assert_eq!(queue[0].stage, 1);
        assert_eq!(queue[0].runs, 5);
        assert_eq!(queue[0].output_qty, 500);
        assert_eq!(queue[0].duration_secs, 2500);
        assert_eq!(queue[0].used_by, vec!["Final Hull", "Mid Comp"]);

        assert_eq!(queue[1].product_name, "Mid Comp");
        assert_eq!(queue[1].stage, 2);

        assert_eq!(queue[2].product_name, "Final Hull");
        assert_eq!(queue[2].stage, 3);
        assert_eq!(queue[2].prerequisite_blueprint_ids, vec![101, 102]);
    }

    #[test]
    fn multi_entry_consolidates_shared_components_and_tracks_all_consumers() {
        let comp_1 = craft_node(10, 210, "Core Component", BlueprintActivity::Manufacturing, 2, 2, 100.0, 600, vec![]);
        let ship_a = craft_node(11, 211, "Ship Alpha", BlueprintActivity::Manufacturing, 1, 1, 500.0, 3600, vec![comp_1]);

        let comp_2 = craft_node(10, 210, "Core Component", BlueprintActivity::Manufacturing, 3, 3, 150.0, 900, vec![]);
        let ship_b = craft_node(12, 212, "Ship Beta", BlueprintActivity::Manufacturing, 1, 1, 600.0, 7200, vec![comp_2]);

        let queue = build_ordered_job_queue(&[ship_a, ship_b]);
        assert_eq!(queue.len(), 3);

        assert_eq!(queue[0].product_name, "Core Component");
        assert_eq!(queue[0].stage, 1);
        assert_eq!(queue[0].runs, 5);
        assert_eq!(queue[0].used_by, vec!["Ship Alpha", "Ship Beta"]);

        // Stage 2: Ship Beta (7200s) before Ship Alpha (3600s)
        assert_eq!(queue[1].product_name, "Ship Beta");
        assert_eq!(queue[1].stage, 2);
        assert_eq!(queue[2].product_name, "Ship Alpha");
        assert_eq!(queue[2].stage, 2);
    }

    #[test]
    fn cyclic_graph_falls_back_gracefully_without_dropping_jobs() {
        let mut jobs = HashMap::new();
        jobs.insert(
            1,
            JobSummary {
                blueprint_type_id: 1,
                product_type_id: 1,
                product_name: "A".to_string(),
                activity: BlueprintActivity::Manufacturing,
                runs: 1,
                output_qty: 1,
                job_cost: 10.0,
                duration_secs: 100,
                stage: 1,
                prerequisite_blueprint_ids: Vec::new(),
                used_by: Vec::new(),
            },
        );
        jobs.insert(
            2,
            JobSummary {
                blueprint_type_id: 2,
                product_type_id: 2,
                product_name: "B".to_string(),
                activity: BlueprintActivity::Manufacturing,
                runs: 1,
                output_qty: 1,
                job_cost: 20.0,
                duration_secs: 200,
                stage: 1,
                prerequisite_blueprint_ids: Vec::new(),
                used_by: Vec::new(),
            },
        );

        let mut prereqs = HashMap::new();
        prereqs.insert(1, HashSet::from([2]));
        prereqs.insert(2, HashSet::from([1]));

        let queue = order_job_queue(jobs, &prereqs);
        assert_eq!(queue.len(), 2);
        assert!(queue.iter().all(|j| j.stage == 1));
    }
}
