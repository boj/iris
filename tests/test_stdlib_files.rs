
//! Tests for the IRIS stdlib file operations.
//!
//! Validates the high-level file primitives by exercising the RuntimeEffectHandler
//! effect system directly:
//! - read_file / write_file / append_file round-trip
//! - file_exists via file_stat
//! - read_lines splitting

use iris_exec::effect_runtime::RuntimeEffectHandler;
use iris_types::eval::{EffectHandler, EffectRequest, EffectTag, Value};

// ---------------------------------------------------------------------------
// Helper: perform a file operation via the RuntimeEffectHandler
// ---------------------------------------------------------------------------

fn file_open(handler: &RuntimeEffectHandler, path: &str, mode: i64) -> i64 {
    let req = EffectRequest {
        tag: EffectTag::FileOpen,
        args: vec![Value::String(path.to_string()), Value::Int(mode)],
    };
    match handler.handle(req).unwrap() {
        Value::Int(h) => h,
        other => panic!("file_open: expected Int handle, got {:?}", other),
    }
}

fn file_write_bytes(handler: &RuntimeEffectHandler, handle: i64, data: &[u8]) -> i64 {
    let req = EffectRequest {
        tag: EffectTag::FileWriteBytes,
        args: vec![Value::Int(handle), Value::Bytes(data.to_vec())],
    };
    match handler.handle(req).unwrap() {
        Value::Int(n) => n,
        other => panic!("file_write_bytes: expected Int, got {:?}", other),
    }
}

fn file_read_bytes(handler: &RuntimeEffectHandler, handle: i64, max: i64) -> Vec<u8> {
    let req = EffectRequest {
        tag: EffectTag::FileReadBytes,
        args: vec![Value::Int(handle), Value::Int(max)],
    };
    match handler.handle(req).unwrap() {
        Value::Bytes(b) => b,
        other => panic!("file_read_bytes: expected Bytes, got {:?}", other),
    }
}

fn file_close(handler: &RuntimeEffectHandler, handle: i64) {
    let req = EffectRequest {
        tag: EffectTag::FileClose,
        args: vec![Value::Int(handle)],
    };
    handler.handle(req).unwrap();
}

fn file_stat(handler: &RuntimeEffectHandler, path: &str) -> Result<Value, iris_types::eval::EffectError> {
    let req = EffectRequest {
        tag: EffectTag::FileStat,
        args: vec![Value::String(path.to_string())],
    };
    handler.handle(req)
}

// ---------------------------------------------------------------------------
// Simulate the IRIS stdlib read_file: open(read), read_bytes, close
// ---------------------------------------------------------------------------

fn read_file(handler: &RuntimeEffectHandler, path: &str) -> Vec<u8> {
    let h = file_open(handler, path, 0);
    let contents = file_read_bytes(handler, h, 16_777_216);
    file_close(handler, h);
    contents
}

fn write_file(handler: &RuntimeEffectHandler, path: &str, content: &[u8]) {
    let h = file_open(handler, path, 1);
    file_write_bytes(handler, h, content);
    file_close(handler, h);
}

fn append_file(handler: &RuntimeEffectHandler, path: &str, content: &[u8]) {
    let h = file_open(handler, path, 2);
    file_write_bytes(handler, h, content);
    file_close(handler, h);
}

// ---------------------------------------------------------------------------
// 1. Write + Read round-trip
// ---------------------------------------------------------------------------

#[test]
fn write_then_read_round_trip() {
    let handler = RuntimeEffectHandler::new();
    let dir = std::env::temp_dir().join("iris_stdlib_files");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("round_trip.txt");
    let path_str = path.to_str().unwrap();

    // Clean slate
    let _ = std::fs::remove_file(&path);

    let content = b"Hello, IRIS stdlib!";
    write_file(&handler, path_str, content);

    let read_back = read_file(&handler, path_str);
    assert_eq!(read_back, content.to_vec());

    // Cleanup
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}

