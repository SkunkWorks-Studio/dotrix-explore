#![allow(dead_code, unused_imports, unused_variables, unused_mut)]

use dotrix::assets::Mesh;
use dotrix::camera;
use dotrix::egui::{self, Egui};
use dotrix::input::{ActionMapper, Button, KeyCode, Mapper};
use dotrix::math::{Point3, Vec3};
use dotrix::overlay::{self, Overlay};
use dotrix::pbr::{self, Light};
use dotrix::prelude::*;
use dotrix::sky::{skybox, SkyBox};
use dotrix::{Animator, Assets, Camera, Color, CubeMap, Frame, Input, Pipeline, State, Transform, Window, World};

const DEBUG_YELLOW: egui::Rgba = egui::Rgba::from_rgb(255.0, 255.0, 0.0);
const PAN_SPEED: f32 = 30.0;
const SCROLL_SPEED: f32 = 60.0;

struct MainState {
	name: String,
	positions: Vec<[f32; 3]>,
}

struct PauseState {
	name: String,
	handled: bool,
}

struct Player {}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
enum Action {
	TogglePause,
	Exit,
	PanUp,
	PanDown,
	PanLeft,
	PanRight,
}

impl ActionMapper<Action> for Input {
	fn action_mapped(&self, action: Action) -> Option<&Button> {
		let mapper = self.mapper::<Mapper<Action>>();
		mapper.get_button(action)
	}
}

fn main() {
	Dotrix::application("Isometric TD Tech Demo")
		.with(System::from(startup))
		.with(System::from(ui_main).with(State::off::<PauseState>()))
		.with(System::from(ui_paused).with(State::on::<PauseState>()))
		.with(System::from(player_control).with(State::on::<MainState>()))
		.with(System::from(global_control))
		.with(overlay::extension)
		.with(egui::extension)
		.with(skybox::extension)
		.with(pbr::extension)
		.run();
}

fn startup(mut assets: Mut<Assets>, mut input: Mut<Input>, mut state: Mut<State>, mut world: Mut<World>, mut window: Mut<Window>, mut camera: Mut<Camera>) {
	window.set_cursor_grab(true);
	camera.target.y = -8.5;
	camera.xz_angle = 1.2;

	init_input(&mut input);
	init_skybox(&mut assets, &mut world);
	init_terrain(&mut assets, &mut world, &mut state);
	init_lights(&mut world);
}

fn init_input(input: &mut Input) {
	input.set_mapper(Box::new(Mapper::<Action>::new()));
	input
		.mapper_mut::<Mapper<Action>>()
		.set(vec![
			(Action::TogglePause, Button::Key(KeyCode::Escape)),
			(Action::Exit, Button::Key(KeyCode::C)),
			(Action::PanUp, Button::Key(KeyCode::W)),
			(Action::PanDown, Button::Key(KeyCode::S)),
			(Action::PanLeft, Button::Key(KeyCode::A)),
			(Action::PanRight, Button::Key(KeyCode::D)),
		]);
}

fn init_terrain(assets: &mut Assets, world: &mut World, state: &mut State) {
	// Generate terrain mesh like this:
	//   0   1
	// 0 +---+---+---> x
	//   | / | / |
	// 1 +---+---+
	//   | / | / |
	//   +---+---+
	//   |
	//   z

	let size = 5;
	let mut positions = Vec::with_capacity(3 * 2 * size * size);
	let mut uvs = Vec::new();
	for x in 0..size {
		let x0 = x as f32;
		let x1 = x0 + 1.0;
		for z in 0..size {
			let z0 = z as f32;
			let z1 = z0 + 1.0;
			// Add vertices
			positions.push([x0, 0.0, z0]);
			positions.push([x0, 0.0, z1]);
			positions.push([x1, 0.0, z0]);
			positions.push([x1, 0.0, z0]);
			positions.push([x0, 0.0, z1]);
			positions.push([x1, 0.0, z1]);
			// Add texture vertices
			uvs.push([0.0, 0.0]);
			uvs.push([0.0, 1.0]);
			uvs.push([1.0, 0.0]);
			uvs.push([1.0, 0.0]);
			uvs.push([0.0, 1.0]);
			uvs.push([1.0, 1.0]);
		}
	}

	let normals = Mesh::calculate_normals(&positions, None);

	let mut mesh = Mesh::default();

	mesh.with_vertices(&positions);
	mesh.with_vertices(&normals);
	mesh.with_vertices(&uvs);

	// Store mesh and get its ID
	let mesh = assets.store_as(mesh, "terrain");

	// import terrain texture and get its ID
	assets.import("assets/terrain.png");
	let texture = assets.register("terrain");

	// Center terrain tile at coordinate system center (0.0, 0.0, 0.0) by moving the tile on a
	// half of its size by X and Z axis
	let shift = (size / 2) as f32;

	world.spawn(
		(pbr::solid::Entity {
			mesh,
			texture,
			translate: Vec3::new(-shift, 0.0, -shift),
			..Default::default()
		})
		.some(),
	);

	state.push(MainState {
		name: String::from("Main State"),
		positions: positions,
	});
}

fn init_lights(world: &mut World) {
	// spawn source of white light at (0.0, 100.0, 0.0)
	world.spawn(Some((Light::Simple {
		// direction: Vec3::new(0.3, -0.5, -0.6),
		position: Vec3::new(0.0, 1000.0, 0.0),
		color: Color::white(),
		intensity: 0.5,
		enabled: true,
	},)));
	// spawn source of white light at (0.0, 100.0, 0.0)
	world.spawn(Some((Light::Ambient {
		color: Color::white(),
		intensity: 0.5,
	},)));
}

