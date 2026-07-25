pub fn require_args(args: &[String], min_args: usize, error_msg: &str) -> Result<(), String> {
    if args.len() < min_args {
        return Err(error_msg.to_string());
    }

    Ok(())
}