// ---------------------------------------------------------------------------
// 2. Append adds to existing content
// ---------------------------------------------------------------------------

#[test]
fn append_adds_to_existing() {
    let handler = RuntimeEffectHandler::new();
    let dir = std::env::temp_dir().join("iris_stdlib_files_append");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("append_test.txt");
    let path_str = path.to_str().unwrap();

    // Clean slate
    let _ = std::fs::remove_file(&path);

    write_file(&handler, path_str, b"first");
    append_file(&handler, path_str, b"second");

    let read_back = read_file(&handler, path_str);
    assert_eq!(read_back, b"firstsecond".to_vec());

    // Cleanup
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}

// ---------------------------------------------------------------------------
// 3. file_exists via file_stat
// ---------------------------------------------------------------------------

#[test]
fn file_exists_via_stat() {
    let handler = RuntimeEffectHandler::new();
    let dir = std::env::temp_dir().join("iris_stdlib_files_exists");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("exists_test.txt");
    let path_str = path.to_str().unwrap();

    // File does not exist yet
    let _ = std::fs::remove_file(&path);
    let result = file_stat(&handler, path_str);
    assert!(result.is_err(), "file_stat should fail for nonexistent file");

    // Create the file
    write_file(&handler, path_str, b"exists");

    // Now it should succeed
    let result = file_stat(&handler, path_str);
    assert!(result.is_ok(), "file_stat should succeed for existing file");

    // Verify stat returns size
    match result.unwrap() {
        Value::Tuple(fields) => {
            match &fields[0] {
                Value::Int(size) => assert_eq!(*size, 6, "file size should be 6 bytes"),
                other => panic!("expected Int size, got {:?}", other),
            }
        }
        other => panic!("expected Tuple from file_stat, got {:?}", other),
    }

    // Cleanup
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}

// ---------------------------------------------------------------------------
// 4. read_lines simulation (read + split by newline)
// ---------------------------------------------------------------------------

#[test]
fn read_lines_splits_by_newline() {
    let handler = RuntimeEffectHandler::new();
    let dir = std::env::temp_dir().join("iris_stdlib_files_lines");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("lines_test.txt");
    let path_str = path.to_str().unwrap();

    // Clean slate
    let _ = std::fs::remove_file(&path);

    let content = b"line1\nline2\nline3";
    write_file(&handler, path_str, content);

    // Simulate read_lines: read file then split
    let bytes = read_file(&handler, path_str);
    let text = String::from_utf8(bytes).unwrap();
    let lines: Vec<&str> = text.split('\n').collect();

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "line1");
    assert_eq!(lines[1], "line2");
    assert_eq!(lines[2], "line3");

    // Cleanup
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}

// ---------------------------------------------------------------------------
// 5. Write empty file and read back
// ---------------------------------------------------------------------------

#[test]
fn write_and_read_empty_file() {
    let handler = RuntimeEffectHandler::new();
    let dir = std::env::temp_dir().join("iris_stdlib_files_empty");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("empty.txt");
    let path_str = path.to_str().unwrap();

    let _ = std::fs::remove_file(&path);

    write_file(&handler, path_str, b"");
    let read_back = read_file(&handler, path_str);
    assert!(read_back.is_empty(), "empty file should read back as empty");

    // Cleanup
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}

// ---------------------------------------------------------------------------
// 6. Multiple appends accumulate
// ---------------------------------------------------------------------------

#[test]
fn multiple_appends_accumulate() {
    let handler = RuntimeEffectHandler::new();
    let dir = std::env::temp_dir().join("iris_stdlib_files_multi_append");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("multi_append.txt");
    let path_str = path.to_str().unwrap();

    let _ = std::fs::remove_file(&path);

    // Append three lines
    append_file(&handler, path_str, b"a\n");
    append_file(&handler, path_str, b"b\n");
    append_file(&handler, path_str, b"c\n");

    let read_back = read_file(&handler, path_str);
    assert_eq!(read_back, b"a\nb\nc\n".to_vec());

    // Cleanup
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}

