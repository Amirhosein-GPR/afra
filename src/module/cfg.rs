use std::{fs, time::Instant};

use tokio::{io::AsyncBufReadExt, sync::mpsc::Sender};

use crate::module::{
    analyzer::AssemblyMetaData,
    assembly::{AssemblyInstructionType, TargetIdWrapper},
    basic_block::{self},
    config::CFG_EXT,
    io::{self, FileType, InputManager},
    ui::component::main_panel::ChannelMessage,
};

pub async fn compute_control_flow(mut asm_meta_data: AssemblyMetaData, tx: Sender<ChannelMessage>) {
    tx.send(ChannelMessage::Text(
        "=== Computing Control Flow Graphs ===".to_string(),
    ))
    .await
    .unwrap();

    let subroutines = asm_meta_data.subroutines.as_ref().unwrap();

    for i in 0..asm_meta_data.basic_blocks.len() {
        for j in 0..asm_meta_data.basic_blocks[i].len() {
            tx.send(
                ChannelMessage::Text(format!(
                "Computing Control Flow {}/{} (Progress: {:.1}%). Basic Block {}/{} (Progress: {:.1}%)",
                i + 1,
                asm_meta_data.basic_blocks.len(),
                (i + 1) as f32 / asm_meta_data.basic_blocks.len() as f32 * 100.0,
                j + 1,
                asm_meta_data.basic_blocks[i].len(),
                (j + 1) as f32 / asm_meta_data.basic_blocks[i].len() as f32 * 100.0,
                ))
            ).await.unwrap();

            let mut edges = Vec::new();

            let bb_last_inst =
                asm_meta_data.basic_blocks[i][j].last_asm_instruction(&subroutines[i]);
            match &bb_last_inst.inst_type {
                AssemblyInstructionType::CtiConditional(_ta)
                | AssemblyInstructionType::CtiUnconditional(_ta) => {
                    if bb_last_inst.is_the_final_instruction_of_the_main_subroutine(
                        &subroutines[i],
                        asm_meta_data.main_subroutine_indices[i],
                    ) {
                        continue;
                    }

                    if bb_last_inst.reachable {
                        let bb_target_id_wrapper = bb_last_inst.get_target_ids();
                        match bb_target_id_wrapper {
                            TargetIdWrapper::Some(bb_target_ids) => {
                                let target_bb_indices =
                                    basic_block::find_basic_blocks_indices_by_ids(
                                        &asm_meta_data.basic_blocks[i],
                                        &bb_target_ids,
                                    );

                                for target_bb_index in target_bb_indices {
                                    edges.push(Edge::new(
                                        bb_last_inst.branch_condition().unwrap(),
                                        target_bb_index,
                                    ));
                                }
                            }
                            TargetIdWrapper::OutOfRange => {}
                            TargetIdWrapper::None => {}
                        }
                    }

                    match &bb_last_inst.inst_type {
                        AssemblyInstructionType::CtiConditional(_ta) => {
                            if bb_last_inst
                                .next_asm_inst_id_in_the_same_subroutine(&subroutines[i])
                                .is_some()
                            {
                                edges.push(Edge::new(
                                    bb_last_inst.branch_condition_complement().unwrap(),
                                    j + 1,
                                ));
                            }
                        }
                        _ => {}
                    }
                }

                AssemblyInstructionType::Ncti => {
                    if bb_last_inst
                        .next_asm_inst_id_in_the_same_subroutine(&subroutines[i])
                        .is_some()
                    {
                        edges.push(Edge::new("true".to_string(), j + 1));
                    }
                }
            }

            asm_meta_data.basic_blocks[i][j].edges = edges;
        }

        tx.send(ChannelMessage::PhaseProgress(
            (i + 1) as f64 / asm_meta_data.basic_blocks.len() as f64,
        ))
        .await
        .unwrap();
    }

    tx.send(ChannelMessage::Text(
        "=== Computing Control Flow Graphs Was Finished! ===".to_string(),
    ))
    .await
    .unwrap();
    tx.send(ChannelMessage::AssemblyMetaData(asm_meta_data))
        .await
        .unwrap();

    tx.send(ChannelMessage::PhaseFinishedSignal).await.unwrap();
}

