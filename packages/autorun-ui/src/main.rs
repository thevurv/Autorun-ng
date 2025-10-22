use glam::{Mat4, Vec2, Vec4};
use miniquad::*;
use taffy::prelude::*;

#[repr(C)]
struct Vertex {
	pos: Vec2,
	uv: Vec2,
	color: Vec4,
}

struct UINode {
	background_color: Vec4,
}

type UITree = TaffyTree<UINode>;

struct Stage {
	ctx: Box<dyn RenderingBackend>,
	pipeline: Pipeline,
	bindings: Bindings,
	taffy_tree: UITree,
	root_node: NodeId,
}

impl Stage {
	pub fn new() -> Result<Stage, Box<dyn std::error::Error>> {
		let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

		// Create Taffy tree
		let mut taffy_tree = TaffyTree::new();
		let left_side = taffy_tree
			.new_leaf_with_context(
				Style {
					display: Display::Flex,
					flex_direction: FlexDirection::Column,
					flex_grow: 1.0,
					..Default::default()
				},
				UINode {
					background_color: Vec4::new(0.2, 0.2, 0.8, 1.0),
				},
			)
			.unwrap();

		let right_side = taffy_tree
			.new_leaf_with_context(
				Style {
					display: Display::Flex,
					flex_direction: FlexDirection::Column,
					flex_grow: 1.0,
					..Default::default()
				},
				UINode {
					background_color: Vec4::new(0.8, 0.2, 0.2, 1.0),
				},
			)
			.unwrap();

		let root_node = taffy_tree
			.new_with_children(
				Style {
					display: Display::Flex,
					flex_direction: FlexDirection::Row,
					size: Size {
						width: length(800.0),
						height: length(600.0),
					},
					..Default::default()
				},
				&[left_side, right_side],
			)
			.unwrap();

		// Compute layout
		taffy_tree
			.compute_layout(
				root_node,
				Size {
					width: AvailableSpace::Definite(800.0),
					height: AvailableSpace::Definite(600.0),
				},
			)
			.unwrap();

		// Build initial vertex/index buffers from layout
		let (vertices, indices) = Self::build_ui_geometry(&taffy_tree, root_node)?;

		let vertex_buffer = ctx.new_buffer(
			BufferType::VertexBuffer,
			BufferUsage::Stream, // Use Stream for dynamic updates
			BufferSource::slice(&vertices),
		);

		let index_buffer = ctx.new_buffer(BufferType::IndexBuffer, BufferUsage::Stream, BufferSource::slice(&indices));

		let bindings = Bindings {
			vertex_buffers: vec![vertex_buffer],
			index_buffer: index_buffer,
			images: vec![],
		};

		let shader = ctx
			.new_shader(
				ShaderSource::Glsl {
					vertex: include_str!("shaders/vertex.glsl"),
					fragment: include_str!("shaders/fragment.glsl"),
				},
				shader::meta(),
			)
			.unwrap();

		let pipeline = ctx.new_pipeline(
			&[BufferLayout::default()],
			&[
				VertexAttribute::new("inPos", VertexFormat::Float2),
				VertexAttribute::new("inUV", VertexFormat::Float2),
				VertexAttribute::new("inColor", VertexFormat::Float4),
			],
			shader,
			PipelineParams::default(),
		);

		Ok(Stage {
			pipeline,
			bindings,
			ctx,
			taffy_tree,
			root_node,
		})
	}

	// Convert Taffy layout tree to vertices/indices
	fn build_ui_geometry(tree: &UITree, node: NodeId) -> taffy::TaffyResult<(Vec<Vertex>, Vec<u16>)> {
		let mut vertices = Vec::new();
		let mut indices = Vec::new();

		Self::build_node_geometry(tree, node, &mut vertices, &mut indices)?;

		Ok((vertices, indices))
	}

	fn build_node_geometry(
		tree: &UITree,
		node: NodeId,
		vertices: &mut Vec<Vertex>,
		indices: &mut Vec<u16>,
	) -> taffy::TaffyResult<()> {
		// If no context, it's just a layout node, skip.
		let Some(context) = tree.get_node_context(node) else {
			for child in tree.children(node).unwrap() {
				Self::build_node_geometry(tree, child, vertices, indices)?;
			}

			return Ok(());
		};

		let layout = tree.layout(node).unwrap();
		let base_index = vertices.len() as u16;

		// Create a quad for this node
		let x = layout.location.x;
		let y = layout.location.y;
		let w = layout.size.width;
		let h = layout.size.height;

		// Add 4 vertices for the quad
		vertices.push(Vertex {
			pos: Vec2::new(x, y),
			uv: Vec2::new(0.0, 0.0),
			color: context.background_color,
		});
		vertices.push(Vertex {
			pos: Vec2::new(x + w, y),
			uv: Vec2::new(1.0, 0.0),
			color: context.background_color,
		});
		vertices.push(Vertex {
			pos: Vec2::new(x + w, y + h),
			uv: Vec2::new(1.0, 1.0),
			color: context.background_color,
		});
		vertices.push(Vertex {
			pos: Vec2::new(x, y + h),
			uv: Vec2::new(0.0, 1.0),
			color: context.background_color,
		});

		// Add 6 indices for 2 triangles
		indices.extend_from_slice(&[
			base_index,
			base_index + 1,
			base_index + 2,
			base_index + 2,
			base_index + 3,
			base_index,
		]);

		// Recursively process children
		for child in tree.children(node).unwrap() {
			Self::build_node_geometry(tree, child, vertices, indices)?;
		}

		Ok(())
	}
}

impl EventHandler for Stage {
	fn update(&mut self) {}

	fn draw(&mut self) {
		self.ctx.begin_default_pass(PassAction::clear_color(0.1, 0.1, 0.1, 1.0));
		self.ctx.apply_pipeline(&self.pipeline);
		self.ctx.apply_bindings(&self.bindings);

		self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms {
			proj_matrix: Mat4::orthographic_rh_gl(0.0, 800.0, 600.0, 0.0, -1.0, 1.0),
		}));

		// Draw all the geometry in one call
		let num_indices = 12; // 2 quads * 6 indices each
		self.ctx.draw(0, num_indices, 1);

		self.ctx.end_render_pass();
		self.ctx.commit_frame();
	}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	miniquad::start(conf::Conf::default(), || Box::new(Stage::new().unwrap()));
	Ok(())
}

mod shader {
	use miniquad::*;
	pub fn meta() -> ShaderMeta {
		ShaderMeta {
			images: vec![],
			uniforms: UniformBlockLayout {
				uniforms: vec![UniformDesc::new("projMatrix", UniformType::Mat4)],
			},
		}
	}
	#[repr(C)]
	pub struct Uniforms {
		pub proj_matrix: glam::Mat4,
	}
}
