use std::collections::HashSet;

use tokio::sync::mpsc::Sender;

use crate::module::{
    assembly::{AssemblyInstruction, AssemblyInstructionId, Subroutine},
    cfg::Edge,
    ui::component::main_panel::ChannelMessage,
};

#[derive(Hash, Eq, PartialEq)]
pub struct BasicBlock {
    pub asm_inst_ids: Vec<AssemblyInstructionId>,
    pub edges: Vec<Edge>,
    pub connected: bool,
}

impl BasicBlock {
    pub fn new(asm_inst_ids: Vec<AssemblyInstructionId>, edges: Vec<Edge>) -> Self {
        Self {
            asm_inst_ids,
            edges,
            connected: true,
        }
    }

    pub fn last_asm_instruction<'a>(
        &self,
        subroutines: &'a [Subroutine],
    ) -> &'a AssemblyInstruction {
        let last_asm_inst_id = self.asm_inst_ids.last().unwrap();
        &subroutines[last_asm_inst_id.subroutine_index].asm_insts[last_asm_inst_id.asm_inst_index]
    }
}

pub async fn extract_basic_blocks(
    subroutines: &[Subroutine],
    nec_subs_indices: &[usize],
    tx: &Sender<ChannelMessage>,
) -> Vec<BasicBlock> {
    let mut basic_blocks = Vec::new();
    let mut asm_inst_ids = Vec::new();

    for i in nec_subs_indices {
        let mut j = 0;
        'outer_loop: while j < subroutines[*i].asm_insts.len() {
            if subroutines[*i].asm_insts[j].leader {
                asm_inst_ids.push(subroutines[*i].asm_insts[j].id.clone());

                j += 1;
                while j < subroutines[*i].asm_insts.len() {
                    if !subroutines[*i].asm_insts[j].leader {
                        asm_inst_ids.push(subroutines[*i].asm_insts[j].id.clone());
                    } else {
                        tx.send(ChannelMessage::Text(format!(
                            "Subroutine #{}: '{}' => Created Basic Block #{}",
                            *i + 1,
                            subroutines[*i].definition_label,
                            basic_blocks.len() + 1
                        )))
                        .await
                        .unwrap();

                        basic_blocks.push(BasicBlock::new(asm_inst_ids, Vec::new()));
                        asm_inst_ids = Vec::new();
                        continue 'outer_loop;
                    }

                    j += 1;
                }
            }

            j += 1;
        }
        tx.send(ChannelMessage::Text(format!(
            "Subroutine #{}: '{}' => Created Basic Block #{}",
            *i + 1,
            subroutines[*i].definition_label,
            basic_blocks.len() + 1
        )))
        .await
        .unwrap();
        // This is for the last basic block in the current subroutine.
        basic_blocks.push(BasicBlock::new(asm_inst_ids, Vec::new()));
        asm_inst_ids = Vec::new();
    }

    basic_blocks
}

pub fn find_basic_blocks_indices_by_ids(
    basic_blocks: &[BasicBlock],
    basic_blocks_ids: &[AssemblyInstructionId],
) -> Vec<usize> {
    let mut bb_indices = Vec::new();
    for basic_block_id in basic_blocks_ids {
        for (i, basic_block) in basic_blocks.iter().enumerate() {
            if basic_block_id == basic_block.asm_inst_ids.first().unwrap() {
                bb_indices.push(i);
                break;
            }
        }
    }

    if bb_indices.len() == basic_blocks_ids.len() {
        bb_indices
    } else {
        panic!(
            "Some basic blocks indices couldn't be found! [bb_indices_len: {} != bb_ids_len: {}]",
            bb_indices.len(),
            basic_blocks_ids.len()
        );
    }
}

pub fn get_starting_basic_block_index(main_sub_index: usize, basic_blocks: &[BasicBlock]) -> usize {
    for (i, basic_block) in basic_blocks.iter().enumerate() {
        let first_asm_inst = basic_block.asm_inst_ids.first().unwrap();

        // Checks if the first instruction of the basic block is the the first instruction of the main subroutine. If so, it is the starting basic block.
        if first_asm_inst.subroutine_index == main_sub_index && first_asm_inst.asm_inst_index == 0 {
            return i;
        }
    }

    panic!("The starting basic block index was not found!");
}

pub fn flag_connected_basic_blocks(main_sub_index: usize, basic_blocks: &mut [BasicBlock]) {
    let mut connected_indices = HashSet::new();
    let mut unconnected_indices = Vec::new();

    connected_indices.insert(get_starting_basic_block_index(main_sub_index, basic_blocks));

    for basic_block in basic_blocks.as_ref() {
        for edge in &basic_block.edges {
            connected_indices.insert(edge.destination_index);
        }
    }

    for i in 0..basic_blocks.len() {
        if !connected_indices.contains(&i) {
            unconnected_indices.push(i);
        }
    }

    for u in unconnected_indices {
        basic_blocks[u].connected = false;
    }
}
