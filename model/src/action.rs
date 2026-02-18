use super::Exception;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use url::Url;
use uuid::Uuid;

/// State of an action
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActionState {
    /// Action is waiting to be executed
    #[default]
    Pending,
    /// Action is currently running
    Running,
    /// Action completed successfully
    Complete,
    /// Action failed with an error
    Failed,
    /// Action was skipped (e.g., dependency failed)
    Skipped,
    /// Action state is unknown
    Unknown,
}

/// Unique identifier for an action in the DAG
pub type ActionId = u32;

/// A single action node in the DAG
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Action {
    /// Numeric ID for indexing
    pub id: ActionId,
    /// UUID for external references
    pub uuid: Uuid,
    /// Optional URL for more info
    pub url: Option<Url>,
    /// Human-readable description
    pub message: String,
    /// Current state
    pub state: ActionState,
    /// Error information if failed
    pub exception: Option<Exception>,
    /// IDs of actions that must complete before this one
    pub dependencies: Vec<ActionId>,
    /// IDs of actions that depend on this one
    pub dependents: Vec<ActionId>,
    /// Priority (higher = more important)
    pub priority: i32,
    /// Estimated duration in seconds (for scheduling)
    pub estimated_duration: Option<u32>,
    /// Actual duration in seconds (after completion)
    pub actual_duration: Option<u32>,
}

impl Action {
    /// Create a new action with the given message
    pub fn new(id: ActionId, message: impl Into<String>) -> Self {
        Self {
            id,
            uuid: Uuid::new_v4(),
            url: None,
            message: message.into(),
            state: ActionState::Pending,
            exception: None,
            dependencies: Vec::new(),
            dependents: Vec::new(),
            priority: 0,
            estimated_duration: None,
            actual_duration: None,
        }
    }

    /// Check if this action is ready to execute (all dependencies complete)
    pub fn is_ready(&self, dag: &ActionDag) -> bool {
        if self.state != ActionState::Pending {
            return false;
        }

        self.dependencies.iter().all(|dep_id| {
            dag.get(*dep_id)
                .map(|dep| dep.state == ActionState::Complete)
                .unwrap_or(false)
        })
    }

    /// Check if this action can be skipped (a dependency failed)
    pub fn should_skip(&self, dag: &ActionDag) -> bool {
        self.dependencies.iter().any(|dep_id| {
            dag.get(*dep_id)
                .map(|dep| matches!(dep.state, ActionState::Failed | ActionState::Skipped))
                .unwrap_or(false)
        })
    }
}

/// Directed Acyclic Graph of actions
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ActionDag {
    /// All actions indexed by ID
    actions: HashMap<ActionId, Action>,
    /// Next available action ID
    next_id: ActionId,
    /// Root actions (no dependencies)
    roots: Vec<ActionId>,
}

