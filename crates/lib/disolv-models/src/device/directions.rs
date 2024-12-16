use serde::Deserialize;

use disolv_core::agent::AgentClass;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct CommDirections {
    pub stage_name: String,
    pub is_sidelink: bool,
    pub target_classes: Vec<AgentClass>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Directions {
    pub stage_one: CommDirections,
    pub stage_two: CommDirections,
    pub stage_three: CommDirections,
}

impl Directions {
    pub fn new(directions: &Vec<CommDirections>) -> Self {
        let mut stage_one = CommDirections::default();
        let mut stage_two = CommDirections::default();
        let mut stage_three = CommDirections::default();
        directions.clone().iter().for_each(|dir| {
            match dir.stage_name.to_lowercase().as_str() {
                "stage_one" => stage_one = dir.to_owned(),
                "stage_two" => stage_two = dir.to_owned(),
                "stage_three" => stage_three = dir.to_owned(),
                _ => panic!("Invalid stage name passed. Only valid values are: stage_one, stage_two, stage_three")
            }
        });
        Self {
            stage_one,
            stage_two,
            stage_three,
        }
    }
}
