use log::{debug, error};

use disolv_core::agent::{AgentId, AgentProperties};
use disolv_core::hashbrown::HashMap;
use disolv_core::message::{ContentType, DataUnit, Metadata, Payload, QueryType};
use disolv_core::radio::{Action, ActionType};

/// Prepares a list of data units that the payload should consider forwarding.
///
/// # Arguments
/// * `target_info` - The target agent details
/// * `to_forward` - Payloads that requested to be forwarded
///
/// # Returns
/// * `Vec<DataUnit>` - List of data units that need to be forwarded
pub fn filter_units_to_fwd<
    C: ContentType,
    D: DataUnit<C>,
    P: AgentProperties,
    M: Metadata,
    Q: QueryType,
>(
    target_info: &P,
    to_forward: &[Payload<C, D, M, P, Q>],
) -> Vec<D> {
    let mut units_to_forward: Vec<D> = Vec::new();
    for payload in to_forward.iter() {
        for unit in payload.data_units.iter() {
            if should_i_forward(unit, target_info) {
                debug!(
                    "Decided to forward unit of type {} from agent {} to agent {}",
                    unit.content_type(),
                    payload.agent_state.id(),
                    target_info.id()
                );
                units_to_forward.push(unit.to_owned());
            }
        }
    }
    units_to_forward
}

/// Assigns the actions to the data units in the payload. This is done by the sender
/// as a last step before sending the payload.
///
/// # Arguments
/// * `payload` - The payload to set actions for
/// * `actions` - The actions to set
///
/// # Returns
/// * `DPayload` - The payload with the new actions set
pub fn set_actions_before_tx<
    C: ContentType,
    D: DataUnit<C>,
    P: AgentProperties,
    M: Metadata,
    Q: QueryType,
>(
    mut payload: Payload<C, D, M, P, Q>,
    target_id: AgentId,
    actions: &HashMap<C, Action>,
) -> Payload<C, D, M, P, Q> {
    payload.data_units.iter_mut().for_each(|unit| {
        match actions.get(unit.content_type()) {
            Some(action) => {
                if action.clone().action_type == ActionType::Consume {
                    let mut new_action = Action::with_action_type(ActionType::Consume);
                    new_action.to_agent = Some(target_id);
                    unit.update_action(&new_action);
                }
                unit.update_action(action);
            }
            None => {
                error!("No action found for data type {}", unit.content_type());
                panic!("Action missing for data type {}", unit.content_type());
            }
        };
    });
    payload
}

/// Performs the actions instructed by the sender.
/// At this point, we have received the data units with actions instructed by the sender.
/// We need to apply these actions to the data units and set the actions for the next hop.
///
/// # Arguments
/// * `payload` - The payload to set actions for
/// * `agent_info` - The agent info of the current agent
///
/// # Returns
/// * `DPayload` - The payload with the new actions set
pub fn complete_actions<
    C: ContentType,
    D: DataUnit<C>,
    P: AgentProperties,
    M: Metadata,
    Q: QueryType,
>(
    payload: &mut Payload<C, D, M, P, Q>,
    agent_content: &P,
) {
    payload
        .data_units
        .iter_mut()
        .for_each(|unit| match unit.action().action_type {
            ActionType::Consume => {}
            ActionType::Forward => {
                if am_i_target(unit.action(), agent_content) {
                    let new_action = Action::with_action_type(ActionType::Consume);
                    unit.update_action(&new_action);
                }
            }
        });
    payload.consume();
}

/// Checks if the current agent is the intended target of the data
///
/// # Arguments
/// * `action` - The action to check
/// * `agent_info` - The agent info of the current agent
///
/// # Returns
/// * `bool` - True if the current agent is the intended target, false otherwise
pub fn am_i_target<P: AgentProperties>(action: &Action, agent_info: &P) -> bool {
    // Order of precedence: Agent -> Broadcast -> Class -> Kind
    if let Some(target_agent) = action.to_agent {
        return target_agent == agent_info.id();
    }
    if let Some(broadcast) = &action.to_broadcast {
        return broadcast.contains(&agent_info.id());
    }
    if let Some(target_class) = action.to_class {
        return target_class == *agent_info.class();
    }
    if let Some(target_kind) = action.to_kind {
        return target_kind == *agent_info.kind();
    }
    false
}

/// Checks if the current agent should forward the data unit
///
/// # Arguments
/// * `unit` - The data unit to check
/// * `target_info` - The agent info of the target agent
///
/// # Returns
/// * `bool` - True if the current agent should forward the data unit, false otherwise
fn should_i_forward<C: ContentType, D: DataUnit<C>, P: AgentProperties>(
    unit: &D,
    target_info: &P,
) -> bool {
    if unit.action().action_type == ActionType::Consume {
        error!("This should have been consumed by now");
        panic!("This should have been consumed by now");
    }
    if let Some(id) = unit.action().to_agent {
        return id == target_info.id();
    }
    if let Some(broadcast) = &unit.action().to_broadcast {
        return broadcast.contains(&target_info.id());
    }
    if let Some(class) = unit.action().to_class {
        return class == *target_info.class();
    }
    if let Some(kind) = unit.action().to_kind {
        return kind == *target_info.kind();
    }
    false
}