pub async fn trim_unconnected_basic_blocks(
    mut asm_meta_data: AssemblyMetaData,
    tx: Sender<ChannelMessage>,
) {
    tx.send(ChannelMessage::Text(
        "=== Trimming Unconnected Basic Blocks ===".to_string(),
    ))
    .await
    .unwrap();

    let basic_blocks_vec_len = asm_meta_data.basic_blocks.len();
    for (i, basic_blocks) in asm_meta_data.basic_blocks.iter_mut().enumerate() {
        basic_block::flag_connected_basic_blocks(
            asm_meta_data.main_subroutine_indices[i],
            basic_blocks,
        );

        tx.send(ChannelMessage::PhaseProgress(
            (i + 1) as f64 / basic_blocks_vec_len as f64,
        ))
        .await
        .unwrap();
    }

    tx.send(ChannelMessage::Text(
        "=== Trimming Unconnected Basic Blocks Was Finished! ===".to_string(),
    ))
    .await
    .unwrap();
    tx.send(ChannelMessage::AssemblyMetaData(asm_meta_data))
        .await
        .unwrap();

    tx.send(ChannelMessage::PhaseFinishedSignal).await.unwrap();
}

fn programs_basic_blocks_flows_in_text(asm_meta_data: &AssemblyMetaData) -> Vec<Vec<String>> {
    let subroutines = asm_meta_data.subroutines.as_ref().unwrap();

    let mut programs_bbf = Vec::new();

    for (i, basic_blocks) in asm_meta_data.basic_blocks.iter().enumerate() {
        let mut program_bbf = Vec::new();
        for (j, basic_block) in basic_blocks.iter().enumerate() {
            if basic_block.connected {
                let mut edges_info = Vec::new();
                let mut raw_contents = Vec::new();
                for (k, edge) in basic_block.edges.iter().enumerate() {
                    edges_info.push(format!(
                        "E #{} => BB #{}",
                        k + 1,
                        edge.destination_index + 1
                    ));
                }
                for asm_inst_id in &basic_block.asm_inst_ids {
                    raw_contents.push(format!(
                        "  ({})  {}",
                        subroutines[i][asm_inst_id.subroutine_index].asm_insts
                            [asm_inst_id.asm_inst_index]
                            .asm_line_number,
                        subroutines[i][asm_inst_id.subroutine_index].asm_insts
                            [asm_inst_id.asm_inst_index]
                            .content
                    ));
                }
                program_bbf.push(format!(
                    "Program #{}, BB #{}:\n  Content:\n  {}\n  Edges:\n    {}\n",
                    i + 1,
                    j + 1,
                    raw_contents.join("\n  "),
                    if edges_info.len() > 0 {
                        edges_info.join(", ")
                    } else {
                        "None".to_string()
                    },
                ));
            }
        }
        programs_bbf.push(program_bbf);
    }

    programs_bbf
}

pub fn print(asm_meta_data: &AssemblyMetaData) {
    let programs_bbf = programs_basic_blocks_flows_in_text(asm_meta_data);

    for (i, program_bbf) in programs_bbf.iter().enumerate() {
        println!("=== Program #{i} ===");
        for bbf in program_bbf {
            println!("{bbf}");
        }
        println!("");
    }
}

