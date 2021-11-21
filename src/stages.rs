use std::rc::Rc;

use web_sys::{
    WebGl2RenderingContext,
    WebGlFramebuffer,
    WebGlProgram,
    WebGlShader,
    WebGlTexture,
};

use super::pipeline::{AppContext, Stage};
use super::web_util::{perf_to_system, performance};


pub struct DrawStage {
    gl: Rc<WebGl2RenderingContext>,
    program: Rc<WebGlProgram>,
    vert_count: u32,
    vertices: [f32; 18],
}

pub struct ComputeStage {
    gl: Rc<WebGl2RenderingContext>,
    ctx: Rc<AppContext>,
    program: Rc<WebGlProgram>,
    framebuffer: WebGlFramebuffer,
    transfer_buffer: WebGlFramebuffer,
    pub in_state: WebGlTexture,
    pub out_state: WebGlTexture,
    draw_stage: DrawStage,
    pub frame: u32,
}

pub struct RenderStage {
    gl: Rc<WebGl2RenderingContext>,
    ctx: Rc<AppContext>,
    program: Rc<WebGlProgram>,
    compute_stage: ComputeStage,
    draw_stage: DrawStage,
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

impl ComputeStage {
    pub fn new(
        gl: Rc<WebGl2RenderingContext>,
        ctx: Rc<AppContext>,
    ) -> ComputeStage {
        let vert_shader = compile_shader(
            &gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            include_str!("shaders/state/vert.glsl"),
        ).expect("expect vertex shader");

        let frag_shader = compile_shader(
            &gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            include_str!("shaders/state/frag.glsl"),
        ).expect("expect frag shader");

        let program = Rc::new(
            link_program(
                &gl,
                &vert_shader,
                &frag_shader
            ).expect("expect linked program")
        );

        let framebuffer = gl.create_framebuffer().expect("should have a framebuffer");
        let transfer_buffer = gl.create_framebuffer().expect("should have a framebuffer");
        let in_state = gl.create_texture().expect("should have a texture");
        let out_state = gl.create_texture().expect("should have a texture");

        let draw_stage = DrawStage::new(
            Rc::clone(&gl),
            Rc::clone(&program),
        );

        let mut stage = ComputeStage {
            gl: gl,
            ctx: ctx,
            program: program,
            framebuffer: framebuffer,
            transfer_buffer: transfer_buffer,
            in_state: in_state,
            out_state: out_state,
            draw_stage: draw_stage,
            frame: 0,
        };
        stage.init();

        return stage;
    }

    fn init(&mut self) {
        self.gl.use_program(Some(&self.program));

        self.gl.viewport(0, 0, self.ctx.width as i32, self.ctx.height as i32);

        {
            self.gl.bind_framebuffer(
                WebGl2RenderingContext::FRAMEBUFFER,
                Some(&self.transfer_buffer)
            );

            self.gl.active_texture(
                WebGl2RenderingContext::TEXTURE0,
            );

            self.gl.bind_texture(
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&self.in_state)
            );

            self.gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                WebGl2RenderingContext::NEAREST as i32
            );
            self.gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::NEAREST as i32
            );

