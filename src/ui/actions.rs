use crate::ui::key::Key;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::slice::Iter;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Action {
    /// The application should exit
    Quit,
    Sleep,
    IncrementDelay,
    DecrementDelay,
}

impl Action {
    /// All available actions
    #[allow(unused)]
    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 4] = [
            Action::Quit,
            Action::Sleep,
            Action::IncrementDelay,
            Action::DecrementDelay,
        ];
        ACTIONS.iter()
    }

    /// List of key associated to action
    pub fn keys(&self) -> &[Key] {
        match self {
            Action::Quit => &[Key::Char('q'), Key::Ctrl('c')],
            Action::Sleep => &[Key::Char('s')],
            Action::IncrementDelay => &[Key::Char('+')],
            Action::DecrementDelay => &[Key::Char('-')],
        }
    }
}

/// Could display a user friendly short description of action
impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Action::Quit => "Quit",
            Action::Sleep => "Sleep",
            Action::IncrementDelay => "Increment delay",
            Action::DecrementDelay => "Decrement delay",
        };
        write!(f, "{}", str)
    }
}

/// A list of actions
///
/// NOTE(manuel) this is useful because we might want to restrict the subset of actions available
/// to a subset of all available actions, I guess maybe if we are in a specific screen or in a
/// specific app mode
#[derive(Default, Debug, Clone)]
pub struct Actions(Vec<Action>);

impl Actions {
    /// Give a key, find the corresponding action
    pub fn find(&self, key: Key) -> Option<&Action> {
        Action::iterator()
            .filter(|action| self.0.contains(action))
            .find(|action| action.keys().contains(&key))
    }

    /// Get contextual actions.
    /// (just for building a help view)
    pub fn actions(&self) -> &[Action] {
        self.0.as_slice()
    }
}

impl From<Vec<Action>> for Actions {
    /// Build contextual actions
    ///
    /// # Panics
    ///
    /// If two actions have the same key
    fn from(actions: Vec<Action>) -> Self {
        let mut map: HashMap<Key, Vec<Action>> = HashMap::new();
        for action in actions.iter() {
            for key in action.keys().iter() {
                match map.get_mut(key) {
                    Some(vec) => vec.push(*action),
                    None => {
                        map.insert(*key, vec![*action]);
                    }
                }
            }
        }
        let errors = map
            .iter()
            .filter(|(_, actions)| actions.len() > 1)
            .map(|(key, actions)| {
                let actions = actions
                    .iter()
                    .map(Action::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("Conflict key {} with actions {}", key, actions)
            })
            .collect::<Vec<_>>();
        if !errors.is_empty() {
            panic!("{}", errors.join("; "))
        }

        Self(actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_find_action_by_key() {
        let actions: Actions = vec![Action::Quit, Action::Sleep].into();
        let result = actions.find(Key::Ctrl('c'));
        assert_eq!(result, Some(&Action::Quit));
    }

    #[test]
    fn should_find_action_by_key_not_ofund() {
        let actions: Actions = vec![Action::Quit, Action::Sleep].into();
        let result = actions.find(Key::Alt('w'));
        assert_eq!(result, None);
    }

    #[test]
    fn should_create_actions_from_vect() {
        let _actions: Actions = vec![
            Action::Quit,
            Action::Sleep,
            Action::IncrementDelay,
            Action::DecrementDelay,
        ]
        .into();
    }

    #[test]
    #[should_panic]
    fn should_panic_if_two_actions_have_the_same_key() {
        let _actions: Actions = vec![
            Action::Quit,
            Action::Sleep,
            Action::IncrementDelay,
            Action::DecrementDelay,
            Action::Sleep,
            Action::IncrementDelay,
        ]
        .into();
    }
}