// ---------------------------------------------------------------------------
// 7. Read nonexistent file — verify error handling
// ---------------------------------------------------------------------------

#[test]
fn test_read_nonexistent_file() {
    let handler = RuntimeEffectHandler::new();
    let path = std::env::temp_dir()
        .join("iris_stdlib_files_nonexist")
        .join("does_not_exist.txt");
    let path_str = path.to_str().unwrap();

    // file_open in read mode (0) should fail for a nonexistent file.
    let req = EffectRequest {
        tag: EffectTag::FileOpen,
        args: vec![Value::String(path_str.to_string()), Value::Int(0)],
    };
    let result = handler.handle(req);
    assert!(result.is_err(), "opening nonexistent file should return error");
    let err = result.unwrap_err();
    assert_eq!(err.tag, EffectTag::FileOpen);
}

// ---------------------------------------------------------------------------
// 8. Write does NOT create intermediate directories
// ---------------------------------------------------------------------------

#[test]
fn test_write_does_not_create_directories() {
    let handler = RuntimeEffectHandler::new();
    let path = std::env::temp_dir()
        .join("iris_stdlib_files_no_mkdir")
        .join("nested")
        .join("deep")
        .join("file.txt");
    let path_str = path.to_str().unwrap();

    // Ensure the parent dir does not exist.
    let _ = std::fs::remove_dir_all(
        std::env::temp_dir().join("iris_stdlib_files_no_mkdir"),
    );

    // file_open in write mode (1) should fail when parent dir is missing.
    let req = EffectRequest {
        tag: EffectTag::FileOpen,
        args: vec![Value::String(path_str.to_string()), Value::Int(1)],
    };
    let result = handler.handle(req);
    assert!(
        result.is_err(),
        "write to file in nonexistent directory should fail"
    );
}

// ---------------------------------------------------------------------------
// 9. Large file round-trip (100KB)
// ---------------------------------------------------------------------------

#[test]
fn test_large_file_roundtrip() {
    let handler = RuntimeEffectHandler::new();
    let dir = std::env::temp_dir().join("iris_stdlib_files_large");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("large_file.bin");
    let path_str = path.to_str().unwrap();

    let _ = std::fs::remove_file(&path);

    // Build 100KB of patterned data.
    let size = 100 * 1024;
    let content: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

    write_file(&handler, path_str, &content);
    let read_back = read_file(&handler, path_str);

    assert_eq!(
        read_back.len(),
        content.len(),
        "read size mismatch: {} vs {}",
        read_back.len(),
        content.len()
    );
    assert_eq!(read_back, content, "large file content mismatch");

    // Cleanup
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}

// ---------------------------------------------------------------------------
// 10. Binary file round-trip (bytes with null bytes)
// ---------------------------------------------------------------------------

#[test]
fn test_binary_file_roundtrip() {
    let handler = RuntimeEffectHandler::new();
    let dir = std::env::temp_dir().join("iris_stdlib_files_binary");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("binary_file.bin");
    let path_str = path.to_str().unwrap();

    let _ = std::fs::remove_file(&path);

    // Content with null bytes, high bytes, and control chars.
    let content: Vec<u8> = vec![
        0x00, 0x01, 0xFF, 0xFE, 0x00, 0x7F, 0x80, 0x00,
        0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x00, 0x00, 0x00,
    ];

    write_file(&handler, path_str, &content);
    let read_back = read_file(&handler, path_str);

    assert_eq!(read_back, content, "binary content mismatch");

    // Cleanup
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}

// ---------------------------------------------------------------------------
// 11. Concurrent file access — two threads write to different files
// ---------------------------------------------------------------------------

