use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use uve_core::Project;

#[no_mangle]
pub extern "C" fn uve_project_create(
    name: *const c_char,
    width: u32,
    height: u32,
    frame_rate: f32,
) -> *mut c_char {
    let name = match read_string(name) {
        Some(name) => name,
        None => return std::ptr::null_mut(),
    };
    let project = Project::new(name, width, height, frame_rate);
    serialize_project(&project)
}

#[no_mangle]
pub extern "C" fn uve_project_validate_json(json: *const c_char) -> bool {
    let Some(json) = read_string(json) else {
        return false;
    };
    Project::from_json(&json).is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn uve_string_free(value: *mut c_char) {
    if !value.is_null() {
        drop(CString::from_raw(value));
    }
}

fn read_string(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(value).to_str().ok().map(str::to_owned) }
}

fn serialize_project(project: &Project) -> *mut c_char {
    match project
        .to_json()
        .ok()
        .and_then(|json| CString::new(json).ok())
    {
        Some(json) => json.into_raw(),
        None => std::ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn creates_and_validates_project_json() {
        let name = CString::new("Android project").unwrap();
        let json = uve_project_create(name.as_ptr(), 1280, 720, 30.0);
        assert!(!json.is_null());
        assert!(uve_project_validate_json(json));
        unsafe { uve_string_free(json) };
    }

    #[test]
    fn rejects_null_input() {
        assert!(!uve_project_validate_json(std::ptr::null()));
        assert!(uve_project_create(std::ptr::null(), 1280, 720, 30.0).is_null());
    }
}