pub async fn export_to_all_formats(
    asm_meta_data: AssemblyMetaData,
    cfg_text_paths: Vec<String>,
    cfg_graphics_paths: Vec<String>,
    tx: Sender<ChannelMessage>,
) {
    tx.send(ChannelMessage::Text(
        "=== Exporting Control Flow Graph To Text ===".to_string(),
    ))
    .await
    .unwrap();

    let export_start = Instant::now();
    export(&asm_meta_data, FileType::CfgText, cfg_text_paths, &tx).await;
    let text_export_time = export_start.elapsed().as_millis();

    tx.send(ChannelMessage::Text(
        "=== Exporting Control Flow Graph To Text Was Finished! ===".to_string(),
    ))
    .await
    .unwrap();

    tx.send(ChannelMessage::Text(
        "=== Exporting Control Flow Graph To PDF and SVG Format ===".to_string(),
    ))
    .await
    .unwrap();

    let export_start = Instant::now();
    export(
        &asm_meta_data,
        FileType::CfgGraphics,
        cfg_graphics_paths,
        &tx,
    )
    .await;
    let graphics_export_time = export_start.elapsed().as_millis();

    tx.send(ChannelMessage::Text(
        "=== Exporting Control Flow Graph To PDF and SVG Format Was Finished! ===".to_string(),
    ))
    .await
    .unwrap();

    tx.send(ChannelMessage::AssemblyMetaData(asm_meta_data))
        .await
        .unwrap();

    tx.send(ChannelMessage::CFGExportTimes(
        text_export_time,
        graphics_export_time,
    ))
    .await
    .unwrap();

    tx.send(ChannelMessage::PhaseFinishedSignal).await.unwrap();
}

