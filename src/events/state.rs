/// Tweek acts as the coordinator when there are multiple tweens being animated with one or more timelines.
///
use super::clock::*;
use crate::events::*;
// use crate::gui::{gui_print_type, Stage, GUI_NAMES_MAP};

use iced_core::Vector;

use std::{
    any::TypeId,
    collections::{BTreeMap, HashMap},
};

//-- Base -----------------------------------------------------------------------

pub const ID_RANGE_SIZE: u32 = 1000;
pub type NodeTag = u32;
pub type NodeEvent = (NodeTag, EventBox);

/// Mutable state object passed through Responder methods for capturing and handling
/// user events from keyboard and mouse
pub struct AppState {
    /// The size of the window
    pub window_size: (f32, f32),
    /// An instance of the Clock service
    pub clock: Clock,
    /// Ratio value to alter speed of playback, where 1.0 is natural time
    pub time_scale: f32,
    /// Elapsed time
    pub elapsed_time: f64,
    /// Total time
    pub total_time: f64,
    /// Offset x-y when Scene is animating/moving
    pub offset: Vector,
    /// The event queue
    pub event_bus: EventBus,
    /// The observers that have been declared
    pub(crate) observers_map: HashMap<String, Vec<NodePath>>,
    /// Storage where key=Tag and value = NodePath
    pub(crate) node_tags: HashMap<NodeTag, NodePath>,
    /// Outbound notifications, enriched with sender/receiver data
    pub(crate) send_notifications: HashMap<String, NotificationData>,
    /// The hierarchy of NodePaths where Stage is ignored
    pub(crate) node_tree: BTreeMap<String, NodePath>,
    /// A number that stores the next id value to assign through the new_id() function
    next_id: u32,
}

impl AppState {
    /// Constructor
    pub fn new() -> Self {
        let clock = Clock::new();
        AppState {
            window_size: (0.0, 0.0),
            clock,
            time_scale: 1.0,
            elapsed_time: 0.0,
            total_time: 0.0,
            offset: Vector::new(0.0, 0.0),
            event_bus: EventBus::default(),
            node_tags: HashMap::new(),
            observers_map: HashMap::new(),
            send_notifications: HashMap::new(),
            node_tree: BTreeMap::new(),
            next_id: 0,
        }
    }

    /// A method for assigning a globally unique id number for a gui object
    pub fn new_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub(crate) fn _set_next_id(&mut self, next_id: u32) {
        self.next_id = next_id;
    }

    /// Insert a node in the directory
    pub fn append_node(&mut self, node: NodePath) {
        self.node_tree.insert(node.as_string(), node);
    }

    pub(crate) fn _assign_tag(&mut self, tag: NodeTag, path: NodePath) {
        log::trace!("Assigning tag={:?} for path={:?}", tag, path.as_string());
        self.node_tags.insert(tag, path);
    }

    pub(crate) fn _register_observer(&mut self, name: String, observer: NodePath) {
        log::trace!("register_observer name {:?} for {:?}", name, observer.as_string());
        self.observers_map.entry(name).or_insert(Vec::new()).push(observer);
    }

    /// A Displayable Layer can post a notification string
    pub fn post_notification(&mut self, name: &str, sender: NodePath) {
        log::trace!("post_notification received: {} from: {:?}", name, sender.as_string());

        let mut data = NotificationData::with_name(name.to_string());
        data.sender = sender;
        self.send_notifications.insert(name.to_string(), data.clone());

        // For any observers, send a copy with receiver address
        if let Some(observers) = self.observers_map.get(name) {
            for receiver in observers {
                log::trace!("Found observer: {:?}", receiver.as_string());
                data.receiver = receiver.clone();
                self.send_notifications.insert(name.to_string(), data.clone());
            }
        }
    }

    /// Find a notification by name
    pub fn lookup_notification(&self, name: &str) -> Option<NotificationData> {
        if let Some(data) = self.send_notifications.get(name) {
            Some(data.clone())
        } else {
            None
        }
    }

    /// Find the NodePath of an object given the u32 NodeTag that was set
    pub fn find_node_by_tag(&mut self, tag: NodeTag) -> Option<NodePath> {
        if let Some(node_path) = self.node_tags.get(&tag) {
            log::trace!("Found node={:?} with tag={:?}", node_path.as_string(), tag);
            Some(node_path.clone())
        } else {
            None
        }
    }

    /// Hacky way of forcing top-level controller to zero
    pub fn zero_offset(&mut self) {
        self.offset = Vector::new(0.0, 0.0);
    }

    pub fn print_tree(&self) {
        for (path, _) in &self.node_tree {
            log::debug!("{:?}", path);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeID {
    pub id: u32,
    pub type_id: TypeId,
}

impl Default for NodeID {
    fn default() -> Self {
        NodeID { id: 0, type_id: TypeId::of::<NodeID>() }
    }
}

impl NodeID {
    /// Constructor
    pub fn new(id: u32, type_id: TypeId) -> Self {
        NodeID { id, type_id }
    }

    // Print in format like: Scene-1000
    // pub fn id_string(&self) -> String {
    //     format!("{}-{}", gui_print_type(&self.type_id), self.id)
    // }
}

#[derive(Debug, Clone)]
pub struct NodePath {
    /// The path of nodes as an array
    pub(crate) nodes: Vec<NodeID>,
}

impl Default for NodePath {
    fn default() -> Self {
        NodePath { nodes: Vec::new() }
    }
}

impl NodePath {
    /// Constructor
    pub fn new(nodes: Vec<NodeID>) -> Self {
        NodePath { nodes }
    }

    /// Helper to fetch the last node. If there is no last node, then the default
    /// NodeID is the Stage with id=0. Sensible alternative to Option result
    pub fn last_node(&self) -> NodeID {
        if let Some(node) = self.nodes.last() {
            node.clone()
        } else {
            // return noop placeholder
            NodeID::default()
        }
    }

    // Serialize the nodes as a string
    // TODO: Implement from_string() -> NodePath
    pub fn as_string(&self) -> String {
        // TODO: optimize this with write! macro?
        // self.nodes.iter().fold(String::new(), |acc, x| format!("{}/{}", acc, x.id_string()))
        // FIXME: id_string() has been disabled
        String::default()
    }

    // Constructor/helper to convert a string path into a NodePath
    // pub fn from_string(path_text: &str) -> Self {
    //     let mut nodes: Vec<NodeID> = Vec::new();
    //     let parts = path_text.split("/");
    //     for part in parts {
    //         if part.len() > 0 {
    //             let values: Vec<&str> = part.split("-").collect();
    //             if values.len() == 2 {
    //                 let val1 = values[0];
    //                 let val2 = values[1];
    //                 if let Some(type_id) = GUI_NAMES_MAP.get(val1) {
    //                     if let Ok(num) = val2.parse::<u32>() {
    //                         let node = NodeID::new(num, *type_id);
    //                         nodes.push(node);
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     NodePath { nodes }
    // }
}

/// A wrapper to hold data
#[derive(Debug, Clone)]
pub struct NotificationData {
    pub name: String,
    pub sender: NodePath,
    pub receiver: NodePath,
    pub info: Option<String>,
}

impl NotificationData {
    /// Constructor using only the name
    pub fn with_name(name: String) -> Self {
        NotificationData { name, sender: NodePath::default(), receiver: NodePath::default(), info: None }
    }
}
