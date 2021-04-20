extern crate nalgebra_glm as glm;

use glutin::dpi::LogicalSize;
use glutin::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use glutin::{GlProfile, GlRequest};

use gl;
use gl::types::*;

use std::ffi::CString;
use std::fs;
use std::time::Instant;

use stb_image::image::{self, LoadResult};

#[derive(Default)]
struct Input {
    forward: bool,
    back: bool,
    left: bool,
    right: bool,
}

fn main() {
    let el = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("Boulder Dash")
        .with_inner_size(LogicalSize::new(1000.0, 1000.0));
    let gl_request = GlRequest::Latest;
    let gl_profile = GlProfile::Core;
    let windowed_context = ContextBuilder::new()
        .with_gl(gl_request)
        .with_gl_profile(gl_profile)
        .with_double_buffer(Some(true))
        .with_vsync(true)
        .build_windowed(wb, &el)
        .unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    gl::load_with(|s| windowed_context.get_proc_address(s) as *const _);

    let window_size = windowed_context.window().inner_size();
    unsafe {
        gl::Viewport(0, 0, window_size.width as i32, window_size.height as i32);
        gl::ClearColor(0.05, 0.05, 0.05, 1.0);
    }

    // Load image
    // unsafe {
    //     stb_image::stb_image::bindgen::stbi_set_flip_vertically_on_load(1);
    // }

    let img = match image::load_with_depth("./bd-sprites.png", 3, false) {
        LoadResult::ImageU8(image) => image,
        _ => panic!("Couldnt load image"),
    };

    // Quad
    let right = 32.0 / img.width as f32;
    let bottom = 32.0 / img.height as f32;

    println!("right {}, bottom {}", right, bottom);

    #[rustfmt::skip]
    let vertices: Vec<f32> = vec![
        0.0, 0.5, 0.0,  
        0.0, -0.25, 0.0, 
        -0.5, -0.5, 0.0,

        0.0, 0.5, 0.0,  
        0.0, -0.25, 0.0, 
        0.5, -0.5, 0.0,
    ];

    let mut buffer_id: GLuint = 0;
    unsafe {
        gl::GenBuffers(1, &mut buffer_id);
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer_id);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * 4) as isize,
            vertices.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );
    }

    let mut vao_id: GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao_id);
        gl::BindVertexArray(vao_id);

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * 4, 0 as *const GLvoid);
        gl::EnableVertexAttribArray(0);
    }

    // Vertex shader
    let source = fs::read_to_string("shaders/shader.vert").unwrap();
    let source = CString::new(source).unwrap();
    let vert_shader_id = unsafe { gl::CreateShader(gl::VERTEX_SHADER) };
    unsafe {
        gl::ShaderSource(vert_shader_id, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(vert_shader_id);
    }

    let mut success: GLint = 1;
    unsafe {
        gl::GetShaderiv(vert_shader_id, gl::COMPILE_STATUS, &mut success);
    }
    if success == 0 {
        let mut len: GLint = 0;
        unsafe {
            gl::GetShaderiv(vert_shader_id, gl::INFO_LOG_LENGTH, &mut len);
        }
        let error = new_cstring(len as usize);
        unsafe {
            gl::GetShaderInfoLog(
                vert_shader_id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut GLchar,
            );
        }
        println!("{}", error.to_string_lossy().into_owned());
    }

    // Fragment shader
    let source = fs::read_to_string("shaders/shader.frag").unwrap();
    let source = CString::new(source).unwrap();
    let frag_shader_id = unsafe { gl::CreateShader(gl::FRAGMENT_SHADER) };
    unsafe {
        gl::ShaderSource(frag_shader_id, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(frag_shader_id);
    }

    let mut success: GLint = 1;
    unsafe {
        gl::GetShaderiv(frag_shader_id, gl::COMPILE_STATUS, &mut success);
    }
    if success == 0 {
        let mut len: GLint = 0;
        unsafe {
            gl::GetShaderiv(frag_shader_id, gl::INFO_LOG_LENGTH, &mut len);
        }
        let error = new_cstring(len as usize);
        unsafe {
            gl::GetShaderInfoLog(
                frag_shader_id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut GLchar,
            );
        }
        println!("{}", error.to_string_lossy().into_owned());
    }

    // Shader program
    let program_id = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(program_id, vert_shader_id);
        gl::AttachShader(program_id, frag_shader_id);
        gl::LinkProgram(program_id);

        let mut success: GLint = 1;
        gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut len: GLint = 0;
            gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            let error = new_cstring(len as usize);
            gl::GetProgramInfoLog(
                program_id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut GLchar,
            );
            println!("ERROR: {}", error.to_string_lossy().into_owned());
        }

        gl::UseProgram(program_id);
    }

    let angle_uniform_location = {
        let name = CString::new("angle").unwrap();
        unsafe { gl::GetUniformLocation(program_id, name.as_ptr() as *const GLchar) }
    };

    let start_time = Instant::now();

    let model = glm::scaling(&glm::vec3(0.1, 0.1, 0.1));
    let view = {
        let eye = glm::vec3(0.0, 0.0, 1.0);
        let center = glm::vec3(0.0, 0.0, 0.0);
        let up = glm::vec3(0.0, 1.0, 0.0);

        glm::look_at(&eye, &center, &up)
    };
    let projection = {
        let aspect = 1.0;
        let fovy = glm::pi::<f32>() / 2.0;

        glm::perspective(aspect, fovy, 0.1, 100.0)
    };

    let model_uniform_location = {
        let name = CString::new("Model").unwrap();
        unsafe { gl::GetUniformLocation(program_id, name.as_ptr() as *const GLchar) }
    };
    let view_uniform_location = {
        let name = CString::new("View").unwrap();
        unsafe { gl::GetUniformLocation(program_id, name.as_ptr() as *const GLchar) }
    };
    let projection_uniform_location = {
        let name = CString::new("Projection").unwrap();
        unsafe { gl::GetUniformLocation(program_id, name.as_ptr() as *const GLchar) }
    };

    let mut input = Input::default();

    let mut angle_dir = glm::pi::<f32>();
    let mut frame_start = Instant::now();

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } => {
                    // Key press
                    match key {
                        VirtualKeyCode::Up => input.forward = state == ElementState::Pressed,
                        VirtualKeyCode::Left => input.left = state == ElementState::Pressed,
                        VirtualKeyCode::Right => input.right = state == ElementState::Pressed,
                        VirtualKeyCode::Down => input.back = state == ElementState::Pressed,
                        VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                let delta_time = frame_start.elapsed().as_secs_f32();
                frame_start = Instant::now();
                if input.right {
                    angle_dir += (glm::pi::<f32>()) * delta_time;
                }
                if input.left {
                    angle_dir -= (glm::pi::<f32>()) * delta_time;
                }

                // let angle_y = start_time.elapsed().as_secs_f32() * 3.0;
                // let model = glm::rotate(&model, angle_y, &glm::vec3(0.0, 1.0, 0.0));
                let model = glm::rotate(&model, angle_dir, &glm::vec3(0.0, 0.0, -1.0));

                unsafe {
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                    gl::UniformMatrix4fv(model_uniform_location, 1, gl::FALSE, model.as_ptr());
                    gl::UniformMatrix4fv(view_uniform_location, 1, gl::FALSE, view.as_ptr());
                    gl::UniformMatrix4fv(
                        projection_uniform_location,
                        1,
                        gl::FALSE,
                        projection.as_ptr(),
                    );
                    gl::DrawArrays(gl::TRIANGLES, 0, 6);
                }

                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    })
}

fn new_cstring(len: usize) -> CString {
    let buffer: Vec<u8> = vec![0; len];
    unsafe { CString::from_vec_unchecked(buffer) }
}
