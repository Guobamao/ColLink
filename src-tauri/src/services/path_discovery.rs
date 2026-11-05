use crate::models::{
    PathStep, PathStepColumn, PathStepTable, RelationPath, RelationPathResult,
};
use rusqlite::{Connection, Result};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone)]
struct GraphEdge {
    neighbor_table_id: String,
    from_table_id: String,
    from_table_name: String,
    from_table_comment: String,
    from_column_id: String,
    from_column_name: String,
    from_column_type: String,
    from_column_comment: String,
    to_table_id: String,
    to_table_name: String,
    to_table_comment: String,
    to_column_id: String,
    to_column_name: String,
    to_column_type: String,
    to_column_comment: String,
}

pub struct PathDiscoveryService;

impl PathDiscoveryService {
    pub fn discover_paths(
        conn: &Connection,
        project_id: &str,
        source_table_id: &str,
        target_table_id: &str,
        max_hops: usize,
    ) -> Result<RelationPathResult> {
        let tables = crate::db::tables::list_tables_for_project(conn, project_id)?;
        let table_map: HashMap<String, crate::models::TableMeta> =
            tables.into_iter().map(|t| (t.id.clone(), t)).collect();

        let src_tbl = match table_map.get(source_table_id) {
            Some(t) => t,
            None => {
                return Ok(RelationPathResult {
                    project_id: project_id.to_string(),
                    source_table_id: source_table_id.to_string(),
                    target_table_id: target_table_id.to_string(),
                    paths_count: 0,
                    paths: Vec::new(),
                })
            }
        };

        let _tgt_tbl = match table_map.get(target_table_id) {
            Some(t) => t,
            None => {
                return Ok(RelationPathResult {
                    project_id: project_id.to_string(),
                    source_table_id: source_table_id.to_string(),
                    target_table_id: target_table_id.to_string(),
                    paths_count: 0,
                    paths: Vec::new(),
                })
            }
        };

        if source_table_id == target_table_id {
            return Ok(RelationPathResult {
                project_id: project_id.to_string(),
                source_table_id: source_table_id.to_string(),
                target_table_id: target_table_id.to_string(),
                paths_count: 0,
                paths: Vec::new(),
            });
        }

        let mappings = crate::db::mappings::list_mappings(conn, project_id)?;

        let mut adj: HashMap<String, Vec<GraphEdge>> = HashMap::new();
        let mut seen_pairs: HashSet<(String, String, String, String)> = HashSet::new();

        for m in &mappings {
            if m.source_table_id == m.target_table_id {
                continue;
            }

            let st = match table_map.get(&m.source_table_id) {
                Some(t) => t,
                None => continue,
            };
            let tt = match table_map.get(&m.target_table_id) {
                Some(t) => t,
                None => continue,
            };
            let sc = match st.columns.iter().find(|c| c.id == m.source_column_id) {
                Some(c) => c,
                None => continue,
            };
            let tc = match tt.columns.iter().find(|c| c.id == m.target_column_id) {
                Some(c) => c,
                None => continue,
            };

            // Forward direction
            let k_fwd = (
                m.source_table_id.clone(),
                m.target_table_id.clone(),
                sc.id.clone(),
                tc.id.clone(),
            );
            if !seen_pairs.contains(&k_fwd) {
                seen_pairs.insert(k_fwd);
                adj.entry(m.source_table_id.clone())
                    .or_default()
                    .push(GraphEdge {
                        neighbor_table_id: m.target_table_id.clone(),
                        from_table_id: st.id.clone(),
                        from_table_name: st.table_name.clone(),
                        from_table_comment: st.table_comment.clone().unwrap_or_default(),
                        from_column_id: sc.id.clone(),
                        from_column_name: sc.column_name.clone(),
                        from_column_type: sc.data_type.clone(),
                        from_column_comment: sc.column_comment.clone().unwrap_or_default(),
                        to_table_id: tt.id.clone(),
                        to_table_name: tt.table_name.clone(),
                        to_table_comment: tt.table_comment.clone().unwrap_or_default(),
                        to_column_id: tc.id.clone(),
                        to_column_name: tc.column_name.clone(),
                        to_column_type: tc.data_type.clone(),
                        to_column_comment: tc.column_comment.clone().unwrap_or_default(),
                    });
            }

            // Backward direction
            let k_bwd = (
                m.target_table_id.clone(),
                m.source_table_id.clone(),
                tc.id.clone(),
                sc.id.clone(),
            );
            if !seen_pairs.contains(&k_bwd) {
                seen_pairs.insert(k_bwd);
                adj.entry(m.target_table_id.clone())
                    .or_default()
                    .push(GraphEdge {
                        neighbor_table_id: m.source_table_id.clone(),
                        from_table_id: tt.id.clone(),
                        from_table_name: tt.table_name.clone(),
                        from_table_comment: tt.table_comment.clone().unwrap_or_default(),
                        from_column_id: tc.id.clone(),
                        from_column_name: tc.column_name.clone(),
                        from_column_type: tc.data_type.clone(),
                        from_column_comment: tc.column_comment.clone().unwrap_or_default(),
                        to_table_id: st.id.clone(),
                        to_table_name: st.table_name.clone(),
                        to_table_comment: st.table_comment.clone().unwrap_or_default(),
                        to_column_id: sc.id.clone(),
                        to_column_name: sc.column_name.clone(),
                        to_column_type: sc.data_type.clone(),
                        to_column_comment: sc.column_comment.clone().unwrap_or_default(),
                    });
            }
        }

        // BFS Search
        let mut queue: VecDeque<(String, Vec<GraphEdge>, HashSet<String>)> = VecDeque::new();
        let mut init_visited = HashSet::new();
        init_visited.insert(source_table_id.to_string());
        queue.push_back((source_table_id.to_string(), Vec::new(), init_visited));

        let mut found_paths: Vec<Vec<GraphEdge>> = Vec::new();
        let mut seen_table_seqs: HashSet<Vec<String>> = HashSet::new();

        let mut expansions = 0;
        let max_expansions = 2000;
        let target_paths_limit = 8;

        while let Some((curr_table_id, current_steps, visited)) = queue.pop_front() {
            expansions += 1;
            if expansions >= max_expansions || found_paths.len() >= target_paths_limit {
                break;
            }

            if current_steps.len() >= max_hops {
                continue;
            }

            if let Some(links) = adj.get(&curr_table_id) {
                for link in links {
                    let next_table_id = &link.neighbor_table_id;

                    if next_table_id == target_table_id {
                        let mut new_steps = current_steps.clone();
                        new_steps.push(link.clone());

                        let mut table_seq: Vec<String> =
                            new_steps.iter().map(|s| s.from_table_id.clone()).collect();
                        table_seq.push(target_table_id.to_string());

                        if !seen_table_seqs.contains(&table_seq) {
                            seen_table_seqs.insert(table_seq);
                            found_paths.push(new_steps);
                        }
                        continue;
                    }

                    if !visited.contains(next_table_id) && current_steps.len() + 1 < max_hops {
                        let mut next_visited = visited.clone();
                        next_visited.insert(next_table_id.clone());

                        let mut next_steps = current_steps.clone();
                        next_steps.push(link.clone());

                        queue.push_back((next_table_id.clone(), next_steps, next_visited));
                    }
                }
            }
        }

        // Format and generate SQL
        let mut formatted_paths: Vec<RelationPath> = Vec::new();
        for (idx, raw_steps) in found_paths.iter().enumerate() {
            let mut steps = Vec::new();
            let mut involved_table_names = vec![src_tbl.table_name.clone()];
            let mut involved_table_ids = vec![src_tbl.id.clone()];

            for (s_idx, edge) in raw_steps.iter().enumerate() {
                if !involved_table_names.contains(&edge.to_table_name) {
                    involved_table_names.push(edge.to_table_name.clone());
                }
                if !involved_table_ids.contains(&edge.to_table_id) {
                    involved_table_ids.push(edge.to_table_id.clone());
                }

                steps.push(PathStep {
                    step_index: s_idx + 1,
                    relation_type: "EXPLICIT".to_string(),
                    match_reason: "已建立血缘连线".to_string(),
                    confidence: 1.0,
                    from_table: PathStepTable {
                        id: edge.from_table_id.clone(),
                        table_name: edge.from_table_name.clone(),
                        table_comment: edge.from_table_comment.clone(),
                    },
                    from_column: PathStepColumn {
                        id: edge.from_column_id.clone(),
                        column_name: edge.from_column_name.clone(),
                        data_type: edge.from_column_type.clone(),
                        column_comment: edge.from_column_comment.clone(),
                    },
                    to_table: PathStepTable {
                        id: edge.to_table_id.clone(),
                        table_name: edge.to_table_name.clone(),
                        table_comment: edge.to_table_comment.clone(),
                    },
                    to_column: PathStepColumn {
                        id: edge.to_column_id.clone(),
                        column_name: edge.to_column_name.clone(),
                        data_type: edge.to_column_type.clone(),
                        column_comment: edge.to_column_comment.clone(),
                    },
                    step_label: format!(
                        "{}.{} = {}.{}",
                        edge.from_table_name,
                        edge.from_column_name,
                        edge.to_table_name,
                        edge.to_column_name
                    ),
                });
            }

            // Generate SQL snippet
            let mut sql_lines = Vec::new();
            sql_lines.push("SELECT".to_string());
            sql_lines.push("  t1.*,".to_string());
            sql_lines.push(format!("  t{}.*", involved_table_names.len()));
            sql_lines.push(format!("FROM `{}` t1", src_tbl.table_name));

            for (s_idx, step) in steps.iter().enumerate() {
                let prev_alias = format!("t{}", s_idx + 1);
                let curr_alias = format!("t{}", s_idx + 2);
                sql_lines.push(format!(
                    "JOIN `{}` {} ON {}.`{}` = {}.`{}`",
                    step.to_table.table_name,
                    curr_alias,
                    prev_alias,
                    step.from_column.column_name,
                    curr_alias,
                    step.to_column.column_name
                ));
            }
            let sql_snippet = sql_lines.join("\n") + ";";

            let intermediate_table_ids: Vec<String> = if steps.len() > 1 {
                steps[..steps.len() - 1]
                    .iter()
                    .map(|s| s.to_table.id.clone())
                    .collect()
            } else {
                Vec::new()
            };

            formatted_paths.push(RelationPath {
                path_id: format!("path_{}", idx + 1),
                summary_title: involved_table_names.join(" -> "),
                hops: steps.len(),
                is_fully_explicit: true,
                avg_confidence: 1.0,
                involved_table_ids,
                intermediate_table_ids,
                steps,
                sql_snippet,
            });
        }

        formatted_paths.sort_by_key(|p| p.hops);
        formatted_paths.truncate(6);

        Ok(RelationPathResult {
            project_id: project_id.to_string(),
            source_table_id: source_table_id.to_string(),
            target_table_id: target_table_id.to_string(),
            paths_count: formatted_paths.len(),
            paths: formatted_paths,
        })
    }
}
