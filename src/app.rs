use std::{
	cell::Cell,
	sync::atomic::{AtomicU32, Ordering},
};

asteracea::component! {
	pub(crate) App(
		render: &'static dyn Fn(),
	)()

	<*Counter *render={render}>
}

asteracea::component! {
	Counter(
		priv render: &'static dyn Fn(),
		count: u32 = 0,
	)() -> !Sync

	|count = AtomicU32::new(count)|;

	<button
		!{self.count.load(Ordering::Relaxed)}
		on bubble click = fn on_click(self, _) {
			self.count.store(self.count.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
			(self.render)()
		}
	>
}
