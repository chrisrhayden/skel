use new_rs::{
    cli::config_str_to_user_struct, make_project, parse_args, resolve_default,
};

fn main() {
    let args = match parse_args() {
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
        Ok(val) => val,
    };

    let (user_config, config_dir) =
        match config_str_to_user_struct(&args.user_config_path) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("{}", err);
                return;
            }
        };

    let project = match resolve_default(&args, &user_config, &config_dir) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    if let Err(err) = make_project(&project) {
        eprintln!("{}", err);
    }
}
