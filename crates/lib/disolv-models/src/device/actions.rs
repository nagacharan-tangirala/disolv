use log::{debug, error};

use crate::device::types::DeviceInfo;
use crate::net::message::{DataBlob, DeviceContent, V2XPayload};
use crate::net::radio::{Action, ActionType, DActions};

/// Prepares a list of data blobs that the payload should consider forwarding.
///
/// # Arguments
/// * `target_info` - The target agent details
/// * `to_forward` - Payloads that requested to be forwarded
///
/// # Returns
/// * `Vec<DataBlob>` - List of data blobs that need to be forwarded
pub fn filter_blobs_to_fwd(
    target_info: &DeviceContent,
    to_forward: &Vec<V2XPayload>,
) -> Vec<DataBlob> {
    let mut blobs_to_forward: Vec<DataBlob> = Vec::new();
    for payload in to_forward.iter() {
        debug!(
            "Checking to forward blob count {} from agent {} to agent {}",
            payload.metadata.data_blobs.len(),
            payload.agent_state.device_info.id,
            target_info.device_info.id
        );
        for blob in payload.metadata.data_blobs.iter() {
            if should_i_forward(blob, &target_info.device_info) {
                blobs_to_forward.push(blob.to_owned());
            } else {
                debug!(
                    "Decided not to forward blob {} from agent {} to agent {}",
                    blob.data_type, payload.agent_state.device_info.id, target_info.device_info.id
                );
            }
        }
    }
    blobs_to_forward
}

/// Assigns the actions to the data blobs in the payload. This is done by the sender
/// as a last step before sending the payload.
///
/// # Arguments
/// * `payload` - The payload to set actions for
/// * `actions` - The actions to set
///
/// # Returns
/// * `DPayload` - The payload with the new actions set
pub fn set_actions_before_tx(mut payload: V2XPayload, actions: &DActions) -> V2XPayload {
    payload.metadata.data_blobs.iter_mut().for_each(|blob| {
        let new_action = match actions.action_for(&blob.data_type) {
            Some(action) => action,
            None => {
                error!("No action found for data type {}", blob.data_type);
                panic!("Action missing for data type {}", blob.data_type);
            }
        };
        assign_actions(blob, new_action);
    });
    payload
}

/// Performs the actions instructed by the sender.
/// At this point, we have received the data blobs with actions instructed by the sender.
/// We need to apply these actions to the data blobs and set the actions for the next hop.
///
/// # Arguments
/// * `payload` - The payload to set actions for
/// * `agent_info` - The agent info of the current agent
///
/// # Returns
/// * `DPayload` - The payload with the new actions set
pub fn do_actions(payload: &mut V2XPayload, agent_content: &DeviceContent) {
    payload
        .metadata
        .data_blobs
        .iter_mut()
        .for_each(|blob| match blob.action.action_type {
            ActionType::Consume => {}
            ActionType::Forward => {
                if am_i_target(&blob.action, &agent_content.device_info) {
                    blob.action.action_type = ActionType::Consume;
                }
            }
        });
    payload.metadata.consume();
}

/// Checks if the current agent is the intended target of the data
///
/// # Arguments
/// * `action` - The action to check
/// * `agent_info` - The agent info of the current agent
///
/// # Returns
/// * `bool` - True if the current agent is the intended target, false otherwise
pub(crate) fn am_i_target(action: &Action, agent_info: &DeviceInfo) -> bool {
    // Order of precedence: Agent -> Class -> Kind
    if let Some(target_agent) = action.to_agent {
        if target_agent == agent_info.id {
            return true;
        }
    }
    if let Some(target_class) = action.to_class {
        if target_class == agent_info.device_class {
            return true;
        }
    }
    if let Some(target_kind) = action.to_kind {
        if target_kind == agent_info.device_type {
            return true;
        }
    }
    false
}

/// Sets the new action for the data blob
///
/// # Arguments
/// * `data_blob` - The data blob to set the action for
/// * `new_action` - The new action to set
fn assign_actions(data_blob: &mut DataBlob, new_action: &Action) {
    match new_action.action_type {
        ActionType::Consume => {
            data_blob.action.action_type = ActionType::Consume;
        }
        ActionType::Forward => {
            debug!("Assigning forward action to blob {}", data_blob.data_type);
            if let Some(target_agent) = new_action.to_agent {
                data_blob.action.to_agent = Some(target_agent);
            }
            if let Some(target_class) = new_action.to_class {
                data_blob.action.to_class = Some(target_class);
            }
            if let Some(target_kind) = new_action.to_kind {
                data_blob.action.to_kind = Some(target_kind);
            }
            data_blob.action.action_type = ActionType::Forward;
        }
    };
}

/// Checks if the current agent should forward the data blob
///
/// # Arguments
/// * `blob` - The data blob to check
/// * `target_info` - The agent info of the target agent
///
/// # Returns
/// * `bool` - True if the current agent should forward the data blob, false otherwise
fn should_i_forward(blob: &DataBlob, target_info: &DeviceInfo) -> bool {
    if blob.action.action_type == ActionType::Consume {
        error!("This should have been consumed by now");
        panic!("This should have been consumed by now");
    }
    if let Some(target_id) = blob.action.to_agent {
        if target_id == target_info.id {
            return true;
        }
    }
    if let Some(class) = blob.action.to_class {
        if class == target_info.device_class {
            return true;
        }
    }
    if let Some(target_kind) = blob.action.to_kind {
        if target_info.device_type == target_kind {
            return true;
        }
    }
    false
}