            self.gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::RGBA as i32,
                self.ctx.width as i32,
                self.ctx.height as i32,
                0,
                WebGl2RenderingContext::RGBA as u32,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                None,
            ).expect("expect tex image 2d result");

            self.gl.framebuffer_texture_2d(
                WebGl2RenderingContext::FRAMEBUFFER,
                WebGl2RenderingContext::COLOR_ATTACHMENT0,
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&self.in_state),
                0,
            );
        }

        {
            self.gl.bind_framebuffer(
                WebGl2RenderingContext::FRAMEBUFFER,
                Some(&self.framebuffer)
            );

            self.gl.active_texture(
                WebGl2RenderingContext::TEXTURE1,
            );

            self.gl.bind_texture(
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&self.out_state)
            );

            self.gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                WebGl2RenderingContext::NEAREST as i32
            );
            self.gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::NEAREST as i32
            );

            self.gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::RGBA as i32,
                self.ctx.width as i32,
                self.ctx.height as i32,
                0,
                WebGl2RenderingContext::RGBA as u32,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                None,
            ).expect("expect tex image 2d result");

            self.gl.framebuffer_texture_2d(
                WebGl2RenderingContext::FRAMEBUFFER,
                WebGl2RenderingContext::COLOR_ATTACHMENT0,
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&self.out_state),
                0,
            );
        }

        self.gl.bind_framebuffer(
            WebGl2RenderingContext::FRAMEBUFFER,
            None
        );
    }

    pub fn transfer_texture(&mut self) {
        self.gl.finish();

        self.gl.bind_framebuffer(
            WebGl2RenderingContext::READ_FRAMEBUFFER,
            Some(&self.framebuffer)
        );

        self.gl.bind_framebuffer(
            WebGl2RenderingContext::DRAW_FRAMEBUFFER,
            Some(&self.transfer_buffer)
        );

        self.gl.blit_framebuffer(
            0,
            0,
            self.ctx.width as i32,
            self.ctx.height as i32,
            0,
            0,
            self.ctx.width as i32,
            self.ctx.height as i32,
            WebGl2RenderingContext::COLOR_BUFFER_BIT,
            WebGl2RenderingContext::NEAREST,
        );

        self.gl.bind_framebuffer(
            WebGl2RenderingContext::READ_FRAMEBUFFER,
            None
        );

        self.gl.bind_framebuffer(
            WebGl2RenderingContext::DRAW_FRAMEBUFFER,
            None
        );
    }
}

impl Stage for ComputeStage {
    fn render(&mut self) {
        self.gl.use_program(Some(&self.program));

        self.gl.active_texture(
            WebGl2RenderingContext::TEXTURE0,
        );

        self.gl.bind_framebuffer(
            WebGl2RenderingContext::FRAMEBUFFER,
            Some(&self.framebuffer)
        );

        let u_frame = self.gl.get_uniform_location(&self.program, "u_frame");
        let u_resolution = self.gl.get_uniform_location(&self.program, "u_resolution");
        let u_time = self.gl.get_uniform_location(&self.program, "u_time");

        self.gl.uniform1i(u_frame.as_ref(), self.frame as i32);

        let mut u_res_val = [self.ctx.width as f32, self.ctx.height as f32];
        self.gl.uniform2fv_with_f32_array(u_resolution.as_ref(), &mut u_res_val);

        let start = perf_to_system(performance().now());
        let since_the_epoch = start
            .duration_since(self.ctx.boot_time)
            .expect("Time went backwards");
        self.gl.uniform1f(u_time.as_ref(), since_the_epoch.as_secs_f32());

        self.draw_stage.render();

        self.gl.bind_framebuffer(
            WebGl2RenderingContext::FRAMEBUFFER,
            None
        );

        self.transfer_texture();

        self.frame += 1;
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

        let compute_stage = ComputeStage::new(
            Rc::clone(&gl),
            Rc::clone(&ctx),
        );

        let draw_stage = DrawStage::new(
            Rc::clone(&gl),
            Rc::clone(&program),
        );

        let mut stage = RenderStage {
            gl: gl,
            ctx: ctx,
            program: program,
            compute_stage: compute_stage,
            draw_stage: draw_stage,
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
        self.compute_stage.render();

        self.gl.use_program(Some(&self.program));

        self.gl.active_texture(
            WebGl2RenderingContext::TEXTURE0,
        );

        let u_resolution = self.gl.get_uniform_location(&self.program, "u_resolution");
        let u_time = self.gl.get_uniform_location(&self.program, "u_time");

        let mut u_res_val = [self.ctx.width as f32, self.ctx.height as f32];
        self.gl.uniform2fv_with_f32_array(u_resolution.as_ref(), &mut u_res_val);

        let start = perf_to_system(performance().now());
        let since_the_epoch = start
            .duration_since(self.ctx.boot_time)
            .expect("Time went backwards");

        self.gl.uniform1f(u_time.as_ref(), since_the_epoch.as_secs_f32());

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
