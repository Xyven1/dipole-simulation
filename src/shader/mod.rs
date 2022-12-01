use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::*;

static LINE_VS: &str = include_str!("./flat-vertex.glsl");
static LINE_FS: &str = include_str!("./flat-fragment.glsl");

static MESH_VS: &str = include_str!("./mesh-vertex.glsl");
static MESH_FS: &str = include_str!("./mesh-fragment.glsl");

/// Powers retrieving and using our shaders
pub struct ShaderSystem {
    programs: HashMap<ShaderKind, Shader>,
    active_program: RefCell<ShaderKind>,
}

impl ShaderSystem {
    /// Create  a new ShaderSystem
    pub fn new(gl: &WebGlRenderingContext) -> ShaderSystem {
        let mut programs = HashMap::new();

        let mesh_shader = Shader::new(gl, MESH_VS, MESH_FS).unwrap();
        let line_shader = Shader::new(gl, LINE_VS, LINE_FS).unwrap();

        let active_program = RefCell::new(ShaderKind::Mesh);
        gl.use_program(Some(&mesh_shader.program));

        programs.insert(ShaderKind::Mesh, mesh_shader);
        programs.insert(ShaderKind::Flat, line_shader);

        ShaderSystem {
            programs,
            active_program,
        }
    }

    /// Get one of our Shader's
    pub fn get_shader(&self, shader_kind: &ShaderKind) -> Option<&Shader> {
        self.programs.get(shader_kind)
    }

    /// Use a shader program. We cache the last used shader program to avoid unnecessary
    /// calls to the GPU.
    pub fn use_program(&self, gl: &WebGlRenderingContext, shader_kind: ShaderKind) {
        if *self.active_program.borrow() == shader_kind {
            return;
        }

        gl.use_program(Some(&self.programs.get(&shader_kind).unwrap().program));
        *self.active_program.borrow_mut() = shader_kind;
    }
}

/// Identifiers for our different shaders
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum ShaderKind {
    Mesh,
    Flat,
    Point,
}

/// One per ShaderKind
pub struct Shader {
    pub program: WebGlProgram,
    uniforms: RefCell<HashMap<String, WebGlUniformLocation>>,
}

impl Shader {
    /// Create a new Shader program from a vertex and fragment shader
    fn new(
        gl: &WebGlRenderingContext,
        vert_shader: &str,
        frag_shader: &str,
    ) -> Result<Shader, JsValue> {
        let vert_shader = compile_shader(gl, WebGlRenderingContext::VERTEX_SHADER, vert_shader)?;
        let frag_shader = compile_shader(gl, WebGlRenderingContext::FRAGMENT_SHADER, frag_shader)?;
        let program = link_program(gl, &vert_shader, &frag_shader)?;

        let uniforms = RefCell::new(HashMap::new());

        Ok(Shader { program, uniforms })
    }

    /// Get the location of a uniform.
    /// If this is our first time retrieving it we will cache it so that for future retrievals
    /// we won't need to query the shader program.
    pub fn get_uniform_location(
        &self,
        gl: &WebGlRenderingContext,
        uniform_name: &str,
    ) -> Option<WebGlUniformLocation> {
        let mut uniforms = self.uniforms.borrow_mut();

        if uniforms.get(uniform_name).is_none() {
            uniforms.insert(
                uniform_name.to_string(),
                gl.get_uniform_location(&self.program, uniform_name)
                    .unwrap_or_else(|| panic!(r#"Uniform '{}' not found"#, uniform_name)),
            );
        }

        Some(uniforms.get(uniform_name).expect("loc").clone())
    }
}

/// Create a shader program using the WebGL APIs
fn compile_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| "Could not create shader".to_string())?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "Unknown error creating shader".to_string()))
    }
}

/// Link a shader program using the WebGL APIs
fn link_program(
    gl: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| "Unable to create shader program".to_string())?;

    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);

    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown error creating program".to_string()))
    }
}
