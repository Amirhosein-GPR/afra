use std::{fs, sync::Arc};
use tokio::sync::mpsc::Sender;

use crate::module::{
    assembly::{self, AssemblyInstruction, AssemblyInstructionId, Subroutine},
    basic_block::{self, BasicBlock},
    regex::RegexContainer,
    ui::component::main_panel::ChannelMessage,
};

/// Contains the assembly path to an assembly source file, and it's extracted necessary subroutines.
pub struct AssemblyMetaData {
    pub processed_asm_path: Vec<String>,
    pub subroutines: Option<Vec<Vec<Subroutine>>>,
    pub basic_blocks: Vec<Vec<BasicBlock>>,
    pub main_subroutine_indices: Vec<usize>,
}

impl AssemblyMetaData {
    /// Creates a new `AssemblyData` which has some useful methods used for extracting data from assembly files.
    pub fn new() -> Self {
        Self {
            processed_asm_path: Vec::new(),
            subroutines: None,
            basic_blocks: Vec::new(),
            main_subroutine_indices: Vec::new(),
        }
    }

    pub async fn process_original_assemblies(
        asm_paths: Vec<String>,
        processed_asm_paths: Vec<String>,
        regex_container: Arc<RegexContainer>,
        tx: Sender<ChannelMessage>,
    ) {
        tx.send(ChannelMessage::Text(
            "================ Processing Original assemblies =================".to_string(),
        ))
        .await
        .unwrap();
        tx.send(ChannelMessage::Text(String::new())).await.unwrap();

        for (i, asm_path) in asm_paths.iter().enumerate() {
            tx.send(ChannelMessage::Text(format!("Processing: '{}'", asm_path)))
                .await
                .unwrap();
            let assembly_string = fs::read_to_string(asm_path).unwrap();
            let mut asm_lines = assembly_string.lines();
            let mut processed_asm_lines = Vec::new();
            let mut latest_line_number;

            while let Some(asm_line) = asm_lines.next() {
                if regex_container.definition_label.is_match(asm_line) {
                    processed_asm_lines.push(asm_line.to_string());
                    latest_line_number = None;

                    while let Some(asm_line) = asm_lines.next() {
                        if asm_line.trim().is_empty() {
                            processed_asm_lines.push("".to_string());
                            break;
                        } else if let Some(captures) = regex_container
                            .original_source_line_number
                            .captures(asm_line)
                        {
                            latest_line_number = Some(captures.get(1).unwrap().as_str());
                        } else if !regex_container.redundant_function_call.is_match(asm_line) {
                            if let Some(latest_line_number) = latest_line_number {
                                processed_asm_lines.push(format!(
                                    "{} ({})",
                                    asm_line.split("//").collect::<Vec<_>>()[0].trim(),
                                    latest_line_number
                                ));
                            } else {
                                processed_asm_lines.push(
                                    asm_line.split("//").collect::<Vec<_>>()[0]
                                        .trim()
                                        .to_string(),
                                );
                            }
                        }
                    }
                }
            }
            fs::write(&processed_asm_paths[i], processed_asm_lines.join("\n")).unwrap();
            tx.send(ChannelMessage::Text(format!(
                "Saved at: '{}'",
                &processed_asm_paths[i]
            )))
            .await
            .unwrap();
            tx.send(ChannelMessage::Text(String::new())).await.unwrap();

            tx.send(ChannelMessage::PhaseProgress(
                (i + 1) as f64 / asm_paths.len() as f64,
            ))
            .await
            .unwrap();
        }

        tx.send(ChannelMessage::Text(
            "================ Processing Original assemblies Finished! ================="
                .to_string(),
        ))
        .await
        .unwrap();

        tx.send(ChannelMessage::PhaseFinishedSignal).await.unwrap();
    }

