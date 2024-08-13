// use gl::types::*;
// use std::ffi::{CString, c_void, CStr};

// fn create_shader(shader_type: u32, source: &str) -> u32 {
//     unsafe {
//         let id = gl::CreateShader(shader_type);
//         let source_cstr = CString::new(source).unwrap();
//         gl::ShaderSource(
//             id,
//             1,
//             &source_cstr.as_ptr(),
//             std::ptr::null()
//         );
//         gl::CompileShader(id);
//         let mut success: gl::types::GLint = 1;
//         gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
//         if success == 0 {
//             let mut len: gl::types::GLint = 0;
//             gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
//             let error = {
//                 let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
//                 buffer.extend([b' '].iter().cycle().take(len as usize));
//                 CString::from_vec_unchecked(buffer)
//             };
//             gl::GetShaderInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut gl::types::GLchar);
//             eprintln!("{}", error.to_string_lossy());
//         }
//         id
//     }
// }

// pub fn create_program(
//     vertex_shader: &str,
//     fragment_shader: &str,
// ) -> u32 {
//     let vs = create_shader(gl::VERTEX_SHADER, vertex_shader);
//     let fs = create_shader(gl::FRAGMENT_SHADER, fragment_shader);
    
//     unsafe {
//         let program = gl::CreateProgram();
//         gl::AttachShader(program, vs);
//         gl::AttachShader(program, fs);
//         gl::LinkProgram(program);
//         let mut success: gl::types::GLint = 1;
//         gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
//         if success == 0 {
//             let mut len: gl::types::GLint = 0;
//             gl::GetShaderiv(program, gl::INFO_LOG_LENGTH, &mut len);
//             let error = {
//                 let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
//                 buffer.extend([b' '].iter().cycle().take(len as usize));
//                 CString::from_vec_unchecked(buffer)
//             };
//             gl::GetProgramInfoLog(program, len, std::ptr::null_mut(), error.as_ptr() as *mut gl::types::GLchar);
//             eprintln!("{}", error.to_string_lossy());
//         }
//         gl::DeleteShader(vs);
//         gl::DeleteShader(fs);
//         program
//     }

// }

// pub extern "system" fn debug_callback(
//     _source: GLenum,
//     _type: GLenum,
//     _id: GLenum,
//     severity: GLenum,
//     _length: GLsizei,
//     message: *const GLchar,
//     _user_param: *mut c_void,
// ) {
//     let msg = unsafe {CStr::from_ptr(message).to_str().unwrap()};
//     if severity != gl::DEBUG_SEVERITY_NOTIFICATION {
//         let severity_str = match severity {
//             gl::DEBUG_SEVERITY_HIGH => "high",
//             gl::DEBUG_SEVERITY_MEDIUM => "medium",
//             gl::DEBUG_SEVERITY_LOW => "low",
//             gl::DEBUG_SEVERITY_NOTIFICATION => "notification",
//             _ => "???"
//         };
//         println!("DEBUG MESSAGE: [{severity_str}]{}", msg);
//     }
// }
