use log::{debug, error};
use pavenet_core::entity::NodeInfo;
use pavenet_core::message::{DPayload, DataBlob, NodeContent};
use pavenet_core::radio::{Action, ActionType, DActions};

/// Prepares a list of data blobs that the payload should consider forwarding.
///
/// # Arguments
/// * `target_info` - The target node details
/// * `to_forward` - Payloads that requested to be forwarded
///
/// # Returns
/// * `Vec<DataBlob>` - List of data blobs that need to be forwarded
pub fn filter_blobs_to_fwd(target_info: &NodeContent, to_forward: &Vec<DPayload>) -> Vec<DataBlob> {
    let mut blobs_to_forward: Vec<DataBlob> = Vec::new();
    for payload in to_forward.iter() {
        debug!(
            "Checking to forward blob count {} from node {} to node {}",
            payload.metadata.data_blobs.len(),
            payload.node_state.node_info.id,
            target_info.node_info.id
        );
        for blob in payload.metadata.data_blobs.iter() {
            if should_i_forward(blob, &target_info.node_info) {
                blobs_to_forward.push(blob.to_owned());
            } else {
                debug!(
                    "Decided not to forward blob {} from node {} to node {}",
                    blob.data_type, payload.node_state.node_info.id, target_info.node_info.id
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
pub fn set_actions_before_tx(mut payload: DPayload, actions: &DActions) -> DPayload {
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
/// * `node_info` - The node info of the current node
///
/// # Returns
/// * `DPayload` - The payload with the new actions set
pub fn do_actions(payload: &mut DPayload, node_content: &NodeContent) {
    payload
        .metadata
        .data_blobs
        .iter_mut()
        .for_each(|blob| match blob.action.action_type {
            ActionType::Consume => {}
            ActionType::Forward => {
                if am_i_target(&blob.action, &node_content.node_info) {
                    blob.action.action_type = ActionType::Consume;
                }
            }
        });
    payload.metadata.consume();
}

/// Checks if the current node is the intended target of the data
///
/// # Arguments
/// * `action` - The action to check
/// * `node_info` - The node info of the current node
///
/// # Returns
/// * `bool` - True if the current node is the intended target, false otherwise
pub(crate) fn am_i_target(action: &Action, node_info: &NodeInfo) -> bool {
    // Order of precedence: Node -> Class -> Kind
    if let Some(target_node) = action.to_node {
        if target_node == node_info.id {
            return true;
        }
    }
    if let Some(target_class) = action.to_class {
        if target_class == node_info.node_class {
            return true;
        }
    }
    if let Some(target_kind) = action.to_kind {
        if target_kind == node_info.node_type {
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
            if let Some(target_node) = new_action.to_node {
                data_blob.action.to_node = Some(target_node);
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

/// Checks if the current node should forward the data blob
///
/// # Arguments
/// * `blob` - The data blob to check
/// * `target_info` - The node info of the target node
///
/// # Returns
/// * `bool` - True if the current node should forward the data blob, false otherwise
fn should_i_forward(blob: &DataBlob, target_info: &NodeInfo) -> bool {
    if blob.action.action_type == ActionType::Consume {
        error!("This should have been consumed by now");
        panic!("This should have been consumed by now");
    }
    if let Some(target_id) = blob.action.to_node {
        if target_id == target_info.id {
            return true;
        }
    }
    if let Some(class) = blob.action.to_class {
        if class == target_info.node_class {
            return true;
        }
    }
    if let Some(target_kind) = blob.action.to_kind {
        if target_info.node_type == target_kind {
            return true;
        }
    }
    false
}
