use std::{collections::HashMap, fmt::Debug, str::Chars};

use crate::{controller::ControllerState, model::Model, view::View};

pub(super) trait ActionFn: Fn(&mut View, &mut Model, &mut ControllerState) {}
impl<T> ActionFn for T where T: Fn(&mut View, &mut Model, &mut ControllerState) {}
pub(super) type Action = dyn ActionFn;
impl Debug for Action {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "<action>")
	}
}

#[derive(Default, Debug)]
pub struct CommandTrie {
	children: HashMap<char, CommandTrie>,
	action: Option<Box<Action>>,
}

impl CommandTrie {
	/// Add a new function to the Trie
	/// This is a fluent setter
	///
	/// # Panics
	/// If initial command is empty, or has whitespace,
	/// or if final node already has an action
	///
	/// # Examples
	/// ```
	/// let commands: CommandTrie = CommandTrie::default()
	///     .add("j", |_, _, _| {})
	///     .add("k", |_, _, _| {});
	/// ```
	pub fn add<F>(mut self, command: &str, action: F) -> Self
	where
		F: ActionFn + 'static,
	{
		assert!(!(command.is_empty()), "Command must have some char(s)");
		assert!(
			!command.as_bytes().iter().any(u8::is_ascii_whitespace),
			"Command must not have whitespace"
		);

		self.add_recursive(command.chars(), Box::new(action));
		self
	}

	pub fn traverse<I>(&self, chars: I) -> Option<&Self>
	where
		I: IntoIterator<Item = char>,
	{
		let mut node = self;
		for c in chars {
			node = node.children.get(&c)?;
		}
		Some(node)
	}

	pub fn next(&self, char: char) -> Option<&Self> {
		self.children.get(&char)
	}

	pub fn has_children(&self) -> bool {
		!self.children.is_empty()
	}

	pub fn action(&self) -> Option<&Action> {
		self.action.as_deref()
	}

	fn add_recursive(&mut self, mut command: Chars<'_>, action: Box<Action>) {
		if let Some(c) = command.next() {
			let child = self.children.entry(c).or_default();
			child.add_recursive(command, action);
		} else {
			assert!(self.action.is_none(), "Duplicate command found");
			self.action = Some(action);
		}
	}
}
