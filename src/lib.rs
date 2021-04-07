#![doc(html_root_url = "https://docs.rs/an-asteracea-app/0.0.1")]
#![warn(clippy::pedantic)]

use asteracea::{bumpalo::Bump, lignin, rhizome::Node};
use lignin::auto_safety::{Align, AutoSafe as _, Deanonymize as _};
use lignin_dom::load;
use log::{trace, LevelFilter};
use std::{cell::RefCell, mem::swap, pin::Pin, slice, sync::Mutex};
use wasm_bindgen::prelude::*;

mod app;
use app::App;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn run() {
	#[allow(clippy::main_recursion)]
	main();
}

#[allow(clippy::items_after_statements)]
fn main() {
	console_error_panic_hook::set_once();

	fern::Dispatch::new()
		.format(|out, message, record| {
			out.finish(format_args!(
				"{:5} [{}] {}",
				record.level(),
				record.target(),
				message,
			))
		})
		.level(LevelFilter::Trace)
		.chain(fern::Output::call(console_log::log))
		.apply()
		.expect_throw("Failed to set up logging.");

	enum RootOwner {}
	let mut root = Node::new_for::<RootOwner>();

	struct LoadAllocator<'a>(&'a Bump);
	impl<'a> load::Allocator<'a> for LoadAllocator<'a> {
		fn allocate<T>(&self, instance: T) -> &'a T {
			self.0.alloc(instance)
		}

		fn allocate_slice<T>(&self, iter: &mut dyn ExactSizeIterator<Item = T>) -> &'a [T] {
			self.0.alloc_slice_fill_iter(iter)
		}
	}

	let app: &'static RefCell<Option<Pin<Box<App>>>> = Box::leak(Box::new(RefCell::new(None)));

	let render = {
		let mut bump_a = Bump::new();
		let mut bump_b = Bump::new();
		let app_root = web_sys::window()
			.unwrap_throw()
			.document()
			.unwrap_throw()
			.get_element_by_id("app-root")
			.unwrap_throw();

		let mut previous: lignin::Node<_> = load::load_element(
			&LoadAllocator(unsafe { &*(&bump_a as *const _) }),
			&app_root,
		)
		.content;
		let mut differ = lignin_dom::diff::DomDiffer::new_for_element_child_nodes(app_root);
		let render = Mutex::new(move || {
			trace!("render");

			unsafe {
				let current = app
					.borrow()
					.as_ref()
					.unwrap_throw()
					.as_ref()
					.render(&bump_b, App::render_args_builder().build())
					.unwrap_throw()
					.deanonymize();
				differ.update_child_nodes(
					slice::from_ref(previous.align_ref()),
					slice::from_ref(current.align_ref()),
					1000,
				);
				previous = std::mem::transmute::<lignin::Node<_>, lignin::Node<_>>(current);
				bump_a.reset();
				swap(&mut bump_a, &mut bump_b);
			}
		});
		let render: &dyn Fn() = Box::leak(Box::new(move || (render.lock().unwrap_throw())()));
		render
	};

	*app.borrow_mut() = Some(Box::pin(
		App::new(
			&root.into_arc(),
			App::new_args_builder().render(render).build(),
		)
		.unwrap_throw(),
	));
}
