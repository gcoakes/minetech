use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as GL, WebGlProgram};
use yew::{
    prelude::*,
    services::{RenderService, Task},
};

struct Model {
    canvas: Option<HtmlCanvasElement>,
    gl: Option<GL>,
    node_ref: NodeRef,
    render_loop: Option<Box<dyn Task>>,
    shader: Option<WebGlProgram>,
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
            shader: None,
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
        // Cast the node ref into the canvas element.
        let canvas: HtmlCanvasElement = self.node_ref.cast().unwrap();

        // Establish a new WebGL2RenderingContext. This is done on every render
        // because there are two different layers of rendering occurring. One
        // occurs when the HTML repaints this element. This rarely occurs. The
        // other occurs within the canvas for each webgl frame.
        //
        // Whenever the HTML repaints, we want to re-establish the context.
        let gl: GL = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        // Save the canvas for later.
        self.canvas = Some(canvas);

        // Include the shader source code at compile time.
        let vert_shader_src = include_str!("./basic.vert");
        let frag_shader_src = include_str!("./basic.frag");

        let vert_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
        gl.shader_source(&vert_shader, &vert_shader_src);
        gl.compile_shader(&vert_shader);

        let frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
        gl.shader_source(&frag_shader, &frag_shader_src);
        gl.compile_shader(&frag_shader);

        let shader_program = gl.create_program().unwrap();
        gl.attach_shader(&shader_program, &vert_shader);
        gl.attach_shader(&shader_program, &frag_shader);
        gl.link_program(&shader_program);

        gl.use_program(Some(&shader_program));

        self.shader = Some(shader_program);

        // Save the webgl context for later.
        self.gl = Some(gl);

        // Create a render loop on the first render.
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
        // It can be assumed that the webgl context has already been initialized
        // because this function gets called through the render loop. The render
        // loop is only established after the first HTML render.
        let gl = self.gl.as_ref().expect("GL context initialized");

        let vertices: Vec<f32> = vec![
            -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0,
        ];
        let vert_buf = gl.create_buffer().unwrap();
        let verts = js_sys::Float32Array::from(vertices.as_slice());

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vert_buf));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &verts, GL::STATIC_DRAW);

        let pos = gl.get_attrib_location(&self.shader.as_ref().unwrap(), "a_position") as u32;
        gl.vertex_attrib_pointer_with_i32(pos, 2, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(pos);

        let gl_time = gl.get_uniform_location(&self.shader.as_ref().unwrap(), "u_time");
        gl.uniform1f(gl_time.as_ref(), time as f32);

        gl.draw_arrays(GL::TRIANGLES, 0, 6);

        let render_frame = self.link.callback(Msg::Render);
        let handle = RenderService::request_animation_frame(render_frame);

        self.render_loop = Some(Box::new(handle));
    }
}

fn main() {
    yew::start_app::<Model>();
}
