use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlRenderingContext};
use yew::{
    prelude::*,
    services::{RenderService, Task},
};

struct Model {
    canvas: Option<HtmlCanvasElement>,
    gl: Option<WebGlRenderingContext>,
    node_ref: NodeRef,
    render_loop: Option<Box<dyn Task>>,
    link: ComponentLink<Self>,
}

enum Msg {
    Render(f64),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            canvas: None,
            gl: None,
            node_ref: NodeRef::default(),
            render_loop: None,
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Render(time) => {
                self.render_gl(time);
            }
        }
        false
    }

    fn rendered(&mut self, first_render: bool) {
        let canvas: HtmlCanvasElement = self.node_ref.cast().unwrap();
        let gl: WebGlRenderingContext = canvas
            .get_context("webgl")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        self.canvas = Some(canvas);
        self.gl = Some(gl);
        if first_render {
            let render_frame = self.link.callback(Msg::Render);
            let handle = RenderService::request_animation_frame(render_frame);
            self.render_loop = Some(Box::new(handle));
        }
    }

    fn view(&self) -> Html {
        html! {
            <canvas ref={self.node_ref.clone()} />
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }
}

impl Model {
    fn render_gl(&mut self, time: f64) {
        let gl = self.gl.as_ref().expect("GL context initialized");

        let vert_shader_src = include_str!("./basic.vert");
        let frag_shader_src = include_str!("./basic.frag");

        let vertices: Vec<f32> = vec![
            -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0,
        ];
        let vert_buf = gl.create_buffer().unwrap();
        let verts = js_sys::Float32Array::from(vertices.as_slice());

        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vert_buf));
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &verts,
            WebGlRenderingContext::STATIC_DRAW,
        );

        let vert_shader = gl
            .create_shader(WebGlRenderingContext::VERTEX_SHADER)
            .unwrap();
        gl.shader_source(&vert_shader, &vert_shader_src);
        gl.compile_shader(&vert_shader);

        let frag_shader = gl
            .create_shader(WebGlRenderingContext::FRAGMENT_SHADER)
            .unwrap();
        gl.shader_source(&frag_shader, &frag_shader_src);
        gl.compile_shader(&frag_shader);

        let shader_program = gl.create_program().unwrap();
        gl.attach_shader(&shader_program, &vert_shader);
        gl.attach_shader(&shader_program, &frag_shader);
        gl.link_program(&shader_program);

        gl.use_program(Some(&shader_program));

        let pos = gl.get_attrib_location(&shader_program, "a_position") as u32;
        gl.vertex_attrib_pointer_with_i32(pos, 2, WebGlRenderingContext::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(pos);

        let gl_time = gl.get_uniform_location(&shader_program, "u_time");
        gl.uniform1f(gl_time.as_ref(), time as f32);

        gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 6);

        let render_frame = self.link.callback(Msg::Render);
        let handle = RenderService::request_animation_frame(render_frame);

        self.render_loop = Some(Box::new(handle));
    }
}

fn main() {
    yew::start_app::<Model>();
}