async fn export(
    asm_meta_data: &AssemblyMetaData,
    file_type: FileType,
    cfg_paths: Vec<String>,
    tx: &Sender<ChannelMessage>,
) {
    match file_type {
        FileType::CfgText => {
            let programs_bbf = programs_basic_blocks_flows_in_text(asm_meta_data);
            InputManager::create_parent_dirs(cfg_paths.first().unwrap());

            for (i, program_bbf) in programs_bbf.iter().enumerate() {
                fs::write(
                    format!("{}{CFG_EXT}.txt", cfg_paths[i]),
                    program_bbf.join("\n"),
                )
                .unwrap();

                tx.send(ChannelMessage::PhaseProgress(
                    (i + 1) as f64 / programs_bbf.len() as f64 / 2.0,
                ))
                .await
                .unwrap();
            }
        }
        FileType::CfgGraphics => {
            InputManager::create_parent_dirs(cfg_paths.first().unwrap());
            serialize_in_graphviz_dot(asm_meta_data, &cfg_paths, tx).await;
            export_to_graphical_with_dot(&cfg_paths, tx).await;
        }
        _ => {
            panic!("Wrong file type passed to export function: {file_type:?}");
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct Edge {
    pub condition: String,
    pub destination_index: usize,
}

impl Edge {
    pub fn new(condition: String, destination_index: usize) -> Self {
        Self {
            condition,
            destination_index,
        }
    }
}

async fn serialize_in_graphviz_dot(
    asm_meta_data: &AssemblyMetaData,
    cfg_graphics_paths: &[String],
    tx: &Sender<ChannelMessage>,
) {
    let subroutines = asm_meta_data.subroutines.as_ref().unwrap();

    tx.send(ChannelMessage::Text(String::new())).await.unwrap();
    tx.send(ChannelMessage::Text(
        "===== Serializing CFG Data In Graphviz Dot Format To the Related File =====".to_string(),
    ))
    .await
    .unwrap();

    for (i, basic_blocks) in asm_meta_data.basic_blocks.iter().enumerate() {
        tx.send(ChannelMessage::Text(format!(
            "Serializing CFG Data And Storing Them In {}{CFG_EXT}.dot ({}/{} [{:.1}%])",
            cfg_graphics_paths[i],
            i + 1,
            asm_meta_data.basic_blocks.len(),
            (i + 1) as f32 / asm_meta_data.basic_blocks.len() as f32 / 100.0
        )))
        .await
        .unwrap();

        let mut graphviz_dot_wrapper = Vec::new();
        graphviz_dot_wrapper.push("digraph cfg {\n".to_string());

        for (j, basic_block) in basic_blocks.iter().enumerate() {
            if basic_block.connected {
                let mut asm_insts_contents = Vec::new();
                for asm_inst_id in &basic_block.asm_inst_ids {
                    asm_insts_contents.push(format!(
                        "({})  {}",
                        subroutines[i][asm_inst_id.subroutine_index].asm_insts
                            [asm_inst_id.asm_inst_index]
                            .asm_line_number,
                        subroutines[i][asm_inst_id.subroutine_index].asm_insts
                            [asm_inst_id.asm_inst_index]
                            .content
                            .clone(),
                    ));
                }
                graphviz_dot_wrapper.push(format!(
                    "    {} [label=\"BB {}\\n\\n{}\"];\n",
                    j + 1,
                    j + 1,
                    asm_insts_contents.join("\\l")
                ));

                if basic_block.edges.len() > 0 {
                    for edge in &basic_block.edges {
                        graphviz_dot_wrapper.push(format!(
                            "    {} -> {} [label=\"{}\"];\n",
                            j + 1,
                            edge.destination_index + 1,
                            edge.condition
                        ));
                    }
                }
            }
        }

        graphviz_dot_wrapper.push("}".to_string());

        fs::write(
            format!("{}{CFG_EXT}.dot", cfg_graphics_paths[i]),
            graphviz_dot_wrapper.join(""),
        )
        .unwrap();
    }

    tx.send(ChannelMessage::Text(String::new())).await.unwrap();
    tx.send(ChannelMessage::Text(
        "===== Writing Serialized CFG Data Was Finisehd! =====".to_string(),
    ))
    .await
    .unwrap();
}

async fn export_to_graphical_with_dot(cfg_graphics_paths: &[String], tx: &Sender<ChannelMessage>) {
    tx.send(ChannelMessage::Text(String::new())).await.unwrap();
    tx.send(ChannelMessage::Text(
        "============== Compiling Control Flow Graph Dot Files ==============".to_string(),
    ))
    .await
    .unwrap();
    tx.send(ChannelMessage::Text(
        "CMD ---> dot [DOT_FILE_PATH] -Tpdf -o [PDF_FILE_PATH] -Tsvg -o [SVG_FILE_PATH] -v"
            .to_string(),
    ))
    .await
    .unwrap();
    tx.send(ChannelMessage::Text(String::new())).await.unwrap();

    for i in 0..cfg_graphics_paths.len() {
        tx.send(ChannelMessage::Text(format!(
            "Compiling {}{CFG_EXT}.dot ({}/{} [{:.1}%])",
            cfg_graphics_paths[i],
            i + 1,
            cfg_graphics_paths.len(),
            (i + 1) as f32 / cfg_graphics_paths.len() as f32 / 100.0
        )))
        .await
        .unwrap();

        let (buf_reader_stdout, buf_reader_stderr) = io::run_command(
            "dot",
            &[
                format!("{}{CFG_EXT}.dot", cfg_graphics_paths[i]).as_str(),
                "-Tpdf",
                "-o",
                format!("{}{CFG_EXT}.pdf", cfg_graphics_paths[i]).as_str(),
                "-Tsvg",
                "-o",
                format!("{}{CFG_EXT}.svg", cfg_graphics_paths[i]).as_str(),
                "-v",
            ],
        );

        let mut lines = buf_reader_stderr.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            tx.send(ChannelMessage::Text(line)).await.unwrap();
        }
        let mut lines = buf_reader_stdout.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            tx.send(ChannelMessage::Text(line)).await.unwrap();
        }

        tx.send(ChannelMessage::PhaseProgress(
            0.5 + ((i + 1) as f64 / cfg_graphics_paths.len() as f64) / 2.0,
        ))
        .await
        .unwrap();
    }

    tx.send(ChannelMessage::Text(String::new())).await.unwrap();
    tx.send(ChannelMessage::Text(
        "===== Control Flow Graph Dot Files were compiled successfully! =====".to_string(),
    ))
    .await
    .unwrap();
}
