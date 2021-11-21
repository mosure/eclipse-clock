use std::rc::Rc;
use chrono::{DateTime, Timelike, NaiveDateTime, Utc};

use js_sys::Date;

use web_sys::{
    WebGl2RenderingContext,
    WebGlProgram,
    WebGlShader,
};

use super::pipeline::{AppContext, Stage};
use super::web_util::{perf_to_system, performance};


pub struct DrawStage {
    gl: Rc<WebGl2RenderingContext>,
    program: Rc<WebGlProgram>,
    vert_count: u32,
    vertices: [f32; 18],
}

pub struct RenderStage {
    gl: Rc<WebGl2RenderingContext>,
    ctx: Rc<AppContext>,
    program: Rc<WebGlProgram>,
    draw_stage: DrawStage,
    pub hour: f32,
    pub minute: f32,
    pub second: f32,
    pub millisecond: u32,
    pub time_offset: f64,
}

impl DrawStage {
    pub fn new(
        gl: Rc<WebGl2RenderingContext>,
        program: Rc<WebGlProgram>,
    ) -> DrawStage {
        let vertices: [f32; 18] = [
            -1.0, -1.0, 0.0,
            -1.0, 1.0, 0.0,
            1.0, 1.0, 0.0,

            1.0, 1.0, 0.0,
            1.0, -1.0, 0.0,
            -1.0, -1.0, 0.0,
        ];

        let mut stage = DrawStage {
            gl: gl,
            program: program,
            vertices: vertices,
            vert_count: (vertices.len() / 3) as u32,
        };
        stage.init();

        return stage;
    }

    fn init(&mut self) -> () {
        let position_attribute_location = self.gl.get_attrib_location(&self.program, "position");
        let buffer = self.gl.create_buffer();
        self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, buffer.as_ref());

        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        unsafe {
            let positions_array_buf_view = js_sys::Float32Array::view(&self.vertices);

            self.gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &positions_array_buf_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        let vao = self.gl.create_vertex_array().expect("should have a vertex array object");
        self.gl.bind_vertex_array(Some(&vao));

        self.gl.vertex_attrib_pointer_with_i32(0, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);
        self.gl.enable_vertex_attrib_array(position_attribute_location as u32);

        self.gl.bind_vertex_array(Some(&vao));
    }
}

impl Stage for DrawStage {
    fn render(&mut self) -> () {
        self.gl.clear_color(1.0, 0.0, 0.0, 1.0);
        self.gl.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT |
            WebGl2RenderingContext::DEPTH_BUFFER_BIT
        );

        self.gl.draw_arrays(
            WebGl2RenderingContext::TRIANGLES,
            0,
            self.vert_count as i32
        );
    }
}

impl RenderStage {
    pub fn new(
        gl: Rc<WebGl2RenderingContext>,
        ctx: Rc<AppContext>,
    ) -> RenderStage {
        let vert_shader = compile_shader(
            &gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            include_str!("shaders/render/vert.glsl"),
        ).expect("expect vertex shader");

        let frag_shader = compile_shader(
            &gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            include_str!("shaders/render/frag.glsl"),
        ).expect("expect frag shader");

        let program = Rc::new(
            link_program(
                &gl,
                &vert_shader,
                &frag_shader
            ).expect("expect linked program")
        );

        let draw_stage = DrawStage::new(
            Rc::clone(&gl),
            Rc::clone(&program),
        );

        let mut stage = RenderStage {
            gl: gl,
            ctx: ctx,
            program: program,
            draw_stage: draw_stage,
            hour: 0.0,
            minute: 0.0,
            second: 0.0,
            millisecond: 0,
            time_offset: Date::now(),
        };
        stage.init();

        return stage;
    }

    fn init(&mut self) {
        self.gl.use_program(Some(&self.program));

        self.gl.viewport(0, 0, self.ctx.width as i32, self.ctx.height as i32);
    }
}

impl Stage for RenderStage {
    fn render(&mut self) {
        let start = perf_to_system(performance().now());
        let since_the_boot = start
            .duration_since(self.ctx.boot_time)
            .expect("Time went backwards");

        self.millisecond = since_the_boot.as_millis() as u32 % 1000;

        let central_offset = 6;
        let dt = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(((self.time_offset / 1000.0) as u64 + since_the_boot.as_secs()) as i64, 0), Utc);

        self.hour = dt.hour() as f32 + central_offset as f32 + dt.minute() as f32 / 60.0 + dt.second() as f32 / 60.0 / 60.0 + self.millisecond as f32 / 60.0 / 60.0 / 1000.0;
        self.minute = dt.minute() as f32 + dt.second() as f32 / 60.0 + self.millisecond as f32 / 60.0 / 1000.0;
        self.second = dt.second() as f32 + self.millisecond as f32 / 1000.0;

        self.gl.use_program(Some(&self.program));

        self.gl.active_texture(
            WebGl2RenderingContext::TEXTURE0,
        );

        let u_hour = self.gl.get_uniform_location(&self.program, "u_hour");
        let u_minute = self.gl.get_uniform_location(&self.program, "u_minute");
        let u_second = self.gl.get_uniform_location(&self.program, "u_second");
        let u_millisecond = self.gl.get_uniform_location(&self.program, "u_millisecond");

        let u_resolution = self.gl.get_uniform_location(&self.program, "u_resolution");
        let u_time = self.gl.get_uniform_location(&self.program, "u_time");

        self.gl.uniform1f(u_hour.as_ref(), self.hour as f32);
        self.gl.uniform1f(u_minute.as_ref(), self.minute as f32);
        self.gl.uniform1f(u_second.as_ref(), self.second as f32);
        self.gl.uniform1i(u_millisecond.as_ref(), self.millisecond as i32);

        let mut u_res_val = [self.ctx.width as f32, self.ctx.height as f32];
        self.gl.uniform2fv_with_f32_array(u_resolution.as_ref(), &mut u_res_val);

        self.gl.uniform1f(u_time.as_ref(), since_the_boot.as_secs_f32());

        self.draw_stage.render();

        self.gl.flush();
    }
}



// ---------- Util functions -------------


// Shared stage functions
pub fn compile_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    gl: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