#[test]
fn test_concurrent_file_access() {
    use std::sync::Arc;
    use std::thread;

    let dir = std::env::temp_dir().join("iris_stdlib_files_concurrent");
    let _ = std::fs::create_dir_all(&dir);

    let path1 = dir.join("concurrent_1.txt");
    let path2 = dir.join("concurrent_2.txt");
    let _ = std::fs::remove_file(&path1);
    let _ = std::fs::remove_file(&path2);

    let handler = Arc::new(RuntimeEffectHandler::new());

    let h1 = handler.clone();
    let p1 = path1.to_str().unwrap().to_string();
    let t1 = thread::spawn(move || {
        let content = b"thread-1-data-abcdefg";
        let fh = file_open_shared(&h1, &p1, 1);
        file_write_bytes_shared(&h1, fh, content);
        file_close_shared(&h1, fh);
    });

    let h2 = handler.clone();
    let p2 = path2.to_str().unwrap().to_string();
    let t2 = thread::spawn(move || {
        let content = b"thread-2-data-hijklmn";
        let fh = file_open_shared(&h2, &p2, 1);
        file_write_bytes_shared(&h2, fh, content);
        file_close_shared(&h2, fh);
    });

    t1.join().unwrap();
    t2.join().unwrap();

    // Read back via the main handler.
    let data1 = read_file(&handler, path1.to_str().unwrap());
    let data2 = read_file(&handler, path2.to_str().unwrap());

    assert_eq!(data1, b"thread-1-data-abcdefg".to_vec());
    assert_eq!(data2, b"thread-2-data-hijklmn".to_vec());

    // Cleanup
    let _ = std::fs::remove_file(&path1);
    let _ = std::fs::remove_file(&path2);
    let _ = std::fs::remove_dir(&dir);
}

/// Arc-compatible file_open helper.
fn file_open_shared(handler: &RuntimeEffectHandler, path: &str, mode: i64) -> i64 {
    let req = EffectRequest {
        tag: EffectTag::FileOpen,
        args: vec![Value::String(path.to_string()), Value::Int(mode)],
    };
    match handler.handle(req).unwrap() {
        Value::Int(h) => h,
        other => panic!("file_open: expected Int, got {:?}", other),
    }
}

fn file_write_bytes_shared(handler: &RuntimeEffectHandler, handle: i64, data: &[u8]) {
    let req = EffectRequest {
        tag: EffectTag::FileWriteBytes,
        args: vec![Value::Int(handle), Value::Bytes(data.to_vec())],
    };
    handler.handle(req).unwrap();
}

fn file_close_shared(handler: &RuntimeEffectHandler, handle: i64) {
    let req = EffectRequest {
        tag: EffectTag::FileClose,
        args: vec![Value::Int(handle)],
    };
    handler.handle(req).unwrap();
}

// ---------------------------------------------------------------------------
// 12. Integration: file pipeline — write JSON, read, modify, write back, verify
// ---------------------------------------------------------------------------

#[test]
fn test_file_pipeline() {
    let handler = RuntimeEffectHandler::new();
    let dir = std::env::temp_dir().join("iris_stdlib_files_pipeline");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("pipeline.json");
    let path_str = path.to_str().unwrap();

    let _ = std::fs::remove_file(&path);

    // Step 1: Write initial JSON content.
    let initial = br#"{"count":0,"name":"test"}"#;
    write_file(&handler, path_str, initial);

    // Step 2: Read it back.
    let data = read_file(&handler, path_str);
    let text = String::from_utf8(data).unwrap();
    assert!(text.contains(r#""count":0"#), "initial read mismatch");

    // Step 3: "Modify" — replace count:0 with count:42.
    let modified = text.replace(r#""count":0"#, r#""count":42"#);

    // Step 4: Write the modified version back.
    write_file(&handler, path_str, modified.as_bytes());

    // Step 5: Read again and verify.
    let final_data = read_file(&handler, path_str);
    let final_text = String::from_utf8(final_data).unwrap();
    assert!(
        final_text.contains(r#""count":42"#),
        "modified content not found: {}",
        final_text
    );
    assert!(
        final_text.contains(r#""name":"test""#),
        "name field lost: {}",
        final_text
    );

    // Cleanup
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}
