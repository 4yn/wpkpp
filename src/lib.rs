pub mod vm;
pub mod parse;
pub mod task;
pub mod grader;

pub use grader::do_grade;
pub use parse::do_compress;
pub use parse::check_valid_extension;