impl ActionDag {
    /// Create a new empty DAG
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
            next_id: 0,
            roots: Vec::new(),
        }
    }

    /// Add a new action and return its ID
    pub fn add_action(&mut self, message: impl Into<String>) -> ActionId {
        let id = self.next_id;
        self.next_id += 1;

        let action = Action::new(id, message);
        self.actions.insert(id, action);
        self.roots.push(id);

        id
    }

    /// Add a dependency between two actions
    ///
    /// Returns an error if this would create a cycle
    pub fn add_dependency(
        &mut self,
        action_id: ActionId,
        depends_on: ActionId,
    ) -> Result<(), String> {
        // Check for self-dependency
        if action_id == depends_on {
            return Err("Action cannot depend on itself".to_string());
        }

        // Check if both actions exist
        if !self.actions.contains_key(&action_id) {
            return Err(format!("Action {} not found", action_id));
        }
        if !self.actions.contains_key(&depends_on) {
            return Err(format!("Dependency action {} not found", depends_on));
        }

        // Check for cycles
        if self.would_create_cycle(action_id, depends_on) {
            return Err("Adding this dependency would create a cycle".to_string());
        }

        // Add the dependency
        if let Some(action) = self.actions.get_mut(&action_id) {
            if !action.dependencies.contains(&depends_on) {
                action.dependencies.push(depends_on);
            }
        }

        // Add the reverse reference
        if let Some(dep) = self.actions.get_mut(&depends_on) {
            if !dep.dependents.contains(&action_id) {
                dep.dependents.push(action_id);
            }
        }

        // Remove from roots if it now has dependencies
        self.roots.retain(|&id| id != action_id);

        Ok(())
    }

    /// Check if adding a dependency would create a cycle
    fn would_create_cycle(&self, from: ActionId, to: ActionId) -> bool {
        // DFS from 'to' to see if we can reach 'from'
        let mut visited = HashSet::new();
        let mut stack = vec![to];

        while let Some(current) = stack.pop() {
            if current == from {
                return true;
            }

            if visited.insert(current) {
                if let Some(action) = self.actions.get(&current) {
                    stack.extend(&action.dependencies);
                }
            }
        }

        false
    }

    /// Get an action by ID
    pub fn get(&self, id: ActionId) -> Option<&Action> {
        self.actions.get(&id)
    }

    /// Get a mutable reference to an action
    pub fn get_mut(&mut self, id: ActionId) -> Option<&mut Action> {
        self.actions.get_mut(&id)
    }

    /// Get all actions that are ready to execute
    pub fn ready_actions(&self) -> Vec<ActionId> {
        self.actions
            .values()
            .filter(|action| action.is_ready(self))
            .map(|action| action.id)
            .collect()
    }

    /// Get a topologically sorted list of action IDs
    pub fn topological_sort(&self) -> Result<Vec<ActionId>, String> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        for &id in self.roots.iter() {
            self.topological_visit(id, &mut visited, &mut temp_visited, &mut result)?;
        }

        // Also visit any disconnected nodes
        for &id in self.actions.keys() {
            if !visited.contains(&id) {
                self.topological_visit(id, &mut visited, &mut temp_visited, &mut result)?;
            }
        }

        Ok(result)
    }

    fn topological_visit(
        &self,
        id: ActionId,
        visited: &mut HashSet<ActionId>,
        temp_visited: &mut HashSet<ActionId>,
        result: &mut Vec<ActionId>,
    ) -> Result<(), String> {
        if temp_visited.contains(&id) {
            return Err("Cycle detected in action graph".to_string());
        }
        if visited.contains(&id) {
            return Ok(());
        }

        temp_visited.insert(id);

        if let Some(action) = self.actions.get(&id) {
            for &dep_id in &action.dependents {
                self.topological_visit(dep_id, visited, temp_visited, result)?;
            }
        }

        temp_visited.remove(&id);
        visited.insert(id);
        result.push(id);

        Ok(())
    }

    /// Execute actions in order using a provided executor function
    ///
    /// Returns the number of successful, failed, and skipped actions
    pub fn execute<F>(&mut self, mut executor: F) -> (usize, usize, usize)
    where
        F: FnMut(&mut Action) -> Result<(), String>,
    {
        let order = match self.topological_sort() {
            Ok(order) => order,
            Err(_) => return (0, 0, 0),
        };

        let mut success = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for id in order.into_iter().rev() {
            // Check if we should skip due to failed dependencies
            let should_skip = self.get(id).map(|a| a.should_skip(self)).unwrap_or(true);

            if should_skip {
                if let Some(action) = self.get_mut(id) {
                    action.state = ActionState::Skipped;
                    skipped += 1;
                }
                continue;
            }

            // Execute the action
            if let Some(action) = self.get_mut(id) {
                action.state = ActionState::Running;
                let start = std::time::Instant::now();

                match executor(action) {
                    Ok(()) => {
                        action.state = ActionState::Complete;
                        action.actual_duration = Some(start.elapsed().as_secs() as u32);
                        success += 1;
                    }
                    Err(msg) => {
                        action.state = ActionState::Failed;
                        action.exception = Some(Exception {
                            message: msg,
                            ..Default::default()
                        });
                        action.actual_duration = Some(start.elapsed().as_secs() as u32);
                        failed += 1;
                    }
                }
            }
        }

        (success, failed, skipped)
    }

    /// Get statistics about the DAG
    pub fn stats(&self) -> DagStats {
        let mut pending = 0;
        let mut running = 0;
        let mut complete = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for action in self.actions.values() {
            match action.state {
                ActionState::Pending => pending += 1,
                ActionState::Running => running += 1,
                ActionState::Complete => complete += 1,
                ActionState::Failed => failed += 1,
                ActionState::Skipped => skipped += 1,
                ActionState::Unknown => {}
            }
        }

        DagStats {
            total: self.actions.len(),
            pending,
            running,
            complete,
            failed,
            skipped,
            roots: self.roots.len(),
        }
    }

    /// Get all action IDs
    pub fn action_ids(&self) -> Vec<ActionId> {
        self.actions.keys().copied().collect()
    }

    /// Get total number of actions
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// Check if DAG is empty
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// Clear all actions
    pub fn clear(&mut self) {
        self.actions.clear();
        self.roots.clear();
        self.next_id = 0;
    }
}