fn init_skybox(assets: &mut Assets, world: &mut World) {
	let asset_list = &[
		"assets/skybox_right.png",
		"assets/skybox_left.png",
		"assets/skybox_top.png",
		"assets/skybox_bottom.png",
		"assets/skybox_back.png",
		"assets/skybox_front.png",
	];

	asset_list
		.into_iter()
		.for_each(|asset| {
			assets.import(asset);
		});
	world.spawn(Some((
		SkyBox {
			view_range: 500.0,
			..Default::default()
		},
		CubeMap {
			right: assets.register("skybox_right"),
			left: assets.register("skybox_left"),
			top: assets.register("skybox_top"),
			bottom: assets.register("skybox_bottom"),
			back: assets.register("skybox_back"),
			front: assets.register("skybox_front"),
			..Default::default()
		},
		Pipeline::default(),
	)));
}

fn player_control(mut world: Mut<World>, input: Const<Input>, frame: Const<Frame>, mut camera: Mut<Camera>) {
	let dz = if input.is_action_hold(Action::PanUp) {
		-(PAN_SPEED * frame.delta().as_secs_f32())
	} else if input.is_action_hold(Action::PanDown) {
		PAN_SPEED * frame.delta().as_secs_f32()
	} else {
		0.0
	};

	let dx = if input.is_action_hold(Action::PanRight) {
		PAN_SPEED * frame.delta().as_secs_f32()
	} else if input.is_action_hold(Action::PanLeft) {
		-(PAN_SPEED * frame.delta().as_secs_f32())
	} else {
		0.0
	};

	let dy = if input.mouse_scroll() > 0.0 {
		SCROLL_SPEED * frame.delta().as_secs_f32()
	} else if input.mouse_scroll() < 0.0 {
		-(SCROLL_SPEED * frame.delta().as_secs_f32())
	} else {
		0.0
	};

	let pos_x = camera.target.x - dx;
	let pos_z = camera.target.z - dz;
	let pos_y = camera.target.y - dy;

	camera.target = Point3::new(pos_x, pos_y, pos_z);
}

fn global_control(input: Const<Input>) {
	if input.is_action_activated(Action::Exit) && input.modifiers == dotrix::input::Modifiers::CTRL {
		std::process::exit(0);
	}
}

fn ui_main(mut state: Mut<State>, input: Const<Input>, overlay: Const<Overlay>, frame: Const<Frame>, camera: Const<Camera>) {
	let egui_overlay = overlay
		.get::<Egui>()
		.expect("Egui overlay must be added at startup");

	let main_state = state
		.get::<MainState>()
		.expect("Unable to get main state");

	if input.is_action_activated(Action::TogglePause) {
		state.push(PauseState {
			name: String::from("Paused State"),
			handled: false,
		});

		return;
	}

	egui::Area::new("Information")
		.fixed_pos(egui::pos2(16.0, 16.0))
		.show(&egui_overlay.ctx, |ui| {
			ui.colored_label(DEBUG_YELLOW, "Press ESC to pause and CTRL+C to exit.");
		});

	egui::Area::new("FPS Counter")
		.fixed_pos(egui::pos2(16.0, 32.0))
		.show(&egui_overlay.ctx, |ui| {
			ui.colored_label(DEBUG_YELLOW, format!("FPS: {:.1}", frame.fps()));
		});

	egui::Area::new("Camera")
		.fixed_pos(egui::pos2(16.0, 48.0))
		.show(&egui_overlay.ctx, |ui| {
			ui.colored_label(DEBUG_YELLOW, format!("Camera X,Y,Z: [{:.1},{:.1},{:.1}]", camera.target.x, camera.target.y, camera.target.z));
		});

	let ms = main_state.clone();
	egui::Area::new("Mouse")
		.fixed_pos(egui::pos2(16.0, 64.0))
		.show(&egui_overlay.ctx, |ui| {
			let pos = input.mouse_position().unwrap();
			let ms.positions.filter(|p| p.x );
			ui.colored_label(DEBUG_YELLOW, format!("Mouse X,Y: [{:.1},{:.1}]", pos.x, pos.y));
		});
}

fn ui_paused(mut state: Mut<State>, input: Const<Input>, overlay: Const<Overlay>, mut window: Mut<Window>) {
	window.set_cursor_grab(false);

	let egui_overlay = overlay
		.get::<Egui>()
		.expect("Egui overlay must be added at startup");

	let states_stack_dump = state
		.dump()
		.join(",\n ");

	let mut pause_state = state
		.get_mut::<PauseState>()
		.expect("Cannot find pause state");

	let mut exit_state = pause_state.handled && input.is_action_activated(Action::TogglePause);
	pause_state.handled = true;

	egui::containers::Window::new("Paused")
		.resizable(false)
		.default_width(200.0)
		.show(&egui_overlay.ctx, |ui| {
			ui.label("Execution is paused. Camera is not controllable");
			ui.label(format!("Current states stack: [\n {}\n]", states_stack_dump));
		});

	egui::Area::new("Information")
		.fixed_pos(egui::pos2(16.0, 16.0))
		.show(&egui_overlay.ctx, |ui| {
			ui.colored_label(DEBUG_YELLOW, "Press ESC to resume");
		});

	if exit_state {
		window.set_cursor_grab(true);
		state.pop_any();
	}
}
