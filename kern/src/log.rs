#[macro_export]
macro_rules! ok {
    () => {
        print!(" [\x1b[32mOK\x1b[0m]\n");
    };
}

#[macro_export]
macro_rules! fail {
    () => {
        print!(" [\x1b[31mFAIL\x1b[0m]\n");
    };
}

macro_rules! check_ok {
    ($msg:expr, $val:expr) => {
        print!("{}", $msg);
        if $val.is_ok() {
            ok!();
        } else {
            fail!();
        };
    };
}
