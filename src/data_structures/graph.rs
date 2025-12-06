use crate::{
	crypto::{BuildHasher, DefaultHashBuilder, Hash},
	data_structures::IndexSize,
};

pub struct Graph<
	T,
	S: const IndexSize = usize,
	A: Allocator + Clone = GlobalAllocator,
	H: BuildHasher + Clone = DefaultHashBuilder,
> {
	pub items: SizedVec<T, S, A>,
	pub links: SizedVec<HashSet<GraphNode<S>, H, A>, S, A>,
	pub backlinks: SizedVec<HashSet<GraphNode<S>, H, A>, S, A>,
	pub allocator: A,
	pub hasher: H,
}
impl<T, S: const IndexSize> Default for Graph<T, S, GlobalAllocator, DefaultHashBuilder> {
	fn default() -> Self {
		Self {
			items: Default::default(),
			links: Default::default(),
			backlinks: Default::default(),
			allocator: Default::default(),
			hasher: Default::default(),
		}
	}
}
impl<T, S: const IndexSize> Graph<T, S, GlobalAllocator, DefaultHashBuilder> {
	pub fn new() -> Self {
		Default::default()
	}
}
impl<T, S: const IndexSize, A: Allocator + Clone> Graph<T, S, A, DefaultHashBuilder> {
	pub fn with_allocator(allocator: A) -> Self {
		Self {
			items: SizedVec::with_allocator(allocator.clone()),
			links: SizedVec::with_allocator(allocator.clone()),
			backlinks: SizedVec::with_allocator(allocator.clone()),
			allocator,
			hasher: DefaultHashBuilder::default(),
		}
	}
}
impl<T, S: const IndexSize, H: BuildHasher + Clone> Graph<T, S, GlobalAllocator, H> {
	pub fn with_hasher(hasher: H) -> Self {
		Self {
			items: SizedVec::new(),
			links: SizedVec::new(),
			backlinks: SizedVec::new(),
			allocator: GlobalAllocator,
			hasher,
		}
	}
}
impl<T, S: const IndexSize, A: Allocator + Clone, H: BuildHasher + Clone> Graph<T, S, A, H> {
	pub fn with_hasher_and_allocator(hasher: H, allocator: A) -> Self {
		Self {
			items: SizedVec::with_allocator(allocator.clone()),
			links: SizedVec::with_allocator(allocator.clone()),
			backlinks: SizedVec::with_allocator(allocator.clone()),
			allocator,
			hasher,
		}
	}

	pub fn add_node(&mut self, item: T) -> GraphNode<S> {
		let id = self.items.len();
		self.items.push(item);
		self.links.push(HashSet::with_hasher_in(
			self.hasher.clone(),
			self.allocator.clone(),
		));
		self.backlinks.push(HashSet::with_hasher_in(
			self.hasher.clone(),
			self.allocator.clone(),
		));

		unsafe { GraphNode::new(id) }
	}

	/// Removes all links to this node, isolating it on the graph.
	pub fn isolate(&mut self, node: GraphNode<S>) {
		let links = unsafe { self.links.get_mut_unchecked(node.raw()) };
		for link in links.iter() {
			unsafe { self.backlinks.get_mut_unchecked(link.raw()) }.remove(&node);
		}
		links.clear();

		let backlinks = unsafe { self.backlinks.get_mut_unchecked(node.raw()) };
		for backlink in backlinks.iter() {
			unsafe { self.links.get_mut_unchecked(backlink.raw()) }.remove(&node);
		}
		backlinks.clear();
	}

	pub fn add_link(&mut self, a: GraphNode<S>, b: GraphNode<S>) {
		unsafe { self.links.get_mut_unchecked(a.raw()) }.insert(b);
		unsafe { self.backlinks.get_mut_unchecked(b.raw()) }.insert(a);
	}
	pub fn remove_link(&mut self, a: GraphNode<S>, b: GraphNode<S>) {
		unsafe { self.links.get_mut_unchecked(a.raw()) }.remove(&b);
		unsafe { self.backlinks.get_mut_unchecked(b.raw()) }.remove(&a);
	}

	pub fn get_links(&self, node: GraphNode<S>) -> impl Iterator<Item = GraphNode<S>> {
		unsafe { self.links.get_unchecked(node.raw()) }
			.iter()
			.copied()
	}
	pub fn get_linked_items(&self, node: GraphNode<S>) -> impl Iterator<Item = &T> {
		self.get_links(node).map(|node| self.get_node(node))
	}

	pub fn get_backlinks(&self, node: GraphNode<S>) -> impl Iterator<Item = &GraphNode<S>> {
		unsafe { self.backlinks.get_unchecked(node.raw()) }.iter()
	}
	pub fn get_backlinked_items(&self, node: GraphNode<S>) -> impl Iterator<Item = &T> {
		self.get_backlinks(node).map(|node| self.get_node(*node))
	}

	pub fn get_node(&self, node: GraphNode<S>) -> &T {
		unsafe { self.items.get_unchecked(node.raw()) }
	}
	pub fn get_node_mut(&mut self, node: GraphNode<S>) -> &mut T {
		unsafe { self.items.get_mut_unchecked(node.raw()) }
	}

	pub fn all_nodes(&self) -> impl Iterator<Item = GraphNode<S>> {
		(S::ZERO..self.items.len())
			.into_iter()
			.map(|id| unsafe { GraphNode::new(id) })
	}
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct GraphNode<S: const IndexSize = usize>(S);
impl<I: const IndexSize> GraphNode<I> {
	pub unsafe fn new(raw: I) -> Self {
		Self(raw)
	}
	pub fn raw(self) -> I {
		self.0
	}
}