/// Statistics about the action DAG
#[derive(Debug, Clone)]
pub struct DagStats {
    pub total: usize,
    pub pending: usize,
    pub running: usize,
    pub complete: usize,
    pub failed: usize,
    pub skipped: usize,
    pub roots: usize,
}

/// Legacy type alias for backwards compatibility
pub type Actions = Vec<Action>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_dag() {
        let mut dag = ActionDag::new();
        let a = dag.add_action("Action A");
        let b = dag.add_action("Action B");
        let c = dag.add_action("Action C");

        assert_eq!(dag.len(), 3);
        assert_eq!(dag.roots.len(), 3);

        // B depends on A
        dag.add_dependency(b, a).unwrap();
        assert_eq!(dag.roots.len(), 2);

        // C depends on B
        dag.add_dependency(c, b).unwrap();
        assert_eq!(dag.roots.len(), 1);
    }

    #[test]
    fn test_cycle_detection() {
        let mut dag = ActionDag::new();
        let a = dag.add_action("A");
        let b = dag.add_action("B");
        let c = dag.add_action("C");

        dag.add_dependency(b, a).unwrap();
        dag.add_dependency(c, b).unwrap();

        // This would create a cycle: A -> B -> C -> A
        let result = dag.add_dependency(a, c);
        assert!(result.is_err());
    }

    #[test]
    fn test_topological_sort() {
        let mut dag = ActionDag::new();
        let a = dag.add_action("A");
        let b = dag.add_action("B");
        let c = dag.add_action("C");

        dag.add_dependency(b, a).unwrap();
        dag.add_dependency(c, b).unwrap();

        let order = dag.topological_sort().unwrap();
        // A should come before B, B before C
        let pos_a = order.iter().position(|&x| x == a).unwrap();
        let pos_b = order.iter().position(|&x| x == b).unwrap();
        let pos_c = order.iter().position(|&x| x == c).unwrap();

        assert!(pos_a > pos_b);
        assert!(pos_b > pos_c);
    }

    #[test]
    fn test_ready_actions() {
        let mut dag = ActionDag::new();
        let a = dag.add_action("A");
        let b = dag.add_action("B");

        dag.add_dependency(b, a).unwrap();

        // Only A should be ready initially
        let ready = dag.ready_actions();
        assert_eq!(ready.len(), 1);
        assert!(ready.contains(&a));

        // Mark A as complete
        dag.get_mut(a).unwrap().state = ActionState::Complete;

        // Now B should be ready
        let ready = dag.ready_actions();
        assert_eq!(ready.len(), 1);
        assert!(ready.contains(&b));
    }
}