    /// Extracts subroutines.
    pub async fn extract_subroutines(
        processed_asm_paths: Vec<String>,
        regex_container: Arc<RegexContainer>,
        tx: Sender<ChannelMessage>,
    ) {
        tx.send(ChannelMessage::Text(
            "================ Extracting Subroutines =================".to_string(),
        ))
        .await
        .unwrap();
        tx.send(ChannelMessage::Text(String::new())).await.unwrap();

        let mut subroutines_vec = Vec::new();
        for (i, processed_asm_path) in processed_asm_paths.iter().enumerate() {
            tx.send(ChannelMessage::Text(format!(
                "Extracting subroutines for : {processed_asm_path}"
            )))
            .await
            .unwrap();

            let assembly_string = fs::read_to_string(processed_asm_path).unwrap();
            let mut asm_lines = assembly_string.lines();

            let mut subroutines = Vec::new();
            let mut asm_line_number = 1;
            while let Some(asm_line) = asm_lines.next() {
                if regex_container.definition_label.is_match(asm_line) {
                    let definition_label = asm_line.to_string();
                    let start_line = asm_line_number + 1;

                    let mut assembly_instructions = Vec::new();

                    while let Some(asm_line) = asm_lines.next() {
                        asm_line_number += 1;

                        if asm_line.trim().is_empty() {
                            let end_line = asm_line_number - 1;

                            subroutines.push(Subroutine {
                                definition_label,
                                start_line,
                                end_line,
                                asm_insts: assembly_instructions,
                            });

                            tx.send(ChannelMessage::Text(format!(
                                "Subroutine #{} was extracted",
                                subroutines.len()
                            )))
                            .await
                            .unwrap();

                            break;
                        } else {
                            let source_line_number;
                            if let Some(captures) = regex_container
                                .processed_source_line_number
                                .captures(asm_line)
                            {
                                source_line_number = Some(
                                    captures.get(1).unwrap().as_str().parse::<usize>().unwrap(),
                                );
                            } else {
                                source_line_number = None;
                            }

                            assembly_instructions.push(AssemblyInstruction::new(
                                AssemblyInstructionId {
                                    subroutine_index: subroutines.len(),
                                    asm_inst_index: assembly_instructions.len(),
                                },
                                source_line_number,
                                asm_line_number,
                                asm_line,
                                &regex_container.instruction_name,
                            ));
                        }
                    }
                }

                asm_line_number += 1;
            }

            subroutines_vec.push(subroutines);

            tx.send(ChannelMessage::Text(format!(
                "Extracting subroutines for : {processed_asm_path} was finished!"
            )))
            .await
            .unwrap();
            tx.send(ChannelMessage::Text(String::new())).await.unwrap();

            tx.send(ChannelMessage::PhaseProgress(
                (i + 1) as f64 / processed_asm_paths.len() as f64,
            ))
            .await
            .unwrap();
        }

        tx.send(ChannelMessage::Text(
            "================ Extracting Subroutines Finisehd! =================".to_string(),
        ))
        .await
        .unwrap();

        tx.send(ChannelMessage::Subroutines(subroutines_vec))
            .await
            .unwrap();

        tx.send(ChannelMessage::PhaseFinishedSignal).await.unwrap();
    }

    pub async fn extract_main_subroutine_indices(
        subroutines_vec: Vec<Vec<Subroutine>>,
        tx: Sender<ChannelMessage>,
    ) {
        let mut main_subroutine_indices = Vec::new();
        for (i, subroutines) in subroutines_vec.iter().enumerate() {
            for (j, subroutine) in subroutines.iter().enumerate() {
                if subroutine.definition_label.contains("<main>:") {
                    main_subroutine_indices.push(j);
                    tx.send(ChannelMessage::Text(format!(
                        "Found subroutine index for \"{}\": {}",
                        subroutine.definition_label, j
                    )))
                    .await
                    .unwrap();
                    break;
                }
            }
            if i + 1 != main_subroutine_indices.len() {
                panic!("Main subroutine was not found!");
            }

            tx.send(ChannelMessage::PhaseProgress(
                (i + 1) as f64 / subroutines_vec.len() as f64,
            ))
            .await
            .unwrap();
        }

        tx.send(ChannelMessage::MainSubroutineIndices(
            main_subroutine_indices,
        ))
        .await
        .unwrap();

        tx.send(ChannelMessage::Subroutines(subroutines_vec))
            .await
            .unwrap();
        tx.send(ChannelMessage::PhaseFinishedSignal).await.unwrap();
    }

    pub async fn extract_basic_blocks(
        mut subroutines_vec: Vec<Vec<Subroutine>>,
        trace_paths: Vec<String>,
        regex_container: Arc<RegexContainer>,
        tx: Sender<ChannelMessage>,
    ) {
        tx.send(ChannelMessage::Text(
            "================ Extracting Basic Blocks =================".to_string(),
        ))
        .await
        .unwrap();
        tx.send(ChannelMessage::Text(String::new())).await.unwrap();

        let mut basic_blocks_vec = Vec::new();

        let subroutines_vec_len = subroutines_vec.len();
        for (i, subroutines) in subroutines_vec.iter_mut().enumerate() {
            let main_subroutine_index =
                Subroutine::get_main_subroutine_index(&subroutines, &tx).await;
            let nec_subs_indices =
                assembly::set_target_ids_and_extract_necessary_subroutines_indices(
                    main_subroutine_index,
                    subroutines,
                    &trace_paths[i],
                    &regex_container.gem5_trace_instruction_address,
                    &tx,
                )
                .await;

            assembly::compute_and_flag_leaders(
                main_subroutine_index,
                subroutines,
                &nec_subs_indices,
                &tx,
            )
            .await;

            for i in &nec_subs_indices {
                for asm_inst in &subroutines[*i].asm_insts {
                    if asm_inst.leader {
                        tx.send(ChannelMessage::Text(format!(
                            "Leaders:\n{}",
                            asm_inst.content
                        )))
                        .await
                        .unwrap();
                    }
                }
            }

            let basic_blocks =
                basic_block::extract_basic_blocks(subroutines, &nec_subs_indices, &tx).await;

            basic_blocks_vec.push(basic_blocks);

            tx.send(ChannelMessage::PhaseProgress(
                (i + 1) as f64 / subroutines_vec_len as f64,
            ))
            .await
            .unwrap();
        }

        tx.send(ChannelMessage::Text(
            "================ Extracting Basic Blocks Finished! =================".to_string(),
        ))
        .await
        .unwrap();

        tx.send(ChannelMessage::BasicBlocks(basic_blocks_vec))
            .await
            .unwrap();

        tx.send(ChannelMessage::Subroutines(subroutines_vec))
            .await
            .unwrap();
        tx.send(ChannelMessage::PhaseFinishedSignal).await.unwrap();
    }
}
