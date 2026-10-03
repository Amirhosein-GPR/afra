use std::{
    collections::{HashSet, VecDeque},
    fs,
};

use regex::Regex;
use tokio::sync::mpsc::Sender;

use crate::module::ui::component::main_panel::ChannelMessage;

/// Represents a subroutine in an assembly source file.
///
/// It cotains some useful info about a subroutine in an assembly file.
pub struct Subroutine {
    pub definition_label: String,
    pub start_line: usize,
    pub end_line: usize,
    pub asm_insts: Vec<AssemblyInstruction>,
}

impl Subroutine {
    // TODO: Might be able to increase performance of it later.
    pub async fn get_main_subroutine_index(
        subroutines: &[Subroutine],
        tx: &Sender<ChannelMessage>,
    ) -> usize {
        for (i, subroutine) in subroutines.iter().enumerate() {
            if subroutine.definition_label.contains("<main>:") {
                tx.send(ChannelMessage::Text(format!("{}", i)))
                    .await
                    .unwrap();
                return i;
            }
        }

        panic!("Main subroutine was not found!");
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct AssemblyInstructionId {
    pub subroutine_index: usize,
    pub asm_inst_index: usize,
}

/// Represents a control transfaer instruction (aka, jump instruction) in an assembly file.
///
/// It contains some useful information about each CTI.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct AssemblyInstruction {
    pub id: AssemblyInstructionId,
    pub name: String,
    pub source_line_number: Option<usize>,
    pub asm_line_number: usize,
    pub content: String,
    pub inst_type: AssemblyInstructionType,
    pub reachable: bool,
    pub leader: bool,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum TargetIdWrapper {
    Some(Vec<AssemblyInstructionId>),
    OutOfRange,
    None,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum AssemblyInstructionType {
    CtiConditional(TargetIdWrapper),
    CtiUnconditional(TargetIdWrapper),
    Ncti,
}

impl AssemblyInstruction {
    pub fn new(
        id: AssemblyInstructionId,
        source_line_number: Option<usize>,
        asm_line_number: usize,
        content: &str,
        instruction_name_regex: &Regex,
    ) -> Self {
        let instruction_name = instruction_name_regex
            .captures(content)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str();

        let inst_type = match instruction_name {
            "b.eq" | "b.ne" | "b.cs" | "b.cc" | "b.mi" | "b.pl" | "b.vs" | "b.vc" | "b.hi"
            | "b.ls" | "b.ge" | "b.lt" | "b.gt" | "b.le" | "bc.eq" | "bc.ne" | "bc.cs"
            | "bc.cc" | "bc.mi" | "bc.pl" | "bc.vs" | "bc.vc" | "bc.hi" | "bc.ls" | "bc.ge"
            | "bc.lt" | "bc.gt" | "bc.le" | "cbz" | "cbnz" | "tbz" | "tbnz" => {
                AssemblyInstructionType::CtiConditional(TargetIdWrapper::None)
            }
            "b" | "bl" | "ret" | "br" | "blr" | "b.al" | "bc.al" | "b.nv" | "bc.nv" => {
                AssemblyInstructionType::CtiUnconditional(TargetIdWrapper::None)
            }
            _ => AssemblyInstructionType::Ncti,
        };

        Self {
            id,
            name: instruction_name.to_string(),
            source_line_number,
            asm_line_number,
            content: content.to_string(),
            inst_type,
            reachable: true,
            leader: false,
        }
    }

    pub fn branch_condition(&self) -> Result<String, String> {
        match &self.inst_type {
            AssemblyInstructionType::CtiConditional(_ta)
            | AssemblyInstructionType::CtiUnconditional(_ta) => Ok(match self.name.as_str() {
                "b.eq" => "Z == 1",
                "b.ne" => "Z == 0",
                "b.cs" => "C == 1",
                "b.cc" => "C == 0",
                "b.mi" => "N == 1",
                "b.pl" => "N == 0",
                "b.vs" => "V == 1",
                "b.vc" => "V == 0",
                "b.hi" => "C == 1 and Z == 0",
                "b.ls" => "C == 0 or Z == 1",
                "b.ge" => "N == V",
                "b.lt" => "N != V",
                "b.gt" => "Z == 0 and N == V",
                "b.le" => "Z == 1 or N != V",
                "bc.eq" => "Z == 1",
                "bc.ne" => "Z == 0",
                "bc.cs" => "C == 1",
                "bc.cc" => "C == 0",
                "bc.mi" => "N == 1",
                "bc.pl" => "N == 0",
                "bc.vs" => "V == 1",
                "bc.vc" => "V == 0",
                "bc.hi" => "C == 1 and Z == 0",
                "bc.ls" => "C == 0 or Z == 1",
                "bc.ge" => "N == V",
                "bc.lt" => "N != V",
                "bc.gt" => "Z == 0 and N == V",
                "bc.le" => "Z == 1 or N != V",
                "cbz" => "register == 0",
                "cbnz" => "register != 0",
                "tbz" => "selected bit == 0",
                "tbnz" => "selected bit == 1",
                _ => "true",
            }
            .to_string()),

            AssemblyInstructionType::Ncti => Err(format!(
                "NCTI Instruction: \"{}\" has no branch condition!",
                self.content
            )),
        }
    }

    pub fn branch_condition_complement(&self) -> Result<String, String> {
        match &self.inst_type {
            AssemblyInstructionType::CtiConditional(_ta)
            | AssemblyInstructionType::CtiUnconditional(_ta) => Ok(match self.name.as_str() {
                "b.eq" => "Z != 1",
                "b.ne" => "Z != 0",
                "b.cs" => "C != 1",
                "b.cc" => "C != 0",
                "b.mi" => "N != 1",
                "b.pl" => "N != 0",
                "b.vs" => "V != 1",
                "b.vc" => "V != 0",
                "b.hi" => "C != 1 or Z != 0",
                "b.ls" => "C != 0 and Z != 1",
                "b.ge" => "N != V",
                "b.lt" => "N == V",
                "b.gt" => "Z != 0 or N != V",
                "b.le" => "Z != 1 and N == V",
                "bc.eq" => "Z != 1",
                "bc.ne" => "Z != 0",
                "bc.cs" => "C != 1",
                "bc.cc" => "C != 0",
                "bc.mi" => "N != 1",
                "bc.pl" => "N != 0",
                "bc.vs" => "V != 1",
                "bc.vc" => "V != 0",
                "bc.hi" => "C != 1 or Z != 0",
                "bc.ls" => "C != 0 and Z != 1",
                "bc.ge" => "N != V",
                "bc.lt" => "N == V",
                "bc.gt" => "Z != 0 or N != V",
                "bc.le" => "Z != 1 and N == V",
                "cbz" => "register != 0",
                "cbnz" => "register == 0",
                "tbz" => "selected bit != 0",
                "tbnz" => "selected bit != 1",
                _ => "false",
            }
            .to_string()),

            AssemblyInstructionType::Ncti => Err(format!(
                "NCTI Instruction: \"{}\" has no complement branch condition!",
                self.content
            )),
        }
    }
    // TODO: Might be able to increase performance of it later.
    pub fn get_address(&self) -> &str {
        self.content.split_once(":").unwrap().0
    }

    pub fn get_target_ids(&self) -> TargetIdWrapper {
        match &self.inst_type {
            AssemblyInstructionType::CtiConditional(target_id_wrapper) => match target_id_wrapper {
                TargetIdWrapper::Some(target_ids) => TargetIdWrapper::Some(target_ids.clone()),
                TargetIdWrapper::OutOfRange => TargetIdWrapper::OutOfRange,
                TargetIdWrapper::None => {
                    if self.reachable {
                        panic!(
                            "Target Address is not previously extracted for the assembly instruction: \"{}\"",
                            self.content
                        )
                    } else {
                        TargetIdWrapper::None
                    }
                }
            },
            AssemblyInstructionType::CtiUnconditional(target_id_wrapper) => match target_id_wrapper
            {
                TargetIdWrapper::Some(target_ids) => TargetIdWrapper::Some(target_ids.clone()),
                TargetIdWrapper::OutOfRange => TargetIdWrapper::OutOfRange,
                TargetIdWrapper::None => {
                    if self.reachable {
                        panic!(
                            "Target Address is not previously extracted for the assembly instruction: \"{}\"",
                            self.content
                        )
                    } else {
                        TargetIdWrapper::None
                    }
                }
            },
            AssemblyInstructionType::Ncti => panic!(
                "NCTI (Non Control Transfer Instruction) has no target address: \"{}\"",
                self.content
            ),
        }
    }

    fn extract_target_addresses_from_gem5_trace(
        &self,
        trace_path: &str,
        gem5_trace_inst_adr_regex: &Regex,
    ) -> Result<Vec<String>, String> {
        let mut target_addresses = HashSet::new();
        let trace = fs::read_to_string(trace_path).unwrap();
        let mut trace_lines = trace.lines();

        while let Some(current_line) = trace_lines.next() {
            let caps = gem5_trace_inst_adr_regex.captures(current_line).unwrap();
            if caps.get(1).unwrap().as_str() == self.get_address() {
                if let Some(next_line) = trace_lines.next() {
                    let caps = gem5_trace_inst_adr_regex.captures(next_line).unwrap();
                    target_addresses.insert(caps.get(1).unwrap().as_str().to_string());
                }
            }
        }

        if target_addresses.len() > 0 {
            Ok(target_addresses.into_iter().collect::<Vec<String>>())
        } else {
            Err(format!(
                "No dynamic target address was found for the assembly instruction: \"{}\" on line: {}",
                self.content, self.asm_line_number
            ))
        }
    }

    //TODO REMOVE OR NOT?
    /// Returns the line number of the first assembly instruction in the subroutine.
    /// It can return None if the address is outside of the program.
    pub fn get_subroutine_index_from_address(
        &self,
        subroutines: &[Subroutine],
        address: &str,
    ) -> Result<usize, String> {
        for (i, subroutine) in subroutines.iter().enumerate() {
            for asm_line in &subroutine.asm_insts {
                if asm_line.get_address() == address {
                    return Ok(i);
                }
            }
        }

        Err(format!(
            "No subroutine was found with an assembly instruction line with the address: \"{}\"",
            address
        ))
    }

    pub fn next_asm_inst_id(&self, subroutines: &[Subroutine]) -> AssemblyInstructionId {
        if self.id.asm_inst_index + 1 < subroutines[self.id.subroutine_index].asm_insts.len() {
            AssemblyInstructionId {
                subroutine_index: self.id.subroutine_index,
                asm_inst_index: self.id.asm_inst_index + 1,
            }
        } else {
            panic!(
                "No next assembly instruction id was found! Current assembly instruction is: \"{}\"",
                self.content
            );
        }
    }

    pub fn next_asm_inst_address(&self) -> String {
        let next_bb_adr = usize::from_str_radix(self.get_address(), 16).unwrap() + 4;

        format!("{:x}", next_bb_adr)
    }

    /// Returns Ok when a target id is found or the input address is out of the program's adress range.
    pub fn get_asm_inst_id_by_address(
        subroutines: &[Subroutine],
        address: &str,
    ) -> Result<Option<AssemblyInstructionId>, String> {
        for (i, subroutine) in subroutines.iter().enumerate() {
            for (j, asm_line) in subroutine.asm_insts.iter().enumerate() {
                if asm_line.get_address() == address {
                    return Ok(Some(AssemblyInstructionId {
                        subroutine_index: i,
                        asm_inst_index: j,
                    }));
                }
            }
        }

        if AssemblyInstruction::is_target_address_outside_of_the_program_address_range(
            subroutines,
            address,
        ) {
            Ok(None)
        } else {
            Err(format!(
                "No asssembly instruction was found with the address: \"{}\"",
                address
            ))
        }
    }

    // TODO: May have an increase in performance.
    pub fn is_target_address_outside_of_the_program_address_range(
        subroutines: &[Subroutine],
        target_address: &str,
    ) -> bool {
        let target_address = usize::from_str_radix(target_address, 16).unwrap();
        let last_inst_adr = usize::from_str_radix(
            subroutines
                .last()
                .unwrap()
                .asm_insts
                .last()
                .unwrap()
                .get_address(),
            16,
        )
        .unwrap();

        // If the target address value is higher than the last asm_inst address value, it means that it doesn't exist (doesn't belong to our program) and we can ignore it. If not, the program panics.
        target_address > last_inst_adr
    }

    pub fn next_asm_inst_id_in_the_same_subroutine(
        &self,
        subroutines: &[Subroutine],
    ) -> Option<AssemblyInstructionId> {
        let next_asm_inst_id = AssemblyInstructionId {
            subroutine_index: self.id.subroutine_index,
            asm_inst_index: self.id.asm_inst_index + 1,
        };

        if subroutines[next_asm_inst_id.subroutine_index]
            .asm_insts
            .get(next_asm_inst_id.asm_inst_index)
            .is_some()
        {
            Some(next_asm_inst_id)
        } else {
            None
        }
    }

    pub fn is_the_final_instruction_of_the_main_subroutine(
        &self,
        subroutines: &[Subroutine],
        main_sub_index: usize,
    ) -> bool {
        self.id.subroutine_index == main_sub_index
            && (self.name == "ret"
                || self.id.asm_inst_index == subroutines[main_sub_index].asm_insts.len() - 1)
    }
}

async fn extract_target_ids(
    asm_inst: &AssemblyInstruction,
    subroutines: &[Subroutine],
    trace_path: &str,
    gem5_trace_inst_adr_regex: &Regex,
    tx: &Sender<ChannelMessage>,
) -> TargetIdWrapper {
    let mut target_ids = Vec::new();

    match &asm_inst.inst_type {
        AssemblyInstructionType::CtiConditional(_ta) => {
            let target_adr = if asm_inst.name == "tbz" || asm_inst.name == "tbnz" {
                asm_inst.content.split_whitespace().nth(5).unwrap()
            } else if asm_inst.name == "cbz" || asm_inst.name == "cbnz" {
                asm_inst.content.split_whitespace().nth(4).unwrap()
            } else {
                asm_inst.content.split_whitespace().nth(3).unwrap()
            };

            let target_id =
                AssemblyInstruction::get_asm_inst_id_by_address(subroutines, target_adr).unwrap();
            if let Some(target_id) = target_id {
                target_ids.push(target_id);

                TargetIdWrapper::Some(target_ids)
            } else {
                TargetIdWrapper::OutOfRange
            }
        }
        AssemblyInstructionType::CtiUnconditional(_ta) => {
            if asm_inst.name == "blr" || asm_inst.name == "br" || asm_inst.name == "ret" {
                tx.send(ChannelMessage::Text(format!(
                    "Dynamic target address in the assembly instruction: \"{}\" was detected! Trying to resolve it.",
                    asm_inst.content
                )))
                .await
                .unwrap();

                if let Ok(target_adrs) = asm_inst
                    .extract_target_addresses_from_gem5_trace(trace_path, gem5_trace_inst_adr_regex)
                {
                    for target_adr in target_adrs {
                        let target_id = AssemblyInstruction::get_asm_inst_id_by_address(
                            subroutines,
                            &target_adr,
                        )
                        .unwrap();
                        if let Some(target_id) = target_id {
                            target_ids.push(target_id);
                        }
                    }

                    if target_ids.len() > 0 {
                        TargetIdWrapper::Some(target_ids)
                    } else {
                        TargetIdWrapper::OutOfRange
                    }
                } else {
                    TargetIdWrapper::None
                }
            } else {
                let target_adr = asm_inst.content.split_whitespace().nth(3).unwrap();

                let target_id =
                    AssemblyInstruction::get_asm_inst_id_by_address(subroutines, &target_adr)
                        .unwrap();
                if let Some(target_id) = target_id {
                    target_ids.push(target_id);

                    TargetIdWrapper::Some(target_ids)
                } else {
                    TargetIdWrapper::OutOfRange
                }
            }
        }
        AssemblyInstructionType::Ncti => panic!(
            "Error in target ids extraction! Assembly line: '{}' is not a control transfer instruction!",
            asm_inst.content
        ),
    }
}

pub async fn set_target_ids_and_extract_necessary_subroutines_indices(
    main_sub_index: usize,
    subroutines: &mut [Subroutine],
    trace_path: &str,
    gem5_trace_inst_adr_regex: &Regex,
    tx: &Sender<ChannelMessage>,
) -> Vec<usize> {
    let mut j;
    let mut nec_subs_indices_vec_deq = VecDeque::new();
    let mut nec_subs_indices_set = HashSet::new();

    nec_subs_indices_vec_deq.push_back(main_sub_index);
    nec_subs_indices_set.insert(main_sub_index);

    while let Some(i) = nec_subs_indices_vec_deq.pop_front() {
        j = 0;
        while j < subroutines[i].asm_insts.len() {
            let asm_inst = &subroutines[i].asm_insts[j];

            if !asm_inst
                .is_the_final_instruction_of_the_main_subroutine(subroutines, main_sub_index)
            {
                match &asm_inst.inst_type {
                    AssemblyInstructionType::CtiConditional(_)
                    | AssemblyInstructionType::CtiUnconditional(_) => {
                        tx.send(ChannelMessage::Text(format!(
                            "ASM_INST: {}",
                            asm_inst.content
                        )))
                        .await
                        .unwrap();

                        match extract_target_ids(
                            asm_inst,
                            subroutines,
                            trace_path,
                            gem5_trace_inst_adr_regex,
                            tx,
                        )
                        .await
                        {
                            TargetIdWrapper::Some(extracted_target_ids) => {
                                for target_id in &extracted_target_ids {
                                    if nec_subs_indices_set.insert(target_id.subroutine_index) {
                                        nec_subs_indices_vec_deq
                                            .push_back(target_id.subroutine_index);
                                    }

                                    tx.send(ChannelMessage::Text(format!(
                                        "VECDEQ:{:?}",
                                        nec_subs_indices_vec_deq
                                    )))
                                    .await
                                    .unwrap();
                                    tx.send(ChannelMessage::Text(format!(
                                        "TARGETS: {}",
                                        subroutines[target_id.subroutine_index].asm_insts
                                            [target_id.asm_inst_index]
                                            .content
                                    )))
                                    .await
                                    .unwrap();
                                }

                                let asm_inst = &mut subroutines[i].asm_insts[j];
                                match &mut asm_inst.inst_type {
                                    AssemblyInstructionType::CtiConditional(target_ids)
                                    | AssemblyInstructionType::CtiUnconditional(target_ids) => {
                                        *target_ids = TargetIdWrapper::Some(extracted_target_ids);
                                    }
                                    _ => {}
                                }
                            }
                            TargetIdWrapper::OutOfRange => {
                                let asm_inst = &mut subroutines[i].asm_insts[j];
                                match &mut asm_inst.inst_type {
                                    AssemblyInstructionType::CtiConditional(target_ids)
                                    | AssemblyInstructionType::CtiUnconditional(target_ids) => {
                                        *target_ids = TargetIdWrapper::OutOfRange;
                                    }
                                    _ => {}
                                }
                            }
                            TargetIdWrapper::None => {
                                let asm_inst = &mut subroutines[i].asm_insts[j];
                                asm_inst.reachable = false;
                            }
                        }
                    }
                    AssemblyInstructionType::Ncti => {}
                }
            }

            j += 1;
        }
    }

    let mut nec_subs_indices = nec_subs_indices_set.into_iter().collect::<Vec<usize>>();
    nec_subs_indices.sort_unstable();
    nec_subs_indices
}

pub async fn compute_and_flag_leaders(
    main_sub_index: usize,
    subroutines: &mut [Subroutine],
    nec_subs_indices: &[usize],
    tx: &Sender<ChannelMessage>,
) {
    subroutines[main_sub_index].asm_insts[0].leader = true;

    tx.send(ChannelMessage::Text(format!("{}", nec_subs_indices.len())))
        .await
        .unwrap();
    for i in nec_subs_indices {
        tx.send(ChannelMessage::Text(format!(
            "{}",
            subroutines[*i].definition_label
        )))
        .await
        .unwrap();
        for j in 0..subroutines[*i].asm_insts.len() {
            match subroutines[*i].asm_insts[j].inst_type {
                AssemblyInstructionType::CtiConditional(_)
                | AssemblyInstructionType::CtiUnconditional(_) => {
                    if let Some(next_asm_inst_id) = subroutines[*i].asm_insts[j]
                        .next_asm_inst_id_in_the_same_subroutine(subroutines)
                    {
                        subroutines[next_asm_inst_id.subroutine_index].asm_insts
                            [next_asm_inst_id.asm_inst_index]
                            .leader = true;
                    }

                    if !subroutines[*i].asm_insts[j]
                        .is_the_final_instruction_of_the_main_subroutine(
                            subroutines,
                            main_sub_index,
                        )
                    {
                        let target_id_wrapper = subroutines[*i].asm_insts[j].get_target_ids();
                        match target_id_wrapper {
                            TargetIdWrapper::Some(target_ids) => {
                                for target_id in target_ids {
                                    subroutines[target_id.subroutine_index].asm_insts
                                        [target_id.asm_inst_index]
                                        .leader = true;
                                }
                            }
                            TargetIdWrapper::OutOfRange => {}
                            TargetIdWrapper::None => {}
                        }
                    }
                }
                AssemblyInstructionType::Ncti => {}
            }
        }
    }
}